//! Golden snapshots of the GIR text dump and JSON serialization.
//! A change to lowering shows up as a snapshot diff.

use gala_driver::{compile_source, gir_json, Sources};

fn dump(src: &str) -> String {
    let mut sources = Sources::default();
    compile_source(src, &mut sources).expect("compiles").gir.dump()
}

fn dump_json(src: &str) -> serde_json::Value {
    let mut sources = Sources::default();
    let program = compile_source(src, &mut sources).expect("compiles");
    gir_json(&program)
}

#[test]
fn bell_pair_gir() {
    let src = "fn main() -> Measured<Bool> prob {\n    let a = qubit()\n    let b = qubit()\n    h(a)\n    let (a, b) = cx(a, b)\n    let r = measure(a)\n    measure(b)\n}\n";
    insta::assert_snapshot!("bell_pair_gir", dump(src));
}

#[test]
fn classical_gir() {
    let src = "fn add(a: Int, b: Int) -> Int pure {\n    a + b\n}\n\nfn main() -> Int {\n    add(1, 2)\n}\n";
    insta::assert_snapshot!("classical_gir", dump(src));
}

#[test]
fn qft_gir() {
    let src = "import gala.algorithms.{ qft2 }\n\nfn main() -> Int prob {\n    let q = qubits<2>()\n    x(q[0])\n    let q = qft2(q)\n    let r = measure(q)\n    print(r)\n    0\n}\n";
    insta::assert_snapshot!("qft_gir", dump(src));
}

#[test]
fn bell_pair_gir_json() {
    let src = "fn main() -> Measured<Bool> prob {\n    let a = qubit()\n    let b = qubit()\n    h(a)\n    let (a, b) = cx(a, b)\n    let r = measure(a)\n    measure(b)\n}\n";
    insta::assert_json_snapshot!("bell_pair_gir_json", dump_json(src));
}

#[test]
fn classical_gir_json() {
    let src = "fn add(a: Int, b: Int) -> Int pure {\n    a + b\n}\n\nfn main() -> Int {\n    add(1, 2)\n}\n";
    insta::assert_json_snapshot!("classical_gir_json", dump_json(src));
}

#[test]
fn qft_gir_json() {
    let src = "import gala.algorithms.{ qft2 }\n\nfn main() -> Int prob {\n    let q = qubits<2>()\n    x(q[0])\n    let q = qft2(q)\n    let r = measure(q)\n    print(r)\n    0\n}\n";
    insta::assert_json_snapshot!("qft_gir_json", dump_json(src));
}
