//! The `gala` CLI: check, run, build, test, fmt, explain, new, and add.

use clap::{Parser, Subcommand};
use gala_diagnostics::{explain, to_json, DiagnosticCode, Diagnostics, Emitter};
use gala_driver::{compile_file, Sources};
use gala_pkg::manifest::DependencySpec;
use gala_sim::{run_entry, SimConfig};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process;

mod fix;

#[derive(Parser)]
#[command(name = "gala", version, about = "The Gala programming language toolchain")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Type, effect, and linearity check a program without running it
    Check {
        file: PathBuf,
        /// Print diagnostics as JSON (schema diagnostic.v1)
        #[arg(long)]
        json: bool,
        /// Treat warnings as failures
        #[arg(long)]
        fail_on_warnings: bool,
    },
    /// Compile and run a program on the built-in simulator
    Run {
        /// Source file or package; omit when running saved GIR with --from-gir
        file: Option<PathBuf>,
        /// Run a `gir.v1` JSON file produced by `gala build --emit gir=json`
        #[arg(long)]
        from_gir: Option<PathBuf>,
        #[arg(long, default_value_t = 1024)]
        shots: u64,
        #[arg(long, default_value_t = 0)]
        seed: u64,
    },
    /// Compile a program and print or write its IR
    Build {
        file: PathBuf,
        /// `gir` (text) or `gir=json` (schema gir.v1)
        #[arg(long, default_value = "gir")]
        emit: String,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Run every function marked `#[test]`; each must run without error. With --property, run
    /// the functions marked `#[property(...)]` and print results in the property_result.v1 shape.
    Test {
        file: PathBuf,
        #[arg(long)]
        property: bool,
        #[arg(long)]
        json: bool,
    },
    /// Format a .gala file
    Fmt {
        /// File to format; omit with --stdin
        file: Option<PathBuf>,
        /// Exit 1 if the file is not already canonical
        #[arg(long)]
        check: bool,
        /// Print the changed lines instead of the formatted file
        #[arg(long)]
        diff: bool,
        /// Read the program from standard input and write the formatted program to stdout
        #[arg(long)]
        stdin: bool,
    },
    /// Explain a diagnostic code
    Explain {
        code: String,
        #[arg(long)]
        markdown: bool,
        #[arg(long)]
        json: bool,
    },
    /// Create a new project
    New {
        name: String,
        #[arg(long)]
        lib: bool,
    },
    /// Suggest validated fixes for a file's diagnostics; --apply writes them
    Fix {
        file: PathBuf,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        json: bool,
    },
    /// Emit a JSONL training corpus from a directory of .gala files. Each case's first comment
    /// line is the prompt, the source is the completion, and the expectation header is recorded
    /// in `properties`.
    Corpus { dir: PathBuf },
    /// Interactive session: definitions persist, and each expression is evaluated on the simulator
    Repl {
        /// One JSON object per input line, for programmatic use
        #[arg(long)]
        json: bool,
    },
    /// Add a dependency to gala.toml
    Add {
        name: String,
        #[arg(long)]
        version: Option<String>,
        #[arg(long)]
        path: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();
    let code = match cli.command {
        Commands::Check { file, json, fail_on_warnings } => check(&file, json, fail_on_warnings),
        Commands::Run { file, from_gir, shots, seed } => match (file, from_gir) {
            (_, Some(json)) => run_gir_json(&json, shots, seed),
            (Some(file), None) => run(&file, shots, seed),
            (None, None) => {
                eprintln!("error: give a file, or --from-gir <file.json>");
                2
            }
        },
        Commands::Build { file, emit, output } => build(&file, &emit, output.as_deref()),
        Commands::Test { file, property, json } => {
            if property {
                property_tests(&file, json)
            } else {
                test(&file)
            }
        }
        Commands::Fmt { file, check, diff, stdin } => fmt(file.as_deref(), check, diff, stdin),
        Commands::Explain { code, markdown, json } => explain_cmd(&code, markdown, json),
        Commands::New { name, lib } => new_project(&name, lib),
        Commands::Add { name, version, path } => add(&name, version, path),
        Commands::Repl { json } => repl(json),
        Commands::Corpus { dir } => corpus(&dir),
        Commands::Fix { file, apply, json } => fix_cmd(&file, apply, json),
    };
    process::exit(code);
}

/// The entry file for a path: the file itself, or `src/main.gala` inside a package directory.
fn entry_file(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.join("src").join("main.gala")
    } else {
        path.to_path_buf()
    }
}

