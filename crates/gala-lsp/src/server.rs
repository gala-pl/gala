//! Request and notification dispatch. `Server::handle` is pure with respect to I/O: it takes one
//! decoded JSON-RPC message and returns the messages to send back, so tests can drive it directly.

use serde_json::{json, Value};
use std::collections::HashMap;

use crate::analysis::{analyze, Analysis};
use crate::position::{lsp_range, position_to_offset};

/// Keywords and builtins offered by completion.
pub const BUILTINS: &[&str] = &[
    "qubit", "qubits", "measure", "drop", "h", "x", "y", "z", "s", "t", "rx", "ry", "rz", "cx",
    "cz", "swap", "cp", "print",
];

const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;

/// Language server state: the open documents and the lifecycle flags.
#[derive(Default)]
pub struct Server {
    docs: HashMap<String, Analysis>,
    shutdown_requested: bool,
    exit_requested: bool,
}

struct RpcError {
    code: i64,
    message: String,
}

fn invalid_params(message: &str) -> RpcError {
    RpcError { code: INVALID_PARAMS, message: message.to_string() }
}

impl Server {
    /// A server with no open documents.
    pub fn new() -> Self {
        Self::default()
    }

    /// True once the client has sent `exit`.
    pub fn exit_requested(&self) -> bool {
        self.exit_requested
    }

    /// Process one message and return the responses and notifications to send, in order.
    pub fn handle(&mut self, msg: &Value) -> Vec<Value> {
        // Responses from the client (no method) are ignored.
        let Some(method) = msg.get("method").and_then(Value::as_str) else {
            return Vec::new();
        };
        let params = msg.get("params").cloned().unwrap_or(Value::Null);
        match msg.get("id") {
            None => self.notification(method, &params),
            Some(id) => {
                let reply = match self.request(method, &params) {
                    Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
                    Err(e) => error_response(id, e.code, &e.message),
                };
                vec![reply]
            }
        }
    }

    fn request(&mut self, method: &str, params: &Value) -> Result<Value, RpcError> {
        match method {
            "initialize" => Ok(initialize_result()),
            "shutdown" => {
                self.shutdown_requested = true;
                Ok(Value::Null)
            }
            "textDocument/hover" => self.hover(params),
            "textDocument/definition" => self.definition(params),
            "textDocument/completion" => self.completion(params),
            "gala/typeAtPosition" => self.type_at_position(params),
            "gala/effectAtPosition" => self.effect_at_position(params),
            "gala/girAtPosition" => self.gir_at_position(params),
            "gala/suggestFixes" => self.suggest_fixes(params),
            _ => Err(RpcError {
                code: METHOD_NOT_FOUND,
                message: format!("method not found: {method}"),
            }),
        }
    }

    fn notification(&mut self, method: &str, params: &Value) -> Vec<Value> {
        match method {
            "exit" => {
                self.exit_requested = true;
                Vec::new()
            }
            "textDocument/didOpen" => {
                match (uri_of(params), params.pointer("/textDocument/text")) {
                    (Ok(uri), Some(text)) => self.update(uri, text.as_str().unwrap_or_default()),
                    _ => Vec::new(),
                }
            }
            "textDocument/didChange" => {
                let text = params
                    .get("contentChanges")
                    .and_then(Value::as_array)
                    .and_then(|changes| changes.last())
                    .and_then(|c| c.get("text"))
                    .and_then(Value::as_str);
                match (uri_of(params), text) {
                    (Ok(uri), Some(text)) => self.update(uri, text),
                    _ => Vec::new(),
                }
            }
            "textDocument/didClose" => match uri_of(params) {
                Ok(uri) => {
                    self.docs.remove(&uri);
                    vec![publish(&uri, &[])]
                }
                Err(_) => Vec::new(),
            },
            // `initialized`, `$/cancelRequest`, and other notifications need no action.
            _ => Vec::new(),
        }
    }

    fn update(&mut self, uri: String, text: &str) -> Vec<Value> {
        let analysis = analyze_guarded(text);
        let note = publish(&uri, &analysis.diagnostics_json());
        self.docs.insert(uri, analysis);
        vec![note]
    }

