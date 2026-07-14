//! Snapshot tests for the Gala formatter.
//!
//! Usage:
//!   cargo test -p gala-fmt --test snapshot_tests
//!   cargo insta review        # accept/reject pending snapshots
//!   INSTA_UPDATE=new cargo test -p gala-fmt --test snapshot_tests

use gala_fmt::format_source;

#[test]
fn all_expression_types() {
    let source = r#"
fn expressions(x: Int, y: Bool, z: Int) -> Int pure {
    let a = if y { 1 } else { 0 };
    let b = match a { 0 => 42, _ => 0 };
    let c = for i in 0..10 { let _ = i; };
    let d = while x > 0 { let _ = x; };
    let e = |a: Int, b: Int| a + b;
    let f = (x, y, z);
    let g = x + y * z - 1;
    return g;
}
"#;
    let formatted = format_source(source);
    insta::assert_debug_snapshot!("all_expression_types", formatted);
}

#[test]
fn struct_definition() {
    let source = r#"
struct Point {
    x: Float,
    y: Float,
}
"#;
    let formatted = format_source(source);
    insta::assert_debug_snapshot!("struct_definition", formatted);
}

#[test]
fn enum_definition() {
    let source = r#"
enum Option<T> {
    Some(T),
    None,
}
"#;
    let formatted = format_source(source);
    insta::assert_debug_snapshot!("enum_definition", formatted);
}

#[test]
fn round_trip() {
    let source = r#"
fn add(a: Int, b: Int) -> Int pure {
    return a + b;
}
"#;
    let formatted = format_source(source);
    let reformatted = format_source(&formatted);
    insta::assert_debug_snapshot!("round_trip_output", reformatted);
    assert_eq!(reformatted, formatted, "formatter is not idempotent");
}
