//! Golden snapshot of a multi-label diagnostic, rendered as users see it.

use gala_diagnostics::{codes, render, Diagnostic};
use gala_span::{ByteSpan, FileId, SourceMap, Span};

#[test]
fn multi_label_linearity_error_renders() {
    let mut sources = SourceMap::new();
    let src = "fn f(q: Qubit) -> Qubit {\n    let a = q\n    let b = q\n    b\n}\n";
    sources.add_file("classifier.gala".into(), src.to_string());
    let first = src.find("let a = q").unwrap() as u32 + 8;
    let second = src.find("let b = q").unwrap() as u32 + 8;
    let diag = Diagnostic::error(codes::USE_AFTER_CONSUME, "qubit `q` used after it was consumed")
        .with_primary_label(Span::new(FileId(0), ByteSpan::new(second, second + 1)), "used here")
        .with_secondary_label(
            Span::new(FileId(0), ByteSpan::new(first, first + 1)),
            "consumed here",
        )
        .with_note("a qubit is a linear resource")
        .with_help("use the value returned by the operation");
    insta::assert_snapshot!("multi_label_linearity", render(&diag, &sources));
}
