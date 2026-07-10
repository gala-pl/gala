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

    let formatted = gala_fmt::format_source(&source);
    println!("{formatted}");
}