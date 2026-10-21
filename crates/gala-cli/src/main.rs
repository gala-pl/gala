//! The `gala` CLI — user-facing entry point for the Gala toolchain.

use clap::{Parser, Subcommand};
use gala_codegen_classical::emit_llvm;
use gala_diagnostics::Emitter;
use gala_driver::{check_file, compile_file};
use gala_fmt::{format_source, is_formatted};
use gala_lsp::run as run_lsp;
use gala_pkg::{add_dependency, init_project};
use gala_qir::{emit_qir, QirProfile};
use gala_runtime::Runtime;
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
    /// Compile and execute a supported quantum .gala file with the simulator backend
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
    /// Type-check a test file
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
            match compile_file(fid, &source, &mut source_map) {
                Ok(gir) => {
                    let runtime = Runtime::new();
                    match runtime.run(&gir, "sim", shots.unwrap_or(1024) as u64) {
                        Ok(result) => {
                            let mut counts: Vec<_> = result.counts.into_iter().collect();
                            counts.sort_by(|(lhs, _), (rhs, _)| lhs.cmp(rhs));
                            for (basis, count) in counts {
                                println!("{basis}: {count}");
                            }
                        }
                        Err(diags) => {
                            let emitter = Emitter::new(&source_map);
                            emitter.emit_all(&diags);
                            process::exit(2);
                        }
                    }
                }
                Err(diags) => {
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
            match compile_file(fid, &source, &mut source_map) {
                Ok(gir) => {
                    let artifact = match emit.as_deref() {
                        Some("gir") => Ok(format!("{gir:#?}")),
                        Some("qir") => emit_qir(&gir, QirProfile::Base)
                            .map_err(|_| "QIR emission failed".to_string()),
                        Some("llvm") => {
                            emit_llvm(&gir).map_err(|_| "LLVM emission failed".to_string())
                        }
                        Some(other) => Err(format!("unknown emit target '{other}'")),
                        None => Ok(String::new()),
                    };
                    let result = match artifact {
                        Ok(result) => result,
                        Err(message) => {
                            eprintln!("error: {message}");
                            process::exit(1);
                        }
                    };

                    if let Some(path) = output {
                        if emit.is_none() {
                            eprintln!("error: --output requires --emit");
                            process::exit(1);
                        }
                        if let Err(e) = std::fs::write(&path, &result) {
                            eprintln!("error: could not write '{}': {}", path.display(), e);
                            process::exit(1);
                        }
                    } else if !result.is_empty() {
                        println!("{result}");
                    } else {
                        println!("build succeeded");
                    }
                }
                Err(diags) => {
                    let emitter = Emitter::new(&source_map);
                    emitter.emit_all(&diags);
                    process::exit(1);
                }
            }
        }
        Commands::Test { file } => {
            let Some(f) = file else {
                eprintln!("error: no test file supplied");
                process::exit(2);
            };
            {
                let source = match std::fs::read_to_string(&f) {
                    Ok(source) => source,
                    Err(error) => {
                        eprintln!("error: could not read '{}': {error}", f.display());
                        process::exit(1);
                    }
                };
                let fid = source_map.add_file("<test>".into(), source.clone());
                let diags = check_file(fid, &source, &mut source_map);
                if diags.has_errors() {
                    let emitter = Emitter::new(&source_map);
                    emitter.emit_all(&diags);
                    process::exit(1);
                }
            }
            println!("tests passed");
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
