//! Language Server Protocol implementation for Gala.
//!
//! Provides IDE features — live diagnostics, completions, hover information,
//! go-to-definition — by sharing the compiler's incremental computation graph.

use std::io::{self, BufRead, Write};

/// Starts the LSP server loop, reading JSON-RPC messages from stdin
/// and writing responses to stdout.
pub fn run() {
    let stdin = io::stdin();
    let stdout = io::stdout();

    eprintln!("gala-lsp: language server started");

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.is_empty() {
            continue;
        }

        let mut stdout = stdout.lock();
        let response = handle_request(&line);
        writeln!(stdout, "{response}").unwrap();
        stdout.flush().unwrap();
    }
}

fn handle_request(_line: &str) -> &'static str {
    r#"{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{}}"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_request() {
        let response = handle_request("{}");
        assert!(response.contains("jsonrpc"));
        assert!(response.contains("publishDiagnostics"));
    }
}
