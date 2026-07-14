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
fn quantum_function() {
    let source = "fn bell_pair() -> Qubit quantum { let q = qubit(); let r = qubit(); h(q); cx(q, r); return q; }";
    let mut map = SourceMap::new();
    let gir = compile_source(source, &mut map).expect("compilation should succeed");
    insta::assert_json_snapshot!("quantum_bell_gir", gir);
}

#[test]
fn classical_arithmetic() {
    let source = "fn main() -> Int pure { return 1 + 2; }";
    let mut map = SourceMap::new();
    let gir = compile_source(source, &mut map).expect("compilation should succeed");
    insta::assert_json_snapshot!("classical_arithmetic_gir", gir);
}
