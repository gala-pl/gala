//! Markdown API documentation for Gala source files (`gala doc`, WP-036).
//!
//! The lexer discards comments, so this works on the source text rather than the AST. Each
//! top-level `fn` gets a section: its name as a heading, the `///` lines immediately above it,
//! and its declaration header (from `fn` up to the opening `{` or `;`) in a fenced block.

use std::path::Path;

/// Render Markdown documentation for the top-level functions of one Gala source file.
///
/// `module_name` becomes the page title. A `///` block documents the next top-level function
/// only when no blank or other line separates them.
pub fn document_source(source: &str, module_name: &str) -> String {
    let mut out = format!("# {module_name}\n");
    let mut doc: Vec<&str> = Vec::new();
    let mut offset = 0usize;
    for line in source.split_inclusive('\n') {
        let line_start = offset;
        offset += line.len();
        let text = line.trim_end_matches(['\n', '\r']);
        if let Some(rest) = text.strip_prefix("///") {
            doc.push(rest.strip_prefix(' ').unwrap_or(rest));
            continue;
        }
        if text.starts_with("#[") {
            continue;
        }
        if is_top_level_fn(text) {
            let header = header_at(source, line_start);
            out.push_str(&format!("\n## {}\n\n", fn_name(&header)));
            if !doc.is_empty() {
                out.push_str(&doc.join("\n"));
                out.push_str("\n\n");
            }
            out.push_str(&format!("```gala\n{header}\n```\n"));
        }
        doc.clear();
    }
    out
}

/// Read a Gala source file and render its Markdown documentation. The module name is the file
/// stem, so `library/std/gates.gala` is titled `gates`.
pub fn document_file(path: &Path) -> std::io::Result<String> {
    let source = std::fs::read_to_string(path)?;
    let module = path.file_stem().and_then(|s| s.to_str()).unwrap_or("module");
    Ok(document_source(&source, module))
}

/// A line that starts a top-level function declaration (no indentation).
fn is_top_level_fn(line: &str) -> bool {
    line.starts_with("fn ") || line.starts_with("pub fn ")
}

/// The declaration header that starts at `start`: whitespace collapsed to single spaces, with the
/// spaces just inside parentheses removed, ending before the first `{` or `;`.
fn header_at(source: &str, start: usize) -> String {
    let rest = &source[start..];
    let end = rest.find(['{', ';']).unwrap_or_else(|| rest.find('\n').unwrap_or(rest.len()));
    rest[..end]
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace("( ", "(")
        .replace(" )", ")")
        .trim_end()
        .to_string()
}

/// The function name in a header such as `pub fn h(q: Qubit) -> Qubit quantum`.
fn fn_name(header: &str) -> String {
    let after_pub = header.strip_prefix("pub ").unwrap_or(header);
    let after_fn = after_pub.strip_prefix("fn ").unwrap_or(after_pub);
    after_fn.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn gates_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library/std/gates.gala")
    }

    #[test]
    fn documents_the_gates_library() {
        let md = document_file(&gates_path()).expect("gates.gala is readable");
        assert!(md.starts_with("# gates\n"), "{md}");
        assert!(md.contains("## h\n"), "{md}");
        assert!(md.contains("fn h(q: Qubit) -> Qubit quantum"), "{md}");
        assert!(md.contains("Hadamard"), "{md}");
    }

    #[test]
    fn file_without_doc_comments_has_signatures_only() {
        let src = "fn a() -> Int pure { 1 }\nfn b(x: Int) -> Int pure {\n    x\n}\n";
        let md = document_source(src, "m");
        assert_eq!(
            md,
            "# m\n\n## a\n\n```gala\nfn a() -> Int pure\n```\n\n\
             ## b\n\n```gala\nfn b(x: Int) -> Int pure\n```\n"
        );
    }

    #[test]
    fn doc_lines_are_joined_in_order() {
        let src = "/// First line.\n///\n/// Second line.\nfn d() pure;\n";
        let md = document_source(src, "m");
        assert!(md.contains("First line.\n\nSecond line.\n\n```gala\nfn d() pure\n```"), "{md}");
    }

    #[test]
    fn blank_line_detaches_doc_comment() {
        let src = "/// orphan\n\nfn c() pure;\n";
        let md = document_source(src, "m");
        assert!(!md.contains("orphan"), "{md}");
        assert!(md.contains("## c"), "{md}");
    }

    #[test]
    fn multi_line_signature_is_collapsed_to_one_line() {
        let src = "/// Adds.\nfn e(\n    a: Int,\n    b: Int\n) -> Int pure {\n    a + b\n}\n";
        let md = document_source(src, "m");
        assert!(md.contains("```gala\nfn e(a: Int, b: Int) -> Int pure\n```"), "{md}");
    }

    #[test]
    fn indented_functions_and_other_items_are_skipped() {
        let src = "struct S { x: Int }\n    fn inner() pure { 1 }\n";
        let md = document_source(src, "m");
        assert_eq!(md, "# m\n");
    }

    #[test]
    fn pub_prefix_and_type_items_are_handled() {
        let src = "/// Pub.\npub fn p(x: Int) -> Int pure { x }\n";
        let md = document_source(src, "m");
        assert!(md.contains("## p\n\nPub.\n\n```gala\npub fn p(x: Int) -> Int pure\n```"), "{md}");
    }

    #[test]
    fn missing_file_is_an_io_error() {
        let err = document_file(Path::new("/definitely/not/here.gala"));
        assert!(err.is_err());
    }
}