    /// The document and byte offset named by `params.textDocument.uri` and `params.position`.
    /// `Ok(None)` when the document is not open.
    fn locate(&self, params: &Value) -> Result<Option<(&Analysis, usize)>, RpcError> {
        let uri = uri_of(params)?;
        let line = u32_at(params, "/position/line")?;
        let character = u32_at(params, "/position/character")?;
        Ok(self.docs.get(&uri).map(|doc| (doc, position_to_offset(&doc.text, line, character))))
    }

    fn hover(&self, params: &Value) -> Result<Value, RpcError> {
        let Some((doc, offset)) = self.locate(params)? else { return Ok(Value::Null) };
        let text = doc.symbol_at(offset).and_then(|s| doc.hover_text(s));
        Ok(match text {
            Some(t) => {
                json!({"contents": {"kind": "markdown", "value": format!("```gala\n{t}\n```")}})
            }
            None => Value::Null,
        })
    }

    fn definition(&self, params: &Value) -> Result<Value, RpcError> {
        let uri = uri_of(params)?;
        let Some((doc, offset)) = self.locate(params)? else { return Ok(Value::Null) };
        let span = doc.symbol_at(offset).and_then(|s| doc.definition_span(s));
        Ok(match span {
            Some(span) => json!({
                "uri": uri,
                "range": lsp_range(&doc.text, span.byte_span.start as usize, span.byte_span.end as usize),
            }),
            None => Value::Null,
        })
    }

    fn completion(&self, params: &Value) -> Result<Value, RpcError> {
        let mut items: Vec<Value> = BUILTINS
            .iter()
            .map(|name| json!({"label": name, "kind": 3, "detail": "builtin"}))
            .collect();
        if let Some(uri) = uri_of(params).ok().filter(|u| self.docs.contains_key(u)) {
            if let Some(doc) = self.docs.get(&uri) {
                for name in &doc.declared_fns {
                    items.push(json!({"label": name, "kind": 3, "detail": "function"}));
                }
            }
        }
        Ok(Value::Array(items))
    }

    fn type_at_position(&self, params: &Value) -> Result<Value, RpcError> {
        let Some((doc, offset)) = self.locate(params)? else { return Ok(Value::Null) };
        Ok(match doc.symbol_at(offset).and_then(|s| doc.type_text(s)) {
            Some(t) => json!({"type": t}),
            None => Value::Null,
        })
    }

    fn effect_at_position(&self, params: &Value) -> Result<Value, RpcError> {
        let Some((doc, offset)) = self.locate(params)? else { return Ok(Value::Null) };
        let effect = doc.enclosing_fn(offset).and_then(|def| doc.effect_of(def));
        Ok(match effect {
            Some(e) => json!({"effect": e}),
            None => Value::Null,
        })
    }

    fn gir_at_position(&self, params: &Value) -> Result<Value, RpcError> {
        let Some((doc, offset)) = self.locate(params)? else { return Ok(Value::Null) };
        Ok(match doc.gir_at(offset) {
            Some((function, gir)) => json!({"function": function, "gir": gir}),
            None => Value::Null,
        })
    }

    fn suggest_fixes(&self, params: &Value) -> Result<Value, RpcError> {
        let uri = uri_of(params)?;
        Ok(match self.docs.get(&uri) {
            Some(doc) => Value::Array(doc.fixes_json()),
            None => Value::Array(Vec::new()),
        })
    }
}

/// Analyze, turning a panic inside the compiler into an empty analysis so the server survives.
fn analyze_guarded(text: &str) -> Analysis {
    match std::panic::catch_unwind(|| analyze(text)) {
        Ok(analysis) => analysis,
        Err(_) => {
            eprintln!("gala-lsp: internal error while analyzing a document");
            Analysis::empty(text)
        }
    }
}

fn publish(uri: &str, diagnostics: &[Value]) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {"uri": uri, "diagnostics": diagnostics},
    })
}

fn uri_of(params: &Value) -> Result<String, RpcError> {
    params
        .pointer("/textDocument/uri")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| invalid_params("missing textDocument.uri"))
}

