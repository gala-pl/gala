//! Classical codegen: GIR to native code, through Rust.
//!
//! The classical part of a Gala program (arithmetic, comparisons, tuples, vectors, calls, branches,
//! loops, and printing) is emitted as a self-contained Rust program over a small dynamic value
//! type. The value semantics match the simulator's interpreter, so the native program and
//! `gala run` agree on output. `rustc` compiles the result to an executable.
//!
//! Programs with quantum operations are rejected: those need the simulator or QIR.

use gala_diagnostics::{codes, Diagnostic, Diagnostics};
use gala_gir::{Const, Gir, Inst, Term};
use std::fmt::Write;
use std::path::Path;

/// The runtime every generated program starts with.
const PRELUDE: &str = r#"#![allow(unused, non_snake_case, unreachable_code)]
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
enum V {
    Unit,
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Tuple(Vec<V>),
    Vec(Vec<V>),
}

impl V {
    fn truthy(&self) -> bool {
        match self {
            V::Bool(b) => *b,
            other => panic!("a condition cannot be {:?}", other),
        }
    }
    fn as_f64(&self) -> f64 {
        match self {
            V::Int(i) => *i as f64,
            V::Float(f) => *f,
            other => panic!("expected a number, found {:?}", other),
        }
    }
    fn as_index(&self) -> usize {
        match self {
            V::Int(i) if *i >= 0 => *i as usize,
            other => panic!("bad index {:?}", other),
        }
    }
}

impl fmt::Display for V {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            V::Unit => write!(f, "()"),
            V::Int(v) => write!(f, "{}", v),
            V::Float(v) => write!(f, "{}", v),
            V::Bool(v) => write!(f, "{}", v),
            V::Str(s) => write!(f, "{}", s),
            V::Tuple(items) => {
                let parts: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                write!(f, "({})", parts.join(", "))
            }
            V::Vec(items) => {
                let parts: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", parts.join(", "))
            }
        }
    }
}

fn print_line(values: &[V]) {
    let parts: Vec<String> = values.iter().map(|v| v.to_string()).collect();
    println!("{}", parts.join(" "));
}

fn index(base: &V, i: &V) -> V {
    let i = i.as_index();
    match base {
        V::Vec(items) | V::Tuple(items) => items[i].clone(),
        other => panic!("cannot index {:?}", other),
    }
}

fn length(v: &V) -> V {
    match v {
        V::Vec(items) | V::Tuple(items) => V::Int(items.len() as i64),
        other => panic!("len of {:?}", other),
    }
}

fn tuple_get(v: &V, i: usize) -> V {
    match v {
        V::Tuple(items) => items[i].clone(),
        other => panic!("destructuring {:?}", other),
    }
}

fn both_int(a: &V, b: &V) -> Option<(i64, i64)> {
    match (a, b) {
        (V::Int(x), V::Int(y)) => Some((*x, *y)),
        _ => None,
    }
}

fn bin_add(a: &V, b: &V) -> V {
    match both_int(a, b) {
        Some((x, y)) => V::Int(x.wrapping_add(y)),
        None => V::Float(a.as_f64() + b.as_f64()),
    }
}
fn bin_sub(a: &V, b: &V) -> V {
    match both_int(a, b) {
        Some((x, y)) => V::Int(x.wrapping_sub(y)),
        None => V::Float(a.as_f64() - b.as_f64()),
    }
}
fn bin_mul(a: &V, b: &V) -> V {
    match both_int(a, b) {
        Some((x, y)) => V::Int(x.wrapping_mul(y)),
        None => V::Float(a.as_f64() * b.as_f64()),
    }
}
fn bin_div(a: &V, b: &V) -> V {
    match both_int(a, b) {
        Some((x, y)) => {
            if y == 0 {
                panic!("division by zero");
            }
            V::Int(x.wrapping_div(y))
        }
        None => V::Float(a.as_f64() / b.as_f64()),
    }
}
fn bin_mod(a: &V, b: &V) -> V {
    match both_int(a, b) {
        Some((x, y)) => {
            if y == 0 {
                panic!("remainder by zero");
            }
            V::Int(x.wrapping_rem(y))
        }
        None => panic!("`%` needs integers"),
    }
}
fn bin_eq(a: &V, b: &V) -> V {
    match both_int(a, b) {
        Some((x, y)) => V::Bool(x == y),
        None => match (a, b) {
            (V::Int(_) | V::Float(_), V::Int(_) | V::Float(_)) => V::Bool(a.as_f64() == b.as_f64()),
            _ => V::Bool(a == b),
        },
    }
}
fn bin_ne(a: &V, b: &V) -> V {
    match bin_eq(a, b) {
        V::Bool(x) => V::Bool(!x),
        _ => unreachable!(),
    }
}
fn bin_lt(a: &V, b: &V) -> V {
    match both_int(a, b) {
        Some((x, y)) => V::Bool(x < y),
        None => V::Bool(a.as_f64() < b.as_f64()),
    }
}
fn bin_le(a: &V, b: &V) -> V {
    match both_int(a, b) {
        Some((x, y)) => V::Bool(x <= y),
        None => V::Bool(a.as_f64() <= b.as_f64()),
    }
}
fn bin_gt(a: &V, b: &V) -> V {
    match both_int(a, b) {
        Some((x, y)) => V::Bool(x > y),
        None => V::Bool(a.as_f64() > b.as_f64()),
    }
}
fn bin_ge(a: &V, b: &V) -> V {
    match both_int(a, b) {
        Some((x, y)) => V::Bool(x >= y),
        None => V::Bool(a.as_f64() >= b.as_f64()),
    }
}
fn bin_and(a: &V, b: &V) -> V {
    V::Bool(a.truthy() && b.truthy())
}
fn bin_or(a: &V, b: &V) -> V {
    V::Bool(a.truthy() || b.truthy())
}
fn un_neg(a: &V) -> V {
    match a {
        V::Int(x) => V::Int(x.wrapping_neg()),
        V::Float(x) => V::Float(-x),
        other => panic!("cannot negate {:?}", other),
    }
}
fn un_not(a: &V) -> V {
    V::Bool(!a.truthy())
}
"#;

