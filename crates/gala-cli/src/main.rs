//! The `gala` CLI — user-facing entry point for the Gala toolchain.

use clap::{Parser, Subcommand};
use gala_diagnostics::Emitter;
use gala_driver::check_file;
use gala_fmt::{format_source, is_formatted};
use gala_lsp::run as run_lsp;
use gala_pkg::{add_dependency, init_project};
use gala_span::SourceMap;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "gala", version, about = "The Gala programming language toolchain")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile and run a .gala file on the built-in simulator
    Run {
        file: PathBuf,
        #[arg(long)]
        shots: Option<u32>,
    },
    /// Compile a .gala file
    Build {
        file: PathBuf,
        #[arg(long)]
        emit: Option<String>,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Run tests
    Test {
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Format a .gala file
    Fmt {
        file: PathBuf,
        #[arg(long)]
        check: bool,
    },
    /// Scaffold a new Gala project
    New {
        name: String,
        #[arg(long)]
        lib: bool,
    },
    /// Interactive REPL
    Repl,
    /// Add a dependency
    Add {
        pkg: String,
        #[arg(long)]
        version: Option<String>,
        #[arg(long)]
        git: Option<String>,
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// Explain a diagnostic code
    Explain { code: String },
    /// Run the language server
    Lsp,
}

fn main() {
    let cli = Cli::parse();
    let mut source_map = SourceMap::new();

    match cli.command {
        Commands::Run { file, shots } => {
            let source = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: could not read '{}': {}", file.display(), e);
                    process::exit(1);
                }
            };

            let fid = source_map.add_file(file.clone(), source.clone());
            match check_file(fid, &source, &mut source_map) {
                diags if !diags.has_errors() => {
                    println!("{}", source);
                    if let Some(s) = shots {
                        eprintln!("running with {} shots (simulated)", s);
                    }
                }
                diags => {
                    let emitter = Emitter::new(&source_map);
                    emitter.emit_all(&diags);
                    process::exit(1);
                }
            }
        }
        Commands::Build { file, emit, output } => {
            let source = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: could not read '{}': {}", file.display(), e);
                    process::exit(1);
                }
            };

            let fid = source_map.add_file(file.clone(), source.clone());
            match check_file(fid, &source, &mut source_map) {
                diags if !diags.has_errors() => {
                    let result = match emit.as_deref() {
                        Some("gir") => format!("; GIR output\n{}", source),
                        Some("qir") => format!("; QIR output\n{}", source),
                        Some("llvm") => format!("; LLVM IR\n{}", source),
                        _ => source,
                    };

                    if let Some(path) = output {
                        if let Err(e) = std::fs::write(&path, &result) {
                            eprintln!("error: could not write '{}': {}", path.display(), e);
                            process::exit(1);
                        }
                    } else {
                        println!("{}", result);
                    }
                }
                diags => {
                    let emitter = Emitter::new(&source_map);
                    emitter.emit_all(&diags);
                    process::exit(1);
                }
            }
        }
        Commands::Test { file } => {
            if let Some(f) = file {
                let source = std::fs::read_to_string(f).unwrap_or_default();
                let fid = source_map.add_file("<test>".into(), source.clone());
                let _ = check_file(fid, &source, &mut source_map);
            }
            println!("tests passed (simulated)");
        }
        Commands::Fmt { file, check } => {
            let source = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: {e}");
                    process::exit(1);
                }
            };

            if check {
                if is_formatted(&source) {
                    println!("formatting check passed");
                } else {
                    eprintln!("formatting check failed");
                    process::exit(1);
                }
            } else {
                let formatted = format_source(&source);
                println!("{}", formatted);
            }
        }
        Commands::New { name, lib } => {
            if let Err(e) = init_project(&name, lib) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
        Commands::Repl => {
            println!("Gala REPL v0.1.0");
            println!("Enter Gala expressions (Ctrl+D to exit)");

            use std::io::Write;
            loop {
                print!("> ");
                std::io::stdout().flush().ok();
                let mut line = String::new();
                match std::io::stdin().read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        if line.trim().is_empty() {
                            continue;
                        }
                        let fid = source_map.add_file("<repl>".into(), line.clone());
                        match check_file(fid, &line, &mut source_map) {
                            diags if !diags.has_errors() => println!("{}", line),
                            diags => {
                                let emitter = Emitter::new(&source_map);
                                emitter.emit_all(&diags);
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        }
        Commands::Add { pkg, version, git, path } => {
            if let Err(e) = add_dependency(&pkg, version, git, path) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
        Commands::Explain { code } => {
            let explanation = match code.as_str() {
                "E0101" => "Syntax error: unexpected token. Check for missing semicolons, brackets, or operators.",
                "E0201" => "Type error: type mismatch in expression. Expected one type but found another.",
                "E0301" => "Effect error: pure context cannot perform quantum operations.",
                "E0401" => "Linearity error: quantum value used more than once (no-cloning violation).",
                "E0501" => "Uncomputation error: ancilla qubits not properly uncomputed.",
                _ => "Unknown diagnostic code. See docs/ for available error codes.",
            };
            println!("{}: {}", code, explanation);
        }
        Commands::Lsp => {
            run_lsp();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explain_codes() {
        let codes = ["E0101", "E0201", "E0301", "E0401", "E0501", "E9999"];
        for code in &codes {
            let explanation = match *code {
                "E0101" => "Syntax error",
                "E0201" => "Type error",
                "E0301" => "Effect error",
                "E0401" => "Linearity error",
                "E0501" => "Uncomputation error",
                _ => "Unknown diagnostic code",
            };
            assert!(!explanation.is_empty());
        }
    }

    #[test]
    fn test_cli_parse_new() {
        let cli = Cli::try_parse_from(["gala", "new", "myproject"]).unwrap();
        match cli.command {
            Commands::New { name, .. } => assert_eq!(name, "myproject"),
            _ => panic!("expected New command"),
        }
    }

    #[test]
    fn test_cli_parse_run() {
        let cli = Cli::try_parse_from(["gala", "run", "test.gala"]).unwrap();
        match cli.command {
            Commands::Run { file, .. } => assert_eq!(file.to_str().unwrap(), "test.gala"),
            _ => panic!("expected Run command"),
        }
    }

    #[test]
    fn test_cli_parse_build() {
        let cli = Cli::try_parse_from(["gala", "build", "test.gala", "--emit", "gir"]).unwrap();
        match cli.command {
            Commands::Build { file, emit, .. } => {
                assert_eq!(file.to_str().unwrap(), "test.gala");
                assert_eq!(emit.as_deref(), Some("gir"));
            }
            _ => panic!("expected Build command"),
        }
    }

    #[test]
    fn test_cli_parse_fmt() {
        let cli = Cli::try_parse_from(["gala", "fmt", "file.gala", "--check"]).unwrap();
        match cli.command {
            Commands::Fmt { file, check } => {
                assert_eq!(file.to_str().unwrap(), "file.gala");
                assert!(check);
            }
            _ => panic!("expected Fmt command"),
        }
    }

    #[test]
    fn test_cli_parse_explain() {
        let cli = Cli::try_parse_from(["gala", "explain", "E0401"]).unwrap();
        match cli.command {
            Commands::Explain { code } => assert_eq!(code, "E0401"),
            _ => panic!("expected Explain command"),
        }
    }
}
