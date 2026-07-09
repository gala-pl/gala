use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: gala-fmt <file.gala>");
        process::exit(1);
    }

    let source = match fs::read_to_string(&args[1]) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read '{}': {e}", args[1]);
            process::exit(1);
        }
    };

    let formatted = format_source(&source);
    println!("{formatted}");
}

fn format_source(source: &str) -> String {
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
