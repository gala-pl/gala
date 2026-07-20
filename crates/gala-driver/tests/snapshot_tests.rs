//! Snapshot tests for the Gala driver (GIR compilation).
//!
//! Usage:
//!   cargo test -p gala-driver --test snapshot_tests
//!   cargo insta review        # accept/reject pending snapshots
//!   INSTA_UPDATE=new cargo test -p gala-driver --test snapshot_tests

use gala_driver::compile_source;
use gala_span::SourceMap;

#[test]
fn simple_pure_function() {
    let source = "fn main() -> Int pure { return 42; }";
    let mut map = SourceMap::new();
    let gir = compile_source(source, &mut map).expect("compilation should succeed");
    insta::assert_json_snapshot!("simple_pure_gir", gir);
}

#[test]
fn quantum_function_rejected_without_uncompute() {
    // The compiler enforces safe uncomputation for the quantum effect: a quantum
    // function that returns a linear `Qubit` without an explicit uncomputation
    // strategy must be rejected. Quantum intrinsics (qubit(), h(), cx(), ...) are
    // not yet implemented, so we exercise the linearity/uncompute safety on a
    // qubit parameter. This verifies the quantum effect path and the E0501 check.
    let source = "fn bell_pair(q: Qubit) -> Qubit quantum { return q; }";
    let mut map = SourceMap::new();
    let result = compile_source(source, &mut map);
    assert!(
        result.is_err(),
        "quantum function returning a Qubit without uncomputation must be rejected"
    );
    let diags = result.err().unwrap();
    assert!(diags.has_errors(), "expected an uncomputation diagnostic for the quantum function");
    let messages: Vec<&str> = diags.diagnostics.iter().map(|d| d.message.as_str()).collect();
    assert!(
        messages.iter().any(|m| m.contains("cannot automatically uncompute")),
        "expected an uncomputation diagnostic, got: {:?}",
        messages
    );
}

#[test]
fn classical_arithmetic() {
    let source = "fn main() -> Int pure { return 1 + 2; }";
    let mut map = SourceMap::new();
    let gir = compile_source(source, &mut map).expect("compilation should succeed");
    insta::assert_json_snapshot!("classical_arithmetic_gir", gir);
}
