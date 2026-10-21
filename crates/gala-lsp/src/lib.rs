//! A small, std-only JSON-RPC language server for Gala.

use gala_driver::check_file;
use gala_span::SourceMap;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};

/// Start the language server and process framed LSP requests until `exit` or
/// EOF. The implementation intentionally covers the core lifecycle,
/// diagnostics, and document synchronization methods without requiring an
/// async runtime.
pub fn run() {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    let mut documents = HashMap::<String, String>::new();

    while let Some(message) = read_message(&mut input) {
        let (response, notifications) = handle_message(&message, &mut documents);
        for notification in notifications {
            let _ = write_message(&mut output, &notification);
        }
        if let Some(response) = response {
            let _ = write_message(&mut output, &response);
        }
        if message.get("method").and_then(Value::as_str) == Some("exit") {
            break;
        }
    }
}

fn read_message<R: BufRead>(reader: &mut R) -> Option<Value> {
    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        let count = reader.read_line(&mut line).ok()?;
        if count == 0 {
            return None;
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        } else if let Ok(value) = serde_json::from_str::<Value>(line.trim()) {
            return Some(value);
        }
    }
    let length = headers.get("content-length")?.parse::<usize>().ok()?;
    let mut body = vec![0_u8; length];
    reader.read_exact(&mut body).ok()?;
    serde_json::from_slice(&body).ok()
}

fn write_message<W: Write>(writer: &mut W, message: &Value) -> io::Result<()> {
    let body = serde_json::to_vec(message).map_err(io::Error::other)?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()
}

fn handle_message(
    message: &Value,
    documents: &mut HashMap<String, String>,
) -> (Option<Value>, Vec<Value>) {
    let id = message.get("id").cloned();
    let method = message.get("method").and_then(Value::as_str).unwrap_or_default();
    let params = message.get("params").cloned().unwrap_or_else(|| json!({}));
    match method {
        "initialize" => (
            id.map(|id| {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "capabilities": {
                            "textDocumentSync": 1,
                            "hoverProvider": false
                        }
                    }
                })
            }),
            Vec::new(),
        ),
        "initialized" | "$/cancelRequest" => (None, Vec::new()),
        "shutdown" => (id.map(|id| json!({"jsonrpc":"2.0", "id":id, "result":null})), Vec::new()),
        "exit" => (None, Vec::new()),
        "textDocument/didOpen" => {
            let uri = params.pointer("/textDocument/uri").and_then(Value::as_str);
            let text = params.pointer("/textDocument/text").and_then(Value::as_str);
            match (uri, text) {
                (Some(uri), Some(text)) => {
                    documents.insert(uri.to_string(), text.to_string());
                    (None, publish_diagnostics(uri, text))
                }
                _ => (None, Vec::new()),
            }
        }
        "textDocument/didChange" => {
            let uri = params.pointer("/textDocument/uri").and_then(Value::as_str);
            let text = params.pointer("/contentChanges/0/text").and_then(Value::as_str);
            match (uri, text) {
                (Some(uri), Some(text)) => {
                    documents.insert(uri.to_string(), text.to_string());
                    (None, publish_diagnostics(uri, text))
                }
                _ => (None, Vec::new()),
            }
        }
        _ if id.is_some() => (
            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {"code": -32601, "message": format!("method not found: {method}")}
            })),
            Vec::new(),
        ),
        _ => (None, Vec::new()),
    }
}

fn publish_diagnostics(uri: &str, source: &str) -> Vec<Value> {
    let mut source_map = SourceMap::new();
    let file_id = source_map.add_file(uri.into(), source.to_string());
    let diagnostics = check_file(file_id, source, &mut source_map);
    let items: Vec<Value> = diagnostics
        .diagnostics
        .iter()
        .map(|diagnostic| {
            json!({
                "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}},
                "severity": 1,
                "code": diagnostic.code.to_string(),
                "message": diagnostic.message
            })
        })
        .collect();
    vec![json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {"uri": uri, "diagnostics": items}
    })]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_and_diagnostics() {
        let mut documents = HashMap::new();
        let initialize = json!({"jsonrpc":"2.0", "id":1, "method":"initialize"});
        let (response, _) = handle_message(&initialize, &mut documents);
        assert_eq!(response.and_then(|value| value.get("id").cloned()), Some(json!(1)));

        let open = json!({
            "jsonrpc":"2.0",
            "method":"textDocument/didOpen",
            "params":{"textDocument":{"uri":"file:///main.gala", "text":"fn main() -> Int { return 1; }"}}
        });
        let (_, notifications) = handle_message(&open, &mut documents);
        assert_eq!(notifications.len(), 1);
        assert!(notifications[0].to_string().contains("publishDiagnostics"));
    }
}
