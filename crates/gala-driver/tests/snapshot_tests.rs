//! Golden snapshots of the GIR text dump. A change to lowering shows up as a snapshot diff.

use gala_driver::{compile_source, Sources};

fn dump(src: &str) -> String {
    let mut sources = Sources::default();
    compile_source(src, &mut sources).expect("compiles").gir.dump()
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