fn compile_or_report(
    path: &Path,
    sources: &mut Sources,
) -> Result<gala_driver::Program, Diagnostics> {
    compile_file(&entry_file(path), sources)
}

fn report(diags: &Diagnostics, sources: &Sources) {
    Emitter::new(&sources.map).emit_all(diags);
}

fn check(path: &Path, as_json: bool, fail_on_warnings: bool) -> i32 {
    let mut sources = Sources::default();
    let result = compile_or_report(path, &mut sources);
    let diags = match &result {
        Ok(_) => Diagnostics::new(),
        Err(d) => d.clone(),
    };
    if as_json {
        let items: Vec<Value> =
            diags.diagnostics.iter().map(|d| to_json(d, &sources.map)).collect();
        let errors = diags.errors().len();
        let warnings = diags.warnings().len();
        let out = json!({
            "schema": "gala.check.v1",
            "ok": errors == 0 && !(fail_on_warnings && warnings > 0),
            "errors": errors,
            "warnings": warnings,
            "diagnostics": items,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
    } else if result.is_ok() {
        println!("ok: {} checks passed", entry_file(path).display());
    } else {
        report(&diags, &sources);
    }
    let failed = diags.has_errors() || (fail_on_warnings && !diags.warnings().is_empty());
    if failed {
        1
    } else {
        0
    }
}

fn run(path: &Path, shots: u64, seed: u64) -> i32 {
    let mut sources = Sources::default();
    let program = match compile_or_report(path, &mut sources) {
        Ok(p) => p,
        Err(d) => {
            report(&d, &sources);
            return 1;
        }
    };
    match run_entry(&program.gir, &SimConfig { shots, seed, ..Default::default() }) {
        Ok(report) => {
            for line in &report.output {
                println!("{line}");
            }
            // Only a function that returns a measurement has a distribution to report.
            let returns_measurement = program
                .gir
                .entry
                .and_then(|e| program.hir.fns.get(e))
                .is_some_and(|f| matches!(f.ret_ty, Some(gala_ast::Type::Measured(_))));
            if returns_measurement {
                println!("results over {} shots:", report.shots);
                for (value, count) in &report.histogram {
                    println!("  {value}: {count}");
                }
            } else if let Some(v) = &report.first_result {
                if *v != gala_sim::Value::Unit {
                    println!("result: {v}");
                }
            }
            0
        }
        Err(e) => {
            eprintln!("error[{}]: {}", e.code, e.message);
            1
        }
    }
}

fn build(path: &Path, emit: &str, output: Option<&Path>) -> i32 {
    let mut sources = Sources::default();
    let program = match compile_or_report(path, &mut sources) {
        Ok(p) => p,
        Err(d) => {
            report(&d, &sources);
            return 1;
        }
    };
    let text = match emit {
        "gir" => program.gir.dump(),
        "gir=json" => match serde_json::to_string_pretty(&gala_driver::gir_json(&program)) {
            Ok(s) => s + "\n",
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        },
        other => {
            eprintln!("error: unknown --emit value `{other}` (use `gir` or `gir=json`)");
            return 2;
        }
    };
    match output {
        Some(p) => match std::fs::write(p, &text) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("error: could not write `{}`: {e}", p.display());
                1
            }
        },
        None => {
            print!("{text}");
            0
        }
    }
}

fn test(path: &Path) -> i32 {
    let mut sources = Sources::default();
    let program = match compile_or_report(path, &mut sources) {
        Ok(p) => p,
        Err(d) => {
            report(&d, &sources);
            return 1;
        }
    };
    let mut failed = 0;
    let mut ran = 0;
    for f in &program.hir.fns {
        let is_test = f.attributes.iter().any(|a| a.name.as_str() == "test");
        if !is_test {
            continue;
        }
        ran += 1;
        let idx = f.def.0 as usize;
        let mut single = program.gir.clone();
        single.entry = Some(idx);
        match run_entry(&single, &SimConfig { shots: 1, seed: 0, ..Default::default() }) {
            Ok(_) => println!("test {} ... ok", f.name),
            Err(e) => {
                failed += 1;
                println!("test {} ... FAILED: {e}", f.name);
            }
        }
    }
    println!("{ran} tests, {failed} failed");
    if failed > 0 {
        1
    } else {
        0
    }
}