/// Emit a complete Rust program for the classical part of `gir`.
///
/// Returns an error when the program contains a quantum operation, or when it has no `main`.
pub fn emit_rust(gir: &Gir) -> Result<String, Diagnostics> {
    let entry = gir.entry.ok_or_else(|| error("the program has no `main` function"))?;
    let mut out = String::from(PRELUDE);
    for (i, f) in gir.funcs.iter().enumerate() {
        if f.blocks.is_empty() {
            continue;
        }
        emit_function(&mut out, gir, i)?;
    }
    let _ = writeln!(
        out,
        "fn main() {{\n    let r = f{entry}(Vec::new());\n    if r != V::Unit {{\n        println!(\"result: {{}}\", r);\n    }}\n}}"
    );
    Ok(out)
}

fn error(msg: impl Into<String>) -> Diagnostics {
    let mut d = Diagnostics::new();
    d.push(Diagnostic::error(codes::UNSUPPORTED_FEATURE, msg));
    d
}

fn emit_function(out: &mut String, gir: &Gir, index: usize) -> Result<(), Diagnostics> {
    let f = &gir.funcs[index];
    let _ = writeln!(out, "// {} (effect {})", f.name, f.effect);
    let _ = writeln!(out, "fn f{index}(args: Vec<V>) -> V {{");
    let _ = writeln!(out, "    let mut s: Vec<V> = vec![V::Unit; {}];", f.slot_count.max(1));
    for (k, p) in f.params.iter().enumerate() {
        let _ = writeln!(out, "    s[{p}] = args[{k}].clone();");
    }
    let _ = writeln!(out, "    let mut b: usize = 0;");
    let _ = writeln!(out, "    loop {{");
    let _ = writeln!(out, "        match b {{");
    for (bi, block) in f.blocks.iter().enumerate() {
        let _ = writeln!(out, "            {bi} => {{");
        for inst in &block.insts {
            emit_inst(out, index, inst)?;
        }
        emit_term(out, &block.term);
        let _ = writeln!(out, "            }}");
    }
    let _ = writeln!(out, "            _ => unreachable!(),");
    let _ = writeln!(out, "        }}");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out, "}}");
    Ok(())
}

fn emit_term(out: &mut String, t: &Term) {
    match t {
        Term::Unreachable => {
            let _ = writeln!(out, "                return V::Unit;");
        }
        Term::Jump(target) => {
            let _ = writeln!(out, "                b = {target}; continue;");
        }
        Term::Branch { cond, then_block, else_block } => {
            let _ = writeln!(
                out,
                "                b = if s[{cond}].truthy() {{ {then_block} }} else {{ {else_block} }}; continue;"
            );
        }
        Term::Return(Some(v)) => {
            let _ = writeln!(out, "                return s[{v}].clone();");
        }
        Term::Return(None) => {
            let _ = writeln!(out, "                return V::Unit;");
        }
    }
}

