//! Tree-sitter grammar for Gala (Cargo wrapper).

use std::process::Command;

fn main() {
    // Build the tree-sitter grammar
    let status = Command::new("npm")
        .args(["run", "build"])
        .current_dir("extensions/tree-sitter-gala")
        .status()
        .expect("failed to build tree-sitter grammar");

    if !status.success() {
        panic!("tree-sitter grammar build failed");
    }
}