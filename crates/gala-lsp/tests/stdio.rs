//! Spawns the built `gala-lsp` binary and drives it over stdio with real LSP framing.

use gala_lsp::{read_frame, write_frame};
use serde_json::{json, Value};
use std::io::BufReader;
use std::path::Path;
use std::process::{Command, Stdio};

/// Read frames until one satisfies `want`, returning it. Frames seen before it are discarded.
fn recv_until(reader: &mut impl std::io::BufRead, want: impl Fn(&Value) -> bool) -> Value {
    loop {
        let body = read_frame(reader).expect("read frame").expect("server still running");
        let msg: Value = serde_json::from_slice(&body).expect("valid JSON");
        if want(&msg) {
            return msg;
        }
    }
}

#[test]
fn binary_serves_diagnostics_and_hover_over_stdio() {
    let source_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/conformance/linearity/use_after_consume.gala");
    let text = std::fs::read_to_string(&source_path).expect("conformance file");
    let fn_line = text.lines().nth(2).expect("third line");
    assert!(fn_line.starts_with("fn f()"), "fixture changed: {fn_line}");

    let mut child = Command::new(env!("CARGO_BIN_EXE_gala-lsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn gala-lsp");
    let mut stdin = child.stdin.take().expect("stdin");
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout"));
    let uri = "file:///conformance/use_after_consume.gala";

    write_frame(
        &mut stdin,
        &json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}}}),
    )
    .expect("send initialize");
    let init = recv_until(&mut stdout, |m| m["id"] == 1);
    assert_eq!(init["result"]["capabilities"]["textDocumentSync"], 1);
    assert_eq!(init["result"]["capabilities"]["hoverProvider"], true);
    assert_eq!(init["result"]["capabilities"]["definitionProvider"], true);
    assert!(init["result"]["capabilities"]["completionProvider"].is_object());

    write_frame(&mut stdin, &json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}))
        .expect("send initialized");
    write_frame(
        &mut stdin,
        &json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {
            "textDocument": {"uri": uri, "languageId": "gala", "version": 1, "text": text}
        }}),
    )
    .expect("send didOpen");
    let published = recv_until(&mut stdout, |m| m["method"] == "textDocument/publishDiagnostics");
    assert_eq!(published["params"]["uri"], uri);
    let diags = published["params"]["diagnostics"].as_array().expect("diagnostics array");
    let e0412 = diags.iter().find(|d| d["code"] == "E0412").expect("E0412 published");
    assert_eq!(e0412["severity"], 1);
    assert_eq!(e0412["source"], "gala");
    assert!(e0412["message"].as_str().is_some_and(|m| !m.is_empty()));

    // Line 2, character 3 is the `f` in `fn f()`.
    write_frame(
        &mut stdin,
        &json!({"jsonrpc": "2.0", "id": 2, "method": "textDocument/hover", "params": {
            "textDocument": {"uri": uri}, "position": {"line": 2, "character": 3}
        }}),
    )
    .expect("send hover");
    let hover = recv_until(&mut stdout, |m| m["id"] == 2);
    assert_eq!(hover["result"]["contents"]["value"], "```gala\nfn f() -> Measured<Bool> prob\n```");

    write_frame(&mut stdin, &json!({"jsonrpc": "2.0", "id": 3, "method": "shutdown"}))
        .expect("send shutdown");
    let shutdown = recv_until(&mut stdout, |m| m["id"] == 3);
    assert_eq!(shutdown["result"], Value::Null);
    write_frame(&mut stdin, &json!({"jsonrpc": "2.0", "method": "exit"})).expect("send exit");
    drop(stdin);

    let status = child.wait().expect("child exits");
    assert!(status.success(), "exit status {status:?}");
}