fn emit_inst(out: &mut String, _func: usize, inst: &Inst) -> Result<(), Diagnostics> {
    let line = |out: &mut String, s: String| {
        let _ = writeln!(out, "                {s}");
    };
    match inst {
        Inst::Const { dst, value } => {
            let v = match value {
                Const::Unit => "V::Unit".to_string(),
                Const::Int(v) => format!("V::Int({v}i64)"),
                Const::Float(v) => format!("V::Float({})", float_literal(*v)),
                Const::Bool(v) => format!("V::Bool({v})"),
                Const::Str(s) => format!("V::Str(String::from({s:?}))"),
                Const::Complex(..) | Const::Backend { .. } => {
                    return Err(error("this constant has no classical native form"));
                }
            };
            line(out, format!("s[{dst}] = {v};"));
        }
        Inst::Copy { dst, src } => line(out, format!("s[{dst}] = s[{src}].clone();")),
        Inst::Bin { dst, op, a, b } => {
            let name = bin_fn(*op)?;
            line(out, format!("s[{dst}] = {name}(&s[{a}], &s[{b}]);"));
        }
        Inst::Un { dst, op, src } => {
            let name = match op {
                gala_ast::UnOp::Neg => "un_neg",
                gala_ast::UnOp::Not => "un_not",
            };
            line(out, format!("s[{dst}] = {name}(&s[{src}]);"));
        }
        Inst::Tuple { dst, items } => {
            let vals: Vec<String> = items.iter().map(|i| format!("s[{i}].clone()")).collect();
            line(out, format!("s[{dst}] = V::Tuple(vec![{}]);", vals.join(", ")));
        }
        Inst::VecLit { dst, items } => {
            let vals: Vec<String> = items.iter().map(|i| format!("s[{i}].clone()")).collect();
            line(out, format!("s[{dst}] = V::Vec(vec![{}]);", vals.join(", ")));
        }
        Inst::Index { dst, base, index } => {
            line(out, format!("s[{dst}] = index(&s[{base}], &s[{index}]);"));
        }
        Inst::Len { dst, src } => line(out, format!("s[{dst}] = length(&s[{src}]);")),
        Inst::TupleGet { dst, src, i } => {
            line(out, format!("s[{dst}] = tuple_get(&s[{src}], {i});"))
        }
        Inst::Call { dsts, func, args } => {
            let vals: Vec<String> = args.iter().map(|a| format!("s[{a}].clone()")).collect();
            match dsts.first() {
                Some(d) => line(out, format!("s[{d}] = f{func}(vec![{}]);", vals.join(", "))),
                None => line(out, format!("f{func}(vec![{}]);", vals.join(", "))),
            }
        }
        Inst::Print { args } => {
            let vals: Vec<String> = args.iter().map(|a| format!("s[{a}].clone()")).collect();
            line(out, format!("print_line(&[{}]);", vals.join(", ")));
        }
        Inst::AllocQubit { .. }
        | Inst::AllocRegister { .. }
        | Inst::Gate { .. }
        | Inst::Measure { .. } => {
            return Err(error("quantum operations cannot be compiled to classical native code; use the simulator or QIR"));
        }
        Inst::Release { .. } => {
            return Err(error("releasing a qubit cannot be compiled to classical native code"));
        }
        Inst::Run { .. } => {
            return Err(error(
                "`run` cannot be compiled to classical native code; use the simulator",
            ));
        }
    }
    Ok(())
}

fn bin_fn(op: gala_ast::BinOp) -> Result<&'static str, Diagnostics> {
    use gala_ast::BinOp::*;
    Ok(match op {
        Add => "bin_add",
        Sub => "bin_sub",
        Mul => "bin_mul",
        Div => "bin_div",
        Mod => "bin_mod",
        Eq => "bin_eq",
        Ne => "bin_ne",
        Lt => "bin_lt",
        Le => "bin_le",
        Gt => "bin_gt",
        Ge => "bin_ge",
        And => "bin_and",
        Or => "bin_or",
        Range => return Err(error("a range is not a value in classical native code")),
    })
}

/// A Rust float literal that reads back to the same value.
fn float_literal(v: f64) -> String {
    if v.is_nan() {
        "f64::NAN".to_string()
    } else if v.is_infinite() {
        if v > 0.0 {
            "f64::INFINITY".to_string()
        } else {
            "f64::NEG_INFINITY".to_string()
        }
    } else {
        format!("{v:?}")
    }
}

