//! Snapshot tests for the Gala parser.
//!
//! Usage:
//!   cargo test -p gala-parser --test snapshot_tests
//!   cargo insta review        # accept/reject pending snapshots
//!   INSTA_UPDATE=new cargo test -p gala-parser --test snapshot_tests   # create new snapshots

use gala_parser::parse_file;
use gala_span::SourceMap;

#[test]
fn bell_pair_program() {
    let source = r#"
fn bell_pair() -> Qubit quantum {
    let q = qubit();
    let r = qubit();
    h(q);
    cx(q, r);
    return q;
}
"#;
    let mut map = SourceMap::new();
    let fid = map.add_file("bell.gala".into(), source.to_string());
    let items = parse_file(fid, source, &mut map).expect("parsing should succeed");
    insta::assert_debug_snapshot!("bell_pair_ast", items);
}

#[test]
fn classical_program() {
    let source = r#"
fn add(a: Int, b: Int) -> Int pure {
    return a + b;
}

fn main() -> Int pure {
    let x = add(1, 2);
    return x;
}
"#;
    let mut map = SourceMap::new();
    let fid = map.add_file("classical.gala".into(), source.to_string());
    let items = parse_file(fid, source, &mut map).expect("parsing should succeed");
    insta::assert_debug_snapshot!("classical_ast", items);
}

#[test]
fn control_flow_program() {
    let source = r#"
fn classify(x: Bool) -> String pure {
    let result = if x { "yes" } else { "no" };
    return result;
}

fn pick(n: Int) -> String pure {
    let result = match n {
        0 => "zero",
        1 => "one",
        _ => "other",
    };
    return result;
}
"#;
    let mut map = SourceMap::new();
    let fid = map.add_file("control_flow.gala".into(), source.to_string());
    let items = parse_file(fid, source, &mut map).expect("parsing should succeed");
    insta::assert_debug_snapshot!("control_flow_ast", items);
}
