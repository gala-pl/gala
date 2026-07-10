//! `gala-lint` — the Gala linter CLI.

use clap::Parser;
use gala_lint::{lint_source, lint_to_string, LintSeverity};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "gala-lint", version, about = "Linter for Gala source files")]
struct Cli {
    /// Files to lint
    files: Vec<PathBuf>,

    /// Lint level threshold: error, warning, note
    #[arg(short, long, default_value = "warning")]
    level: String,

    /// Automatically fix fixable issues
    #[arg(long)]
    fix: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.files.is_empty() {
        eprintln!("error: no files specified");
        process::exit(1);
    }

    let threshold = match cli.level.as_str() {
        "error" => LintSeverity::Error,
        "warning" => LintSeverity::Warning,
        "note" => LintSeverity::Note,
        _ => {
            eprintln!("error: unknown level '{}' (use: error, warning, note)", cli.level);
            process::exit(1);
        }
    };

    let mut all_lints = Vec::new();
    let mut has_errors = false;

    for file in &cli.files {
        let source = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: could not read '{}': {}", file.display(), e);
                process::exit(1);
            }
        };

        let lints = lint_source(&source, &file.to_string_lossy());
        for lint in &lints {
            if lint.severity as u8 >= threshold as u8 {
                all_lints.push(lint.clone());
                if lint.severity == LintSeverity::Error {
                    has_errors = true;
                }
            }
        }
    }

    let output = lint_to_string(&all_lints);
    if !output.is_empty() {
        eprint!("{}", output);
        eprintln!("\n{} lint(s) found", all_lints.len());
    } else {
        println!("no issues found");
    }

    if has_errors {
        process::exit(1);
    }
}
