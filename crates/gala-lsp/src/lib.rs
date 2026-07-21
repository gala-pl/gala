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
        // Verify the LSP entrypoint exists, is reachable, and does not panic.
        run();
    }

    #[test]
    fn test_run_does_not_panic() {
        // run() just prints to stderr, shouldn't panic
        run();
    }
}