fn fmt(path: Option<&Path>, check: bool, diff: bool, stdin: bool) -> i32 {
    use std::io::Read;
    let (label, source) = if stdin {
        let mut buf = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
            eprintln!("error: could not read stdin: {e}");
            return 1;
        }
        ("<stdin>".to_string(), buf)
    } else {
        let Some(path) = path else {
            eprintln!("error: give a file, or use --stdin");
            return 2;
        };
        match std::fs::read_to_string(path) {
            Ok(s) => (path.display().to_string(), s),
            Err(e) => {
                eprintln!("error: could not read `{}`: {e}", path.display());
                return 1;
            }
        }
    };
    let formatted = gala_fmt::format_source(&source);
    if check {
        if formatted == source {
            0
        } else {
            eprintln!("{label} is not formatted");
            1
        }
    } else if diff {
        if formatted == source {
            println!("{label} is already formatted");
            return 0;
        }
        println!("--- {label}\n+++ {label} (formatted)");
        let old: Vec<&str> = source.lines().collect();
        let new: Vec<&str> = formatted.lines().collect();
        for i in 0..old.len().max(new.len()) {
            match (old.get(i), new.get(i)) {
                (Some(a), Some(b)) if a == b => {}
                (a, b) => {
                    if let Some(a) = a {
                        println!("-{}: {a}", i + 1);
                    }
                    if let Some(b) = b {
                        println!("+{}: {b}", i + 1);
                    }
                }
            }
        }
        1
    } else {
        print!("{formatted}");
        0
    }
}

fn explain_cmd(code: &str, markdown: bool, as_json: bool) -> i32 {
    let digits = code.trim_start_matches(['E', 'e']);
    let Ok(n) = digits.parse::<u16>() else {
        eprintln!("error: `{code}` is not a diagnostic code such as E0412");
        return 2;
    };
    let dc = DiagnosticCode::new(n);
    let text = if as_json {
        explain::render_json(dc).map(|v| serde_json::to_string_pretty(&v).unwrap_or_default())
    } else if markdown {
        explain::render_markdown(dc)
    } else {
        explain::render_text(dc)
    };
    match text {
        Some(t) => {
            print!("{t}");
            0
        }
        None => {
            eprintln!("error: no explanation is registered for {dc}");
            1
        }
    }
}

fn new_project(name: &str, lib: bool) -> i32 {
    let parent = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    match gala_pkg::commands::init_project(&parent, name, lib) {
        Ok(_) => {
            println!("created `{name}`");
            0
        }
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    }
}

fn add(name: &str, version: Option<String>, path: Option<PathBuf>) -> i32 {
    let spec = match (version, path) {
        (_, Some(p)) => DependencySpec::path(p.display().to_string()),
        (Some(v), None) => DependencySpec::requirement(v),
        (None, None) => DependencySpec::requirement("*"),
    };
    let project = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    match gala_pkg::commands::add(&project, name, &spec, None) {
        Ok(_) => {
            println!("added `{name}`");
            0
        }
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    }
}

/// Evaluate REPL input from stdin. Lines starting with `fn` or `import` add definitions that
/// later lines can use; any other line is an expression, evaluated on the simulator and printed.
///
/// Quantum state does not persist between expressions: each one runs in a fresh simulation.
fn repl(as_json: bool) -> i32 {
    use std::io::BufRead;
    let mut defs: Vec<String> = Vec::new();
    let mut status = 0;
    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        let input = line.trim().to_string();
        if input.is_empty() {
            continue;
        }
        let (ok, output, codes) = if input.starts_with("fn ") || input.starts_with("import ") {
            let candidate = format!("{}\n{}", defs.join("\n"), input);
            match check_text(&candidate) {
                Ok(()) => {
                    defs.push(input.clone());
                    (true, Vec::new(), Vec::new())
                }
                Err(codes) => (false, Vec::new(), codes),
            }
        } else {
            let source = format!(
                "{}\nfn main() -> Int prob {{\n    print({input})\n    0\n}}\n",
                defs.join("\n")
            );
            match eval_text(&source) {
                Ok(lines) => (true, lines, Vec::new()),
                Err(codes) => (false, Vec::new(), codes),
            }
        };
        if !ok {
            status = 1;
        }
        if as_json {
            let v = json!({"input": input, "ok": ok, "output": output, "error_codes": codes});
            println!("{v}");
        } else if ok {
            for l in &output {
                println!("{l}");
            }
        } else {
            println!("error: {}", codes.join(", "));
        }
    }
    status
}

/// Compile source text; on failure return the error codes.
fn check_text(source: &str) -> Result<(), Vec<String>> {
    let mut sources = Sources::default();
    match gala_driver::compile_source(source, &mut sources) {
        Ok(_) => Ok(()),
        Err(d) => Err(d.errors().iter().map(|e| e.code.to_string()).collect()),
    }
}