fn u32_at(params: &Value, pointer: &str) -> Result<u32, RpcError> {
    params
        .pointer(pointer)
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(|| invalid_params("missing or invalid position"))
}

fn initialize_result() -> Value {
    json!({
        "capabilities": {
            "textDocumentSync": 1,
            "hoverProvider": true,
            "definitionProvider": true,
            "completionProvider": {},
        },
        "serverInfo": {"name": "gala-lsp", "version": env!("CARGO_PKG_VERSION")},
    })
}

fn error_response(id: &Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

/// A JSON-RPC error response for a message that could not be decoded.
pub fn parse_error_response() -> Value {
    error_response(&Value::Null, -32700, "parse error")
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROGRAM: &str = "fn add(a: Int, b: Int) -> Int {\n    let total = a + b\n    total\n}\n";

    fn open(server: &mut Server, uri: &str, text: &str) -> Vec<Value> {
        server.handle(&json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {"uri": uri, "languageId": "gala", "version": 1, "text": text}}
        }))
    }

    fn request(server: &mut Server, id: u64, method: &str, params: Value) -> Value {
        let out =
            server.handle(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        assert_eq!(out.len(), 1, "one reply for {method}");
        out.into_iter().next().unwrap_or(Value::Null)
    }

    fn at(uri: &str, line: u32, character: u32) -> Value {
        json!({"textDocument": {"uri": uri}, "position": {"line": line, "character": character}})
    }

    #[test]
    fn lifecycle_replies_and_exit() {
        let mut server = Server::new();
        let init = request(&mut server, 1, "initialize", json!({}));
        assert_eq!(init["result"]["capabilities"]["textDocumentSync"], 1);
        assert_eq!(init["result"]["capabilities"]["hoverProvider"], true);
        assert_eq!(init["result"]["capabilities"]["definitionProvider"], true);
        assert!(init["result"]["capabilities"]["completionProvider"].is_object());
        assert!(server
            .handle(&json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}))
            .is_empty());
        let shutdown = request(&mut server, 2, "shutdown", Value::Null);
        assert_eq!(shutdown["result"], Value::Null);
        assert!(!server.exit_requested());
        server.handle(&json!({"jsonrpc": "2.0", "method": "exit"}));
        assert!(server.exit_requested());
    }

    #[test]
    fn unknown_request_is_method_not_found() {
        let mut server = Server::new();
        let reply = request(&mut server, 7, "textDocument/foo", json!({}));
        assert_eq!(reply["error"]["code"], METHOD_NOT_FOUND);
        assert_eq!(reply["id"], 7);
    }

    #[test]
    fn did_open_publishes_clean_diagnostics() {
        let mut server = Server::new();
        let out = open(&mut server, "file:///a.gala", PROGRAM);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0]["method"], "textDocument/publishDiagnostics");
        assert_eq!(out[0]["params"]["uri"], "file:///a.gala");
        assert_eq!(out[0]["params"]["diagnostics"], json!([]));
    }

    #[test]
    fn parse_errors_are_published_on_broken_files() {
        let mut server = Server::new();
        let out = open(&mut server, "file:///broken.gala", "fn f( {\n");
        let diags = out[0]["params"]["diagnostics"].as_array().expect("array");
        assert!(!diags.is_empty());
        assert_eq!(diags[0]["severity"], 1);
        assert!(diags[0]["code"].as_str().is_some_and(|c| c.starts_with('E')));
        assert_eq!(diags[0]["source"], "gala");
    }

    #[test]
    fn did_change_recompiles_full_text() {
        let mut server = Server::new();
        open(&mut server, "file:///a.gala", PROGRAM);
        let out = server.handle(&json!({
            "jsonrpc": "2.0", "method": "textDocument/didChange",
            "params": {"textDocument": {"uri": "file:///a.gala", "version": 2},
                       "contentChanges": [{"text": "fn f( {\n"}]}
        }));
        assert!(!out[0]["params"]["diagnostics"].as_array().expect("array").is_empty());
    }

    #[test]
    fn hover_shows_signature_with_effect() {
        let mut server = Server::new();
        open(&mut server, "file:///a.gala", PROGRAM);
        // Cursor on the `add` in `fn add`.
        let hover = request(&mut server, 2, "textDocument/hover", at("file:///a.gala", 0, 4));
        let value = hover["result"]["contents"]["value"].as_str().expect("hover text");
        assert!(value.contains("fn add(a: Int, b: Int) -> Int pure"), "{value}");
    }

    #[test]
    fn hover_on_local_shows_its_type() {
        let mut server = Server::new();
        open(&mut server, "file:///a.gala", PROGRAM);
        // `total` on the use line.
        let hover = request(&mut server, 3, "textDocument/hover", at("file:///a.gala", 2, 5));
        let value = hover["result"]["contents"]["value"].as_str().expect("hover text");
        assert!(value.contains("total: Int"), "{value}");
    }

    #[test]
    fn hover_on_whitespace_is_null_and_unknown_doc_is_null() {
        let mut server = Server::new();
        open(&mut server, "file:///a.gala", "fn add(a: Int, b: Int) -> Int {\n    a\n}\n");
        let blank = request(&mut server, 4, "textDocument/hover", at("file:///a.gala", 0, 0));
        assert_eq!(blank["result"], Value::Null, "keyword `fn` is not a symbol");
        let missing = request(&mut server, 5, "textDocument/hover", at("file:///none.gala", 0, 0));
        assert_eq!(missing["result"], Value::Null);
    }

    #[test]
    fn definition_of_function_and_local() {
        let mut server = Server::new();
        open(&mut server, "file:///a.gala", PROGRAM);
        // `total` use on line 2, col 4 -> definition on line 1, the `total` after `let`.
        let def = request(&mut server, 6, "textDocument/definition", at("file:///a.gala", 2, 5));
        assert_eq!(def["result"]["uri"], "file:///a.gala");
        assert_eq!(def["result"]["range"]["start"], json!({"line": 1, "character": 8}));
        assert_eq!(def["result"]["range"]["end"], json!({"line": 1, "character": 13}));
        let unresolved =
            request(&mut server, 8, "textDocument/definition", at("file:///a.gala", 0, 0));
        assert_eq!(unresolved["result"], Value::Null);
    }

    #[test]
    fn completion_lists_builtins_and_document_functions() {
        let mut server = Server::new();
        open(&mut server, "file:///a.gala", PROGRAM);
        let out = request(&mut server, 9, "textDocument/completion", at("file:///a.gala", 0, 0));
        let items = out["result"].as_array().expect("array");
        let labels: Vec<&str> = items.iter().filter_map(|i| i["label"].as_str()).collect();
        for b in BUILTINS {
            assert!(labels.contains(b), "missing builtin {b}");
        }
        assert!(labels.contains(&"add"));
        assert!(items.iter().all(|i| i["kind"] == 3));
    }

    #[test]
    fn custom_methods_report_type_effect_and_gir() {
        let mut server = Server::new();
        open(&mut server, "file:///a.gala", PROGRAM);
        let ty = request(&mut server, 10, "gala/typeAtPosition", at("file:///a.gala", 2, 5));
        assert_eq!(ty["result"]["type"], "Int");
        let eff = request(&mut server, 11, "gala/effectAtPosition", at("file:///a.gala", 2, 5));
        assert_eq!(eff["result"]["effect"], "pure");
        let outside = request(&mut server, 12, "gala/effectAtPosition", at("file:///a.gala", 9, 0));
        assert_eq!(outside["result"], Value::Null);
        let gir = request(&mut server, 13, "gala/girAtPosition", at("file:///a.gala", 2, 5));
        assert_eq!(gir["result"]["function"], "add");
        let section = gir["result"]["gir"].as_str().expect("gir text");
        assert!(section.starts_with("fn add #"), "{section}");
        assert!(section.ends_with("}\n"), "section is cut at the closing brace: {section}");
        let fixes = request(&mut server, 14, "gala/suggestFixes", at("file:///a.gala", 0, 0));
        assert_eq!(fixes["result"], json!([]));
    }
}
