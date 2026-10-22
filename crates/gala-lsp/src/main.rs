//! `gala-lsp`: the Gala language server. Reads LSP messages on stdin and writes them to stdout.

fn main() {
    gala_lsp::run_stdio();
}