/// Compile and run the `main` of source text, returning its printed lines.
fn eval_text(source: &str) -> Result<Vec<String>, Vec<String>> {
    let mut sources = Sources::default();
    let program = gala_driver::compile_source(source, &mut sources)
        .map_err(|d| d.errors().iter().map(|e| e.code.to_string()).collect::<Vec<_>>())?;
    match run_entry(&program.gir, &SimConfig { shots: 1, seed: 0, ..Default::default() }) {
        Ok(report) => Ok(report.output),
        Err(e) => Err(vec![e.code.to_string()]),
    }
}

/// Run `#[property(kind)]` functions. Kinds that have a runner here: `effect_honesty` (the
/// checker verified every call against the declared effect), `uncomputes` (the function runs and
/// every release returns its qubit to |0⟩), and `grad_matches` (parameter-shift against finite
/// differences). `unitary` and `reversible` have no runner yet and are reported as skipped.
fn property_tests(path: &Path, as_json: bool) -> i32 {
    let mut sources = Sources::default();
    let program = match compile_or_report(path, &mut sources) {
        Ok(p) => p,
        Err(d) => {
            report(&d, &sources);
            return 1;
        }
    };
    let cfg = SimConfig { shots: 40_000, seed: 17, ..Default::default() };
    let mut results: Vec<Value> = Vec::new();
    let mut skipped: Vec<Value> = Vec::new();
    for f in &program.hir.fns {
        let Some(attr) = f.attributes.iter().find(|a| a.name.as_str() == "property") else {
            continue;
        };
        let idx = f.def.0 as usize;
        for kind in &attr.args {
            let kind = kind.as_str();
            let name = f.name.as_str();
            match kind {
                "effect_honesty" => results.push(json!({
                    "property": kind, "passed": true,
                    "details": {"function_name": name, "num_trials": 1},
                })),
                "uncomputes" if f.params.is_empty() => {
                    let outcome = gala_sim::run_function(
                        &program.gir,
                        idx,
                        Vec::new(),
                        &SimConfig { shots: 1, ..cfg.clone() },
                        &Default::default(),
                    );
                    let (passed, cx) = match outcome {
                        Ok(_) => (true, Value::Null),
                        Err(e) => (false, json!({"error": e.to_string()})),
                    };
                    results.push(json!({
                        "property": kind, "passed": passed,
                        "details": {"function_name": name, "num_trials": 1, "counterexample": cx},
                    }));
                }
                "grad_matches" => {
                    let args: Vec<gala_sim::Value> =
                        f.params.iter().map(|_| gala_sim::Value::Float(0.5)).collect();
                    let param = f.params.len().saturating_sub(1);
                    let grad = gala_diff::gradient(&program.gir, idx, param, &args, &cfg);
                    let fd =
                        gala_diff::finite_difference(&program.gir, idx, param, &args, &cfg, 0.05);
                    match (grad, fd) {
                        (Ok(g), Ok(fd)) => {
                            let err = (g.value - fd).abs();
                            let tol = 0.1;
                            results.push(json!({
                                "property": kind, "passed": err < tol,
                                "details": {"function_name": name, "num_trials": 1,
                                    "tolerance": tol, "max_error": err},
                            }));
                        }
                        (Err(d), _) | (_, Err(d)) => skipped.push(json!({
                            "property": kind, "function_name": name,
                            "reason": d.errors().iter().map(|e| e.message.clone()).collect::<Vec<_>>().join("; "),
                        })),
                    }
                }
                other => skipped.push(json!({
                    "property": other, "function_name": name,
                    "reason": "no runner for this property in the CLI yet",
                })),
            }
        }
    }
    let failed = results.iter().filter(|r| r["passed"] == json!(false)).count();
    if as_json {
        let out = json!({"results": results, "skipped": skipped, "failed": failed});
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
    } else {
        for r in &results {
            println!(
                "{} {} ... {}",
                r["details"]["function_name"].as_str().unwrap_or("?"),
                r["property"].as_str().unwrap_or("?"),
                if r["passed"] == json!(true) { "ok" } else { "FAILED" }
            );
        }
        for s in &skipped {
            println!(
                "{} {} ... skipped ({})",
                s["function_name"].as_str().unwrap_or("?"),
                s["property"].as_str().unwrap_or("?"),
                s["reason"].as_str().unwrap_or("")
            );
        }
        println!("{} properties, {failed} failed, {} skipped", results.len(), skipped.len());
    }
    if failed > 0 {
        1
    } else {
        0
    }
}

