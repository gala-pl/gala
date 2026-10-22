//! Suggested fixes: text recipes for selected diagnostics, each validated by recompiling.
//!
//! A recipe is offered only when the edited program compiles with strictly fewer errors and
//! introduces no new error code. This keeps `gala fix --apply` from trading one error for another.

use gala_diagnostics::Diagnostic;
use gala_driver::{compile_source, Sources};

/// One text replacement: bytes `start..end` become `new_text`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub start: usize,
    pub end: usize,
    pub new_text: String,
}

/// A validated fix for one diagnostic.
#[derive(Debug, Clone)]
pub struct Fix {
    pub description: String,
    pub edits: Vec<Edit>,
    pub fixed_source: String,
}

/// Apply edits (sorted by start, non-overlapping) to a source text.
pub fn apply_edits(source: &str, edits: &[Edit]) -> String {
    let mut sorted = edits.to_vec();
    sorted.sort_by_key(|e| std::cmp::Reverse(e.start));
    let mut out = source.to_string();
    for e in sorted {
        out.replace_range(e.start..e.end, &e.new_text);
    }
    out
}

/// Error codes and count for a source text (empty when it compiles).
fn errors_of(source: &str) -> Vec<String> {
    let mut sources = Sources::default();
    match compile_source(source, &mut sources) {
        Ok(_) => Vec::new(),
        Err(d) => d.errors().iter().map(|e| e.code.to_string()).collect(),
    }
}

/// The name in the first backticked span of a message, e.g. `q` in "qubit `q` is never consumed".
fn quoted_name(message: &str) -> Option<String> {
    let start = message.find('`')? + 1;
    let end = start + message[start..].find('`')?;
    Some(message[start..end].to_string())
}

/// Start of the line containing byte `pos`.
fn line_start(source: &str, pos: usize) -> usize {
    source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0)
}

/// Leading whitespace of the line containing `pos`.
fn indent_at(source: &str, pos: usize) -> String {
    let ls = line_start(source, pos);
    source[ls..].chars().take_while(|c| *c == ' ' || *c == '\t').collect()
}

/// Candidate edits for one diagnostic, before validation.
fn candidates(source: &str, d: &Diagnostic) -> Vec<(String, Vec<Edit>)> {
    let mut out = Vec::new();
    let Some(name) = quoted_name(&d.message) else { return out };
    let Some(span) = d.labels.first().map(|l| l.span.byte_span) else { return out };
    let start = span.start as usize;
    match d.code.to_string().as_str() {
        // Assignment to an immutable binding: declare the binding `mut`.
        "E0203" => {
            if let Some(pos) = find_let(source, &name) {
                out.push((
                    format!("declare `{name}` as `mut`"),
                    vec![Edit { start: pos + 4, end: pos + 4, new_text: "mut ".into() }],
                ));
            }
        }
        // A qubit that is never consumed: release it before its scope ends.
        "E0402" if d.message.contains("never consumed") => {
            let indent = indent_at(source, start);
            if let Some(close) = scope_end(source, start) {
                let ls = line_start(source, close);
                let before_brace = &source[ls..close];
                let edit = if before_brace.trim().is_empty() {
                    Edit { start: ls, end: ls, new_text: format!("{indent}drop({name})\n") }
                } else {
                    Edit {
                        start: close,
                        end: close,
                        new_text: format!("\n{indent}drop({name})\n{indent}"),
                    }
                };
                out.push((
                    format!("release `{name}` with drop({name}) before the end of its scope"),
                    vec![edit],
                ));
            }
        }
        _ => {}
    }
    out
}

/// Byte offset of the `let` that binds `name`, pointing at the `l`.
fn find_let(source: &str, name: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(i) = source[from..].find("let ") {
        let at = from + i;
        let rest = source[at + 4..].trim_start_matches("mut ").trim_start();
        let word: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
        if word == name {
            return Some(at);
        }
        from = at + 4;
    }
    None
}

/// Position of the `}` that closes the block containing `pos`.
fn scope_end(source: &str, pos: usize) -> Option<usize> {
    let mut depth = 0i64;
    for (i, c) in source[pos..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                if depth == 0 {
                    return Some(pos + i);
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    None
}

/// Fixes for a source text, validated: each one must reduce the error count without adding codes.
pub fn suggest(source: &str, diagnostics: &[Diagnostic]) -> Vec<Fix> {
    let before = errors_of(source);
    let mut out = Vec::new();
    for d in diagnostics {
        for (description, edits) in candidates(source, d) {
            let fixed = apply_edits(source, &edits);
            let after = errors_of(&fixed);
            let introduces_new = after.iter().any(|c| !before.contains(c));
            if after.len() < before.len() && !introduces_new {
                out.push(Fix { description, edits, fixed_source: fixed });
                break;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diags_of(src: &str) -> Vec<Diagnostic> {
        let mut sources = Sources::default();
        match compile_source(src, &mut sources) {
            Ok(_) => Vec::new(),
            Err(d) => d.diagnostics,
        }
    }

    #[test]
    fn immutable_assignment_gets_mut_fix() {
        let src = "fn f() -> Int {\n    let x = 1\n    x = 2\n    x\n}\n";
        let fixes = suggest(src, &diags_of(src));
        assert_eq!(fixes.len(), 1);
        assert!(fixes[0].fixed_source.contains("let mut x = 1"));
        assert!(errors_of(&fixes[0].fixed_source).is_empty());
    }

    #[test]
    fn unconsumed_qubit_gets_drop_before_scope_end() {
        let src = "fn f() quantum {\n    let q = qubit()\n}\n";
        let fixes = suggest(src, &diags_of(src));
        assert_eq!(fixes.len(), 1, "{:?}", diags_of(src));
        assert!(fixes[0].fixed_source.contains("drop(q)"));
        assert!(
            errors_of(&fixes[0].fixed_source).is_empty(),
            "{:?}",
            errors_of(&fixes[0].fixed_source)
        );
    }

    #[test]
    fn fix_is_not_offered_when_it_would_add_an_error() {
        // Dropping `q` after `measure` would be a use-after-consume, so no fix is offered.
        let src = "fn f() -> Measured<Bool> prob {\n    let q = qubit()\n    let r = measure(q)\n    r\n}\n";
        assert!(suggest(src, &diags_of(src)).is_empty());
    }

    #[test]
    fn apply_edits_splices_in_reverse_order() {
        let out = apply_edits(
            "abcdef",
            &[
                Edit { start: 1, end: 2, new_text: "X".into() },
                Edit { start: 4, end: 5, new_text: "YY".into() },
            ],
        );
        assert_eq!(out, "aXcdYYf");
    }
}
