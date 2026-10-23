//! Conformance suite: every `.gala` file under `tests/conformance/` states its expected outcome in
//! its first comment lines:
//!
//! - `// expect: ok`            compiles with no errors
//! - `// expect: E0412`         compiles to an error with this code (may list several, comma-separated)
//! - `// expect-output: text`   compiles, runs, and the first shot prints exactly this text
//!   (`// expect-output:` lines are joined by newlines)
//!
//! A file without an expectation is a failure of the suite, so every case says what it means.

use gala_driver::{compile_file, Sources};
use std::fs;
use std::path::{Path, PathBuf};

fn corpus_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/conformance")
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();
    for p in paths {
        if p.is_dir() {
            collect(&p, out);
        } else if p.extension().is_some_and(|e| e == "gala") {
            out.push(p);
        }
    }
}

#[derive(Debug)]
enum Expect {
    Ok,
    Codes(Vec<String>),
    Output(String),
    /// Compiles, then fails at run time with this code.
    RunError(String),
}

fn expectation(text: &str) -> Option<Expect> {
    let mut output: Vec<String> = Vec::new();
    let mut codes: Option<Vec<String>> = None;
    let mut ok = false;
    let mut run_error: Option<String> = None;
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("//") else { continue };
        let rest = rest.trim();
        if let Some(v) = rest.strip_prefix("expect-output:") {
            output.push(v.trim_start().to_string());
        } else if let Some(v) = rest.strip_prefix("expect-run-error:") {
            run_error = Some(v.trim().to_string());
        } else if let Some(v) = rest.strip_prefix("expect:") {
            let v = v.trim();
            if v == "ok" {
                ok = true;
            } else {
                codes = Some(v.split(',').map(|c| c.trim().to_string()).collect());
            }
        }
    }
    if let Some(code) = run_error {
        Some(Expect::RunError(code))
    } else if !output.is_empty() {
        Some(Expect::Output(output.join("\n")))
    } else if let Some(c) = codes {
        Some(Expect::Codes(c))
    } else if ok {
        Some(Expect::Ok)
    } else {
        None
    }
}

/// Run one case and return a failure description, or `None` on success.
fn check_case(path: &Path) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    let Some(expect) = expectation(&text) else {
        return Some(format!("{}: no `// expect:` header", path.display()));
    };
    let mut sources = Sources::default();
    let result = compile_file(path, &mut sources);
    match (expect, result) {
        (Expect::Ok, Ok(_)) => None,
        (Expect::Ok, Err(d)) => {
            let codes: Vec<String> = d.errors().iter().map(|e| e.code.to_string()).collect();
            Some(format!("{}: expected ok, got {codes:?}", path.display()))
        }
        (Expect::Codes(want), Ok(_)) => {
            Some(format!("{}: expected {want:?}, but it compiled", path.display()))
        }
        (Expect::Codes(want), Err(d)) => {
            let got: Vec<String> = d.errors().iter().map(|e| e.code.to_string()).collect();
            if want.iter().all(|w| got.contains(w)) {
                None
            } else {
                Some(format!("{}: expected {want:?}, got {got:?}", path.display()))
            }
        }
        (Expect::Output(want), Ok(program)) => {
            let report = match gala_sim::run_entry(
                &program.gir,
                &gala_sim::SimConfig { shots: 1, seed: 1, ..Default::default() },
            ) {
                Ok(r) => r,
                Err(e) => return Some(format!("{}: run failed: {e}", path.display())),
            };
            let got = report.output.join("\n");
            if got == want {
                None
            } else {
                Some(format!("{}: expected output {want:?}, got {got:?}", path.display()))
            }
        }
        (Expect::Output(_), Err(d)) => {
            let codes: Vec<String> = d.errors().iter().map(|e| e.code.to_string()).collect();
            Some(format!("{}: expected output, got errors {codes:?}", path.display()))
        }
        (Expect::RunError(want), Ok(program)) => {
            match gala_sim::run_entry(
                &program.gir,
                &gala_sim::SimConfig { shots: 1, seed: 1, ..Default::default() },
            ) {
                Err(e) if e.code.to_string() == want => None,
                Err(e) => {
                    Some(format!("{}: expected run error {want}, got {}", path.display(), e.code))
                }
                Ok(_) => Some(format!("{}: expected run error {want}, but it ran", path.display())),
            }
        }
        (Expect::RunError(want), Err(d)) => {
            let codes: Vec<String> = d.errors().iter().map(|e| e.code.to_string()).collect();
            Some(format!(
                "{}: expected run error {want}, got compile errors {codes:?}",
                path.display()
            ))
        }
    }
}

#[test]
fn conformance_suite_passes() {
    let mut files = Vec::new();
    collect(&corpus_root(), &mut files);
    assert!(files.len() >= 20, "expected a real corpus, found {} files", files.len());
    let failures: Vec<String> = files.iter().filter_map(|p| check_case(p)).collect();
    assert!(
        failures.is_empty(),
        "{} of {} conformance cases failed:\n{}",
        failures.len(),
        files.len(),
        failures.join("\n")
    );
}

#[test]
fn every_case_declares_an_expectation() {
    let mut files = Vec::new();
    collect(&corpus_root(), &mut files);
    for p in &files {
        let text = fs::read_to_string(p).unwrap_or_default();
        assert!(expectation(&text).is_some(), "{} has no expectation header", p.display());
    }
}
