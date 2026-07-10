//! Canonical code formatter for the Gala programming language.
//!
//! Formats `.gala` source files with consistent indentation and whitespace.

/// Formats Gala source code with consistent indentation.
///
/// Uses a simple brace-counting algorithm:
/// - Opening `{` or `(` increases indentation
/// - Closing `}` or `)` decreases indentation
/// - Blank lines are preserved
/// - Leading/trailing whitespace is stripped from each line
pub fn format_source(source: &str) -> String {
    let mut out = String::new();
    let mut indent: usize = 0;

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }

        if trimmed.starts_with('}') || trimmed.starts_with(')') {
            indent = indent.saturating_sub(1);
        }

        let pad = "    ".repeat(indent);
        out.push_str(&pad);
        out.push_str(trimmed);
        out.push('\n');

        if trimmed.ends_with('{') || trimmed.ends_with('(') {
            indent += 1;
        }
    }

    out
}

/// Checks if source is already formatted.
pub fn is_formatted(source: &str) -> bool {
    format_source(source) == source
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_formatting() {
        let input = "fn main() -> Int {\nreturn 0;\n}\n";
        let expected = "fn main() -> Int {\n    return 0;\n}\n";
        assert_eq!(format_source(input), expected);
    }

    #[test]
    fn test_nested_blocks() {
        let input = "fn foo() {\nif true {\nbar();\n}\n}\n";
        let expected = "fn foo() {\n    if true {\n        bar();\n    }\n}\n";
        assert_eq!(format_source(input), expected);
    }

    #[test]
    fn test_is_formatted() {
        let formatted = "fn main() -> Int {\n    return 0;\n}\n";
        assert!(is_formatted(formatted));
    }

    #[test]
    fn test_preserves_blank_lines() {
        let input = "fn a() {\n    x();\n\n    y();\n}\n";
        assert_eq!(format_source(input), input);
    }
}
