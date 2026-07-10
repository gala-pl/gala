use std::io::{self, BufRead, Write};

fn main() {
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