/// Compile the classical program to a native executable at `out`, using `rustc` (or `$RUSTC`).
///
/// The Rust source is written next to the executable with a `.rs` extension, so the build can be
/// inspected.
pub fn compile_native(gir: &Gir, out: &Path) -> Result<(), String> {
    let source = emit_rust(gir).map_err(|d| {
        d.diagnostics.iter().map(|x| x.message.clone()).collect::<Vec<_>>().join("; ")
    })?;
    let src = out.with_extension("rs");
    std::fs::write(&src, &source)
        .map_err(|e| format!("could not write `{}`: {e}", src.display()))?;
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let status = std::process::Command::new(rustc)
        .args(["-O", "--edition", "2021", "-o"])
        .arg(out)
        .arg(&src)
        .status()
        .map_err(|e| format!("could not run rustc: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("rustc failed to compile the generated program".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gala_parser::parse_source;
    use gala_span::FileId;

    fn gir_of(src: &str) -> Gir {
        let out = parse_source(FileId(0), src);
        let (pkg, _) = gala_hir::lower_file(FileId(0), out.items);
        let (info, _) = gala_types::check_package(&pkg);
        gala_gir::lower_package(&pkg, &info).expect("lowers")
    }

    #[test]
    fn classical_program_emits_rust_with_every_function() {
        let gir = gir_of("fn add(a: Int, b: Int) -> Int pure {\n    a + b\n}\nfn main() -> Int {\n    add(1, 2)\n}\n");
        let text = emit_rust(&gir).unwrap();
        assert!(text.contains("fn f0("));
        assert!(text.contains("fn f1("));
        assert!(text.contains("fn main()"));
    }

    #[test]
    fn quantum_operations_are_rejected() {
        let gir =
            gir_of("fn main() -> Measured<Bool> prob {\n    let q = qubit()\n    measure(q)\n}\n");
        let err = emit_rust(&gir).unwrap_err();
        assert!(err.errors()[0].message.contains("quantum"));
    }

    #[test]
    fn float_literals_round_trip() {
        assert_eq!(float_literal(0.5), "0.5");
        assert!(float_literal(1e-9).parse::<f64>().is_ok());
    }
}

#[cfg(test)]
mod native_tests {
    use super::*;
    use gala_parser::parse_source;
    use gala_span::FileId;

    fn gir_of(src: &str) -> Gir {
        let out = parse_source(FileId(0), src);
        let (pkg, _) = gala_hir::lower_file(FileId(0), out.items);
        let (info, _) = gala_types::check_package(&pkg);
        gala_gir::lower_package(&pkg, &info).expect("lowers")
    }

    /// The native program prints exactly what the simulator prints.
    #[test]
    fn native_output_matches_the_simulator() {
        let src = "fn fib(n: Int) -> Int pure {\n    if n < 2 {\n        n\n    } else {\n        fib(n - 1) + fib(n - 2)\n    }\n}\nfn main() -> Int {\n    let mut total = 0\n    for i in 0..10 {\n        total += fib(i)\n    }\n    print(total)\n    print(\"done\", 1.5, true)\n    total\n}\n";
        let gir = gir_of(src);
        let dir = std::env::temp_dir().join(format!("gala-native-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("prog");
        compile_native(&gir, &exe).expect("compiles with rustc");
        let run = std::process::Command::new(&exe).output().expect("runs");
        let native = String::from_utf8_lossy(&run.stdout).to_string();
        let sim = gala_sim::run_entry(&gir, &gala_sim::SimConfig::default()).unwrap();
        let mut expected = sim.output.join("\n");
        expected.push('\n');
        expected.push_str(&format!("result: {}\n", sim.first_result.unwrap()));
        assert_eq!(native, expected);
    }

    #[test]
    fn native_loop_runs_and_matches_the_interpreter() {
        // A long classical loop: the native build should be much faster than interpreting it.
        let src = "fn main() -> Int {\n    let mut s = 0\n    for i in 0..3000000 {\n        s += i % 7\n    }\n    print(s)\n    0\n}\n";
        let gir = gir_of(src);
        let dir = std::env::temp_dir().join(format!("gala-native-loop-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("loop");
        compile_native(&gir, &exe).expect("compiles");
        let t0 = std::time::Instant::now();
        let native = std::process::Command::new(&exe).output().expect("runs");
        let native_time = t0.elapsed();
        let t1 = std::time::Instant::now();
        let sim = gala_sim::run_entry(&gir, &gala_sim::SimConfig::default()).unwrap();
        let interp_time = t1.elapsed();
        let mut expected = sim.output.join("\n");
        expected.push_str(&format!("\nresult: {}", sim.first_result.unwrap()));
        assert_eq!(String::from_utf8_lossy(&native.stdout).trim(), expected);
        eprintln!("native {:?} vs interpreter {:?}", native_time, interp_time);
        assert!(
            native_time <= interp_time,
            "native {native_time:?} should not be slower than interpreting {interp_time:?}"
        );
    }
}