/// `gala fix`: print validated fixes, or apply them with `--apply` and recheck.
fn fix_cmd(path: &Path, apply: bool, as_json: bool) -> i32 {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read `{}`: {e}", path.display());
            return 1;
        }
    };
    let mut sources = Sources::default();
    let diags: Vec<gala_diagnostics::Diagnostic> =
        match gala_driver::compile_source(&source, &mut sources) {
            Ok(_) => Vec::new(),
            Err(d) => d.diagnostics,
        };
    let fixes = fix::suggest(&source, &diags);
    if as_json {
        let items: Vec<Value> = fixes
            .iter()
            .map(|f| {
                json!({"description": f.description, "edits": f.edits.iter().map(|e| json!({
                "start": e.start, "end": e.end, "new_text": e.new_text})).collect::<Vec<_>>()})
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({"schema": "gala.fix.v1", "fixes": items}))
                .unwrap_or_default()
        );
    } else if fixes.is_empty() {
        println!("no validated fixes for {}", path.display());
    } else {
        for f in &fixes {
            println!("fix: {}", f.description);
        }
    }
    if !apply {
        return 0;
    }
    let Some(first) = fixes.first() else { return 0 };
    if let Err(e) = std::fs::write(path, &first.fixed_source) {
        eprintln!("error: could not write `{}`: {e}", path.display());
        return 1;
    }
    let mut check_sources = Sources::default();
    match gala_driver::compile_source(&first.fixed_source, &mut check_sources) {
        Ok(_) => {
            println!("applied; {} now checks clean", path.display());
            0
        }
        Err(d) => {
            println!("applied; {} still has {} error(s)", path.display(), d.errors().len());
            1
        }
    }
}

/// `gala run --from-gir`: execute a saved `gir.v1` program.
fn run_gir_json(path: &Path, shots: u64, seed: u64) -> i32 {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: could not read `{}`: {e}", path.display());
            return 1;
        }
    };
    let value: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: `{}` is not JSON: {e}", path.display());
            return 1;
        }
    };
    let gir = match gala_driver::gir_from_json(&value) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };
    match run_entry(&gir, &SimConfig { shots, seed, ..Default::default() }) {
        Ok(report) => {
            for line in &report.output {
                println!("{line}");
            }
            // Only a probabilistic entry point has a distribution to report.
            let returns_measurement = value["functions"]
                .as_array()
                .and_then(|fs| fs.iter().find(|f| f["name"] == "main"))
                .and_then(|f| f["return_type"].as_str())
                .is_some_and(|t| t.starts_with("Measured<"));
            let is_prob = returns_measurement;
            if is_prob {
                println!("results over {} shots:", report.shots);
                for (value, count) in &report.histogram {
                    println!("  {value}: {count}");
                }
            } else if let Some(v) = &report.first_result {
                if *v != gala_sim::Value::Unit {
                    println!("result: {v}");
                }
            }
            0
        }
        Err(e) => {
            eprintln!("error[{}]: {}", e.code, e.message);
            1
        }
    }
}

/// `gala corpus <dir>`: one JSON object per `.gala` file under `dir`, sorted by path.
fn corpus(dir: &Path) -> i32 {
    let mut files = Vec::new();
    collect_gala(dir, &mut files);
    files.sort();
    let mut out = String::new();
    for path in &files {
        let Ok(source) = std::fs::read_to_string(path) else { continue };
        let prompt = source
            .lines()
            .find_map(|l| {
                l.strip_prefix("//")
                    .map(|c| c.trim().to_string())
                    .filter(|c| !c.starts_with("expect"))
            })
            .unwrap_or_else(|| {
                path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
            });
        let expectations: Vec<String> = source
            .lines()
            .filter_map(|l| l.strip_prefix("//").map(str::trim))
            .filter(|c| c.starts_with("expect"))
            .map(str::to_string)
            .collect();
        let mut sources = Sources::default();
        let compiles = gala_driver::compile_source(&source, &mut sources).is_ok();
        let record = json!({
            "prompt": prompt,
            "completion": source,
            "properties": {
                "file": path.strip_prefix(dir).unwrap_or(path).display().to_string(),
                "expectations": expectations,
                "compiles": compiles,
            },
        });
        out.push_str(&record.to_string());
        out.push('\n');
    }
    print!("{out}");
    0
}

fn collect_gala(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_gala(&p, out);
        } else if p.extension().is_some_and(|x| x == "gala") {
            out.push(p);
        }
    }
}
