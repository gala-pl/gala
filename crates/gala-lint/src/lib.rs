//! Linter for Gala source files.

/// A lint diagnostic.
#[derive(Debug, Clone)]
pub struct Lint {
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub code: String,
    pub message: String,
    pub severity: LintSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintSeverity {
    Error,
    Warning,
    Note,
}

/// Run the linter on a source file given as a string.
pub fn lint_source(source: &str, file_path: &str) -> Vec<Lint> {
    let mut lints = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let num_lines = lines.len();

    for (line_num, line) in lines.iter().enumerate() {
        let line_num = line_num + 1;

        if line.len() > 100 {
            lints.push(Lint {
                file: file_path.to_string(),
                line: line_num,
                col: 100,
                code: "W0102".to_string(),
                message: format!("line too long ({} > 100 columns)", line.len()),
                severity: LintSeverity::Warning,
            });
        }

        if line.contains('\t') {
            lints.push(Lint {
                file: file_path.to_string(),
                line: line_num,
                col: line.find('\t').unwrap_or(0),
                code: "W0104".to_string(),
                message: "tab character detected; use spaces for indentation".to_string(),
                severity: LintSeverity::Warning,
            });
        }

        for marker in &["TODO", "FIXME", "HACK", "XXX"] {
            if let Some(pos) = line.find(marker) {
                lints.push(Lint {
                    file: file_path.to_string(),
                    line: line_num,
                    col: pos,
                    code: "W0103".to_string(),
                    message: format!("leftover {} marker in source", marker),
                    severity: LintSeverity::Warning,
                });
            }
        }

        let trimmed = line.trim_end();
        if trimmed.len() < line.len() && !trimmed.is_empty() {
            lints.push(Lint {
                file: file_path.to_string(),
                line: line_num,
                col: trimmed.len(),
                code: "W0105".to_string(),
                message: "trailing whitespace detected".to_string(),
                severity: LintSeverity::Note,
            });
        }
    }

    if !source.ends_with('\n') && num_lines > 0 {
        lints.push(Lint {
            file: file_path.to_string(),
            line: num_lines,
            col: lines[num_lines - 1].len(),
            code: "W0101".to_string(),
            message: "missing trailing newline at end of file".to_string(),
            severity: LintSeverity::Note,
        });
    }

    lints
}

/// Format lint results as a string.
pub fn lint_to_string(lints: &[Lint]) -> String {
    let mut out = String::new();
    for lint in lints {
        let level = match lint.severity {
            LintSeverity::Error => "error",
            LintSeverity::Warning => "warning",
            LintSeverity::Note => "note",
        };
        out.push_str(&format!(
            "{}:{}:{}: {} [{}] {}\n",
            lint.file, lint.line, lint.col, level, lint.code, lint.message
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_too_long() {
        let source = format!("// {}\n", "x".repeat(120));
        let lints = lint_source(&source, "test.gala");
        assert!(lints.iter().any(|l| l.code == "W0102"));
    }

    #[test]
    fn test_tab_detected() {
        let source = "\tlet x = 1;\n";
        let lints = lint_source(source, "test.gala");
        assert!(lints.iter().any(|l| l.code == "W0104"));
    }

    #[test]
    fn test_todo_marker() {
        let source = "// TODO: implement this\n";
        let lints = lint_source(source, "test.gala");
        assert!(lints.iter().any(|l| l.code == "W0103"));
    }

    #[test]
    fn test_trailing_whitespace() {
        let source = "let x = 1;   \n";
        let lints = lint_source(source, "test.gala");
        assert!(lints.iter().any(|l| l.code == "W0105"));
    }

    #[test]
    fn test_missing_newline() {
        let source = "fn main() -> Int { return 0; }";
        let lints = lint_source(source, "test.gala");
        assert!(lints.iter().any(|l| l.code == "W0101"));
    }

    #[test]
    fn test_clean_file_no_lints() {
        let source = "fn main() -> Int {\n    return 0;\n}\n";
        let lints = lint_source(source, "test.gala");
        assert!(lints.is_empty(), "expected no lints, got: {:?}", lints);
    }
}
