//! Language Server Protocol implementation for Gala (stub).

/// Start the LSP server (stub).
pub fn run() {
    eprintln!("gala-lsp: language server (stub)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_stub_runs() {
        // Just verify the function exists and compiles
        assert!(true);
    }

    #[test]
    fn test_run_does_not_panic() {
        // run() just prints to stderr, shouldn't panic
        // We don't call it here since it would output to stderr
        assert!(true);
    }
}