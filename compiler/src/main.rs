use clap::Parser;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "gala", version, about = "The Gala programming language compiler")]
struct Cli {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(short, long)]
    emit: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let source = match std::fs::read_to_string(&cli.input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read '{}': {e}", cli.input.display());
            process::exit(1);
        }
    };

    match gala_compiler::compile(&source) {
        Ok(output) => {
            if let Some(out_path) = &cli.output {
                std::fs::write(out_path, &output).unwrap_or_else(|e| {
                    eprintln!("error: could not write '{}': {e}", out_path.display());
                    process::exit(1);
                });
            } else {
                println!("{output}");
            }
        }
        Err(diags) => {
            for diag in diags.diagnostics {
                eprintln!("{diag}");
            }
            process::exit(1);
        }
    }
}
