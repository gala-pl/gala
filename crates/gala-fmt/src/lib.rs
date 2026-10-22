//! Canonical formatter for Gala source.
//!
//! One style, no options. The formatter works on lines so that comments are never lost (the
//! lexer discards them):
//!
//! - indentation is four spaces per open `{`, `(` or `[`, computed by a scanner that ignores
//!   brackets inside strings and comments;
//! - trailing whitespace is removed, and runs of blank lines collapse to one;
//! - the file ends with exactly one newline.
//!
//! Lines inside a multi-line string or block comment keep their original text. The result is
//! idempotent: `format(format(x)) == format(x)`.

const INDENT: &str = "    ";

/// Format a source file.
pub fn format_source(source: &str) -> String {
    let mut out = String::new();
    let mut depth: i64 = 0;
    // Lines whose interior is part of a string or block comment keep their text unchanged.
    let mut verbatim = false;
    let mut blank_pending = false;
    let mut scanner = Scanner::default();

    for raw in source.lines() {
        let line = raw.trim_end();
        if !verbatim && line.trim().is_empty() {
            blank_pending = !out.is_empty();
            continue;
        }
        if blank_pending {
            out.push('\n');
            blank_pending = false;
        }
        let starts_in_string = scanner.in_string || scanner.block_depth > 0;
        let closers_first = leading_closers(line);
        let indent_depth = (depth - closers_first).max(0);
        if starts_in_string {
            out.push_str(line);
        } else {
            out.push_str(&INDENT.repeat(indent_depth as usize));
            out.push_str(line.trim_start());
        }
        out.push('\n');
        let delta = scanner.scan_line(line);
        depth = (depth + delta).max(0);
        verbatim = scanner.in_string || scanner.block_depth > 0;
    }
    out
}

/// True when formatting would change the text.
pub fn is_formatted(source: &str) -> bool {
    format_source(source) == source
}

/// Number of closing brackets at the start of a line (they dedent that line).
fn leading_closers(line: &str) -> i64 {
    let mut n = 0;
    for c in line.trim_start().chars() {
        match c {
            '}' | ')' | ']' => n += 1,
            _ => break,
        }
    }
    n
}

/// Tracks bracket depth across lines, skipping strings and comments.
#[derive(Default)]
struct Scanner {
    in_string: bool,
    block_depth: u32,
}

impl Scanner {
    /// Scan one line, returning the net change in bracket depth it causes.
    fn scan_line(&mut self, line: &str) -> i64 {
        let mut delta = 0i64;
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if self.in_string {
                match c {
                    '\\' => {
                        chars.next();
                    }
                    '"' => self.in_string = false,
                    _ => {}
                }
                continue;
            }
            if self.block_depth > 0 {
                if c == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    self.block_depth -= 1;
                } else if c == '/' && chars.peek() == Some(&'*') {
                    chars.next();
                    self.block_depth += 1;
                }
                continue;
            }
            match c {
                '"' => self.in_string = true,
                '/' if chars.peek() == Some(&'/') => break,
                '/' if chars.peek() == Some(&'*') => {
                    chars.next();
                    self.block_depth += 1;
                }
                '{' | '(' | '[' => delta += 1,
                '}' | ')' | ']' => delta -= 1,
                _ => {}
            }
        }
        delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indents_nested_blocks() {
        let src = "fn f() -> Int {\nlet x = 1\nif x > 0 {\nx\n}\n}\n";
        let want = "fn f() -> Int {\n    let x = 1\n    if x > 0 {\n        x\n    }\n}\n";
        assert_eq!(format_source(src), want);
    }

    #[test]
    fn is_idempotent() {
        let src = "fn f(a: Int) -> Int {\n      let y = (a +\n1)\n\n\n\n  y\n}";
        let once = format_source(src);
        assert_eq!(format_source(&once), once);
    }

    #[test]
    fn keeps_comments_and_strings_intact() {
        let src = "// keep me\nfn f() -> String {\n\"{ not a block\"\n}\n";
        let out = format_source(src);
        assert!(out.contains("// keep me"));
        assert!(out.contains("    \"{ not a block\""));
        // The brace inside the string is text, not a block: exactly one real closing brace.
        assert_eq!(out.matches('}').count(), 1);
    }

    #[test]
    fn collapses_blank_lines_and_trims_trailing_space() {
        let out = format_source("fn f() {}   \n\n\n\nfn g() {}\n");
        assert_eq!(out, "fn f() {}\n\nfn g() {}\n");
    }

    #[test]
    fn ends_with_single_newline() {
        assert!(format_source("fn f() {}\n\n\n").ends_with("}\n"));
        assert!(!format_source("fn f() {}\n\n\n").ends_with("\n\n"));
    }

    #[test]
    fn check_detects_unformatted_input() {
        assert!(!is_formatted("fn f() {\nx\n}\n"));
        assert!(is_formatted("fn f() {\n    x\n}\n"));
    }

    #[test]
    fn block_comment_interior_is_preserved() {
        let src = "/* a\n   b */\nfn f() {}\n";
        assert!(format_source(src).contains("   b */"));
    }
}
