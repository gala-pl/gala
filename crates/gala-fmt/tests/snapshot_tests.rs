//! Snapshot tests for the Gala formatter, run on real programs from the conformance corpus.
//!
//! Each snapshot is the formatted output of one program. A change in the formatter shows as a
//! snapshot diff; `INSTA_UPDATE=always cargo test -p gala-fmt --test snapshot_tests` accepts it.

use gala_fmt::{format_source, is_formatted};

#[test]
fn bell_pair_is_canonical() {
    let source = "fn bell() -> Qubits<2> quantum {\n let a = qubit()\n let b = qubit()\n h(a)\n let (a, b) = cx(a, b)\n (a, b)\n}\n";
    let formatted = format_source(source);
    insta::assert_snapshot!("bell_pair_formatted", formatted);
    assert!(is_formatted(&formatted));
}

#[test]
fn control_flow_is_indented() {
    let source = "fn main() -> Int {\nlet mut s = 0\nfor i in 0..4 {\ns += i\n}\nif s > 3 {\nprint(s)\n} else {\nprint(0)\n}\n0\n}\n";
    let formatted = format_source(source);
    insta::assert_snapshot!("control_flow_formatted", formatted);
}

#[test]
fn formatting_is_idempotent_on_messy_input() {
    let source = "fn add(a: Int,   b: Int) -> Int pure {\n\n\n      return a + b;\n}\n";
    let once = format_source(source);
    assert_eq!(format_source(&once), once, "formatter is not idempotent");
}
