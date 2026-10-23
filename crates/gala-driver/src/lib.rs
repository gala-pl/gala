//! Compilation driver: loads a program, runs every pass in order, and collects diagnostics.
//!
//! Pass order: parse → HIR (name resolution) → types and effects → linearity → uncomputation →
//! GIR lowering → optimization. A pass whose errors would make later passes meaningless stops
//! the pipeline, but all the errors it found are returned.

pub mod query;

use gala_ast::Item;
use gala_diagnostics::{Diagnostic, Diagnostics};
use gala_gir::Gir;
use gala_hir::{lower_package, HirPackage, ModuleTable, SourceModule};
use gala_span::{FileId, SourceMap};
use gala_types::TypeInfo;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

/// Bundled standard-library modules: dotted path → source.
const BUNDLED: &[(&str, &str, &str)] = &[
    ("gala.gates", "library/std/gates.gala", include_str!("../../../library/std/gates.gala")),
    ("gala.core", "library/std/core.gala", include_str!("../../../library/std/core.gala")),
    (
        "gala.algorithms",
        "library/std/algorithms.gala",
        include_str!("../../../library/std/algorithms.gala"),
    ),
    ("gala.vqa", "library/std/vqa.gala", include_str!("../../../library/std/vqa.gala")),
];

/// A fully checked and lowered program.
#[derive(Debug, Clone)]
pub struct Program {
    /// Name-resolved program.
    pub hir: HirPackage,
    /// Types and effects.
    pub types: TypeInfo,
    /// Lowered, optimized IR.
    pub gir: Gir,
}

/// Source files loaded for a compilation, for rendering diagnostics.
#[derive(Debug, Default)]
pub struct Sources {
    /// Every file that was read or bundled.
    pub map: SourceMap,
}

/// Load a program from a file. If the file is inside a package (a directory with `gala.toml`),
/// modules under the package's `src/` directory are loaded on demand for `import`s.
pub fn load_file(path: &Path, sources: &mut Sources) -> Result<Vec<SourceModule>, Diagnostics> {
    let text = std::fs::read_to_string(path).map_err(|e| io_error(path, &e))?;
    let root_file = sources.map.add_file(path.to_path_buf(), text);
    let base = module_base(path);
    let mut modules =
        vec![SourceModule { path: String::new(), file: root_file, items: Vec::new() }];
    let mut diags = Diagnostics::new();
    let root_items = parse(root_file, &sources.map, &mut diags);
    modules[0].items = root_items;
    let mut loaded: BTreeSet<String> = BTreeSet::new();
    let mut queue: Vec<String> = imports_of(&modules[0].items);
    while let Some(module) = queue.pop() {
        if !loaded.insert(module.clone()) {
            continue;
        }
        if let Some(src) = bundled(&module) {
            let file = sources.map.add_file(PathBuf::from(src.0), src.1.to_string());
            let items = parse(file, &sources.map, &mut diags);
            queue.extend(imports_of(&items));
            modules.push(SourceModule { path: module, file, items });
            continue;
        }
        if module.starts_with("gala") {
            // Compiler builtins such as `gala_std.io` need no file.
            continue;
        }
        let file_path = module_file(&base, &module);
        match std::fs::read_to_string(&file_path) {
            Ok(text) => {
                let file = sources.map.add_file(file_path.clone(), text);
                let items = parse(file, &sources.map, &mut diags);
                queue.extend(imports_of(&items));
                modules.push(SourceModule { path: module, file, items });
            }
            Err(_) => {
                // Left unresolved here; HIR reports it at the import.
            }
        }
    }
    if diags.has_errors() {
        return Err(diags);
    }
    Ok(modules)
}

fn parse(file: FileId, map: &SourceMap, diags: &mut Diagnostics) -> Vec<Item> {
    let text = match map.get_content(file) {
        Some(t) => t,
        None => return Vec::new(),
    };
    let out = gala_parser::parse_source(file, &text);
    diags.extend(out.diagnostics);
    out.items
}

fn bundled(module: &str) -> Option<(&'static str, &'static str)> {
    BUNDLED.iter().find(|(m, _, _)| *m == module).map(|(_, path, src)| (*path, *src))
}

fn module_base(path: &Path) -> PathBuf {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    // Inside a package, module paths are relative to `src/`.
    if dir.file_name().is_some_and(|n| n == "src") {
        dir.to_path_buf()
    } else {
        let src = dir.join("src");
        if src.is_dir() {
            src
        } else {
            dir.to_path_buf()
        }
    }
}

fn module_file(base: &Path, module: &str) -> PathBuf {
    let mut p = base.to_path_buf();
    for seg in module.split('.') {
        p.push(seg);
    }
    p.set_extension("gala");
    p
}

fn imports_of(items: &[Item]) -> Vec<String> {
    let mut out = Vec::new();
    for item in items {
        if let Item::Import(imp) = item {
            let segs: Vec<&str> = imp.path.segments.iter().map(|s| s.ident.as_str()).collect();
            let module = if imp.items.is_some() || imp.glob {
                segs.join(".")
            } else {
                segs[..segs.len().saturating_sub(1)].join(".")
            };
            if !module.is_empty() {
                out.push(module);
            }
        }
    }
    out
}

fn io_error(path: &Path, e: &std::io::Error) -> Diagnostics {
    let mut d = Diagnostics::new();
    d.push(Diagnostic::error(
        gala_diagnostics::codes::RUNTIME_ERROR,
        format!("could not read `{}`: {e}", path.display()),
    ));
    d
}

/// Compile loaded modules through every pass.
pub fn compile_modules(modules: &[SourceModule]) -> Result<Program, Diagnostics> {
    let root =
        modules.iter().find(|m| m.path.is_empty()).map(|m| m.path.clone()).unwrap_or_default();
    let (hir, mut diags) = lower_package(modules, &root);
    if diags.has_errors() {
        return Err(diags);
    }
    let (types, td) = gala_types::check_package(&hir);
    diags.extend(td);
    if diags.has_errors() {
        return Err(diags);
    }
    let (_, ld) = gala_types::check_linearity(&hir, &types);
    diags.extend(ld);
    let (_, ud) = gala_uncompute::analyze(&hir, &types);
    diags.extend(ud);
    if diags.has_errors() {
        return Err(diags);
    }
    let gir = gala_gir::lower_package(&hir, &types)?;
    // Internal invariant: lowering must produce well-formed GIR (checked in debug builds).
    debug_assert!(
        gala_gir::verify(&gir).is_empty(),
        "GIR invariant violated: {:?}",
        gala_gir::verify(&gir)
    );
    let gir = gala_opt::optimize(&gir);
    debug_assert!(
        gala_gir::verify(&gir).is_empty(),
        "optimizer broke GIR: {:?}",
        gala_gir::verify(&gir)
    );
    Ok(Program { hir, types, gir })
}

/// Compile a single in-memory source file as the root module. Bundled standard-library imports
/// are still available.
pub fn compile_source(source: &str, sources: &mut Sources) -> Result<Program, Diagnostics> {
    let file = sources.map.add_file(PathBuf::from("<input>"), source.to_string());
    let mut diags = Diagnostics::new();
    let items = parse(file, &sources.map, &mut diags);
    if diags.has_errors() {
        return Err(diags);
    }
    let mut modules = vec![SourceModule { path: String::new(), file, items: items.clone() }];
    let mut loaded = BTreeSet::new();
    let mut queue = imports_of(&items);
    while let Some(m) = queue.pop() {
        if !loaded.insert(m.clone()) {
            continue;
        }
        if let Some((path, text)) = bundled(&m) {
            let f = sources.map.add_file(PathBuf::from(path), text.to_string());
            let its = parse(f, &sources.map, &mut diags);
            queue.extend(imports_of(&its));
            modules.push(SourceModule { path: m, file: f, items: its });
        }
    }
    if diags.has_errors() {
        return Err(diags);
    }
    compile_modules(&modules)
}

/// Compile a file from disk (with package-relative module loading).
pub fn compile_file(path: &Path, sources: &mut Sources) -> Result<Program, Diagnostics> {
    let mut modules = load_file(path, sources)?;
    // Bundled library modules that the program imports.
    let mut have: BTreeSet<String> = modules.iter().map(|m| m.path.clone()).collect();
    let mut queue: Vec<String> = modules.iter().flat_map(|m| imports_of(&m.items)).collect();
    while let Some(m) = queue.pop() {
        if have.contains(&m) {
            continue;
        }
        if let Some((p, text)) = bundled(&m) {
            let f = sources.map.add_file(PathBuf::from(p), text.to_string());
            let mut d = Diagnostics::new();
            let its = parse(f, &sources.map, &mut d);
            if d.has_errors() {
                return Err(d);
            }
            queue.extend(imports_of(&its));
            have.insert(m.clone());
            modules.push(SourceModule { path: m, file: f, items: its });
        }
    }
    compile_modules(&modules)
}

/// Map from module path to the table used by HIR (exposed for tools that build programs).
pub fn module_table(modules: Vec<SourceModule>) -> ModuleTable {
    ModuleTable::new(modules)
}

/// Names of all bundled standard-library modules.
pub fn bundled_modules() -> Vec<&'static str> {
    BUNDLED.iter().map(|(m, _, _)| *m).collect()
}

/// Per-function summary used by `gala check --json` and tooling.
pub fn function_effects(program: &Program) -> HashMap<String, String> {
    program
        .hir
        .fns
        .iter()
        .map(|f| (f.name.as_str().to_string(), program.types.effects[f.def.0 as usize].to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_bell_pair_end_to_end() {
        let mut s = Sources::default();
        let src = "fn main() -> Measured<Bool> prob {\n let a = qubit()\n let b = qubit()\n h(a)\n let (a, b) = cx(a, b)\n let r = measure(a)\n measure(b)\n}";
        let p = compile_source(src, &mut s).expect("bell pair compiles");
        assert!(p.gir.entry.is_some());
    }

    #[test]
    fn reports_linearity_error_and_stops() {
        let mut s = Sources::default();
        let err = compile_source("fn f() quantum { let q = qubit() }", &mut s).unwrap_err();
        assert!(err.errors().iter().any(|d| d.code.to_string() == "E0402"));
    }

    #[test]
    fn bundled_modules_are_listed() {
        assert!(bundled_modules().contains(&"gala.gates"));
    }

    #[test]
    fn unresolved_import_is_reported() {
        let mut s = Sources::default();
        let err = compile_source("import nope.thing\nfn main() -> Int { 1 }", &mut s).unwrap_err();
        assert!(err.errors().iter().any(|d| d.code.to_string() == "E0202"));
    }
}

/// The program as `gir.v1` JSON (see `tools/gala-mcp/src/schemas/gir.v1.json`): one entry per
/// function with its typed parameters, return type, effect, source span, and blocks in order.
pub fn gir_json(program: &Program) -> serde_json::Value {
    use gala_gir::{Inst, Term};
    use serde_json::{json, Value};

    let slot_list = |xs: &[u32]| -> Value { json!(xs) };
    let inst_json = |i: &Inst| -> Value {
        match i {
            Inst::Const { dst, value } => {
                json!({"op": "const", "dst": dst, "value": const_json(value)})
            }
            Inst::Copy { dst, src } => json!({"op": "copy", "dst": dst, "src": src}),
            Inst::Bin { dst, op, a, b } => {
                json!({"op": "bin", "dst": dst, "operator": bin_name(*op), "a": a, "b": b})
            }
            Inst::Un { dst, op, src } => {
                json!({"op": "un", "dst": dst, "operator": un_name(*op), "src": src})
            }
            Inst::Tuple { dst, items } => {
                json!({"op": "tuple", "dst": dst, "items": slot_list(items)})
            }
            Inst::VecLit { dst, items } => {
                json!({"op": "vec", "dst": dst, "items": slot_list(items)})
            }
            Inst::Index { dst, base, index } => {
                json!({"op": "index", "dst": dst, "base": base, "index": index})
            }
            Inst::Len { dst, src } => json!({"op": "len", "dst": dst, "src": src}),
            Inst::TupleGet { dst, src, i } => {
                json!({"op": "tuple_get", "dst": dst, "src": src, "index": i})
            }
            Inst::Call { dsts, func, args } => {
                json!({"op": "call", "dsts": slot_list(dsts), "func": func, "args": slot_list(args)})
            }
            Inst::AllocQubit { dst } => json!({"op": "alloc_qubit", "dst": dst}),
            Inst::AllocRegister { dst, n } => {
                json!({"op": "alloc_register", "dst": dst, "size": n})
            }
            Inst::Gate { gate, targets, angle, dsts } => json!({
                "op": "gate", "gate": gate.name(), "targets": slot_list(targets),
                "angle": angle, "dsts": slot_list(dsts)
            }),
            Inst::Measure { dst, target } => json!({"op": "measure", "dst": dst, "target": target}),
            Inst::Release { target } => json!({"op": "release", "target": target}),
            Inst::Print { args } => json!({"op": "print", "args": slot_list(args)}),
            Inst::Run { dst, func, shots, backend } => json!({
                "op": "run", "dst": dst, "func": func, "shots": shots, "backend": backend
            }),
        }
    };
    let term_json = |t: &Term| -> Value {
        match t {
            Term::Unreachable => json!({"op": "unreachable"}),
            Term::Jump(b) => json!({"op": "jump", "target": b}),
            Term::Branch { cond, then_block, else_block } => {
                json!({"op": "branch", "cond": cond, "then": then_block, "else": else_block})
            }
            Term::Return(v) => json!({"op": "return", "value": v}),
        }
    };

    let functions: Vec<Value> = program
        .gir
        .funcs
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let hir = program.hir.fns.get(i);
            let sig = program.types.sigs.get(i);
            let params: Vec<Value> = match (hir, sig) {
                (Some(h), Some(s)) => h
                    .params
                    .iter()
                    .zip(s.params.iter())
                    .map(|(p, t)| {
                        json!({
                            "name": p.name,
                            "type": t.to_string(),
                            "linearity": if t.is_linear() { "linear" } else { "unrestricted" },
                        })
                    })
                    .collect(),
                _ => Vec::new(),
            };
            let return_type = sig.map(|s| s.ret.to_string()).unwrap_or_else(|| "Unit".to_string());
            let span = hir.map(|h| h.span).unwrap_or_else(gala_span::Span::dummy);
            let blocks: Vec<Value> = f
                .blocks
                .iter()
                .enumerate()
                .map(|(bi, b)| {
                    json!({
                        "id": bi,
                        "insts": b.insts.iter().map(inst_json).collect::<Vec<_>>(),
                        "term": term_json(&b.term),
                    })
                })
                .collect();
            json!({
                "id": format!("fn{i}"),
                "name": f.name,
                "params": params,
                "return_type": return_type,
                "effect": f.effect.to_string(),
                "body": {"blocks": blocks, "slot_count": f.slot_count},
                "spans": {
                    "file_id": span.file_id.0,
                    "start": span.byte_span.start,
                    "end": span.byte_span.end,
                },
            })
        })
        .collect();

    let mut types = serde_json::Map::new();
    for t in &program.types.sigs {
        for ty in t.params.iter().chain(std::iter::once(&t.ret)) {
            let name = ty.to_string();
            let kind = match ty {
                gala_types::Ty::Qubit => "qubit",
                gala_types::Ty::Qubits(_) => "qubits",
                gala_types::Ty::Measured(_) => "measured",
                gala_types::Ty::Tuple(_) => "tuple",
                gala_types::Ty::Vec(_) => "array",
                _ => "primitive",
            };
            types.entry(name.clone()).or_insert_with(|| {
                json!({"kind": kind, "name": name, "linearity": if ty.is_linear() { "linear" } else { "unrestricted" }})
            });
        }
    }
    let effects: serde_json::Map<String, Value> = program
        .hir
        .fns
        .iter()
        .map(|f| {
            (
                f.name.as_str().to_string(),
                json!(program.types.effects[f.def.0 as usize].to_string()),
            )
        })
        .collect();

    json!({
        "version": 1,
        "functions": functions,
        "types": types,
        "effects": effects,
        "metadata": {"compiler_version": env!("CARGO_PKG_VERSION")},
    })
}

#[cfg(test)]
mod json_tests {
    use super::*;

    #[test]
    fn gir_json_has_schema_v1_keys_and_typed_params() {
        let mut s = Sources::default();
        let src =
            "fn bell(q: Qubit) -> Qubit quantum {\n    h(q)\n}\nfn main() -> Int {\n    0\n}\n";
        let p = compile_source(src, &mut s).expect("compiles");
        let v = gir_json(&p);
        assert_eq!(v["version"], 1);
        for key in ["functions", "types", "effects"] {
            assert!(v.get(key).is_some(), "missing {key}");
        }
        let f = &v["functions"][0];
        for key in ["id", "name", "params", "return_type", "effect", "body", "spans"] {
            assert!(f.get(key).is_some(), "function missing {key}");
        }
        assert_eq!(f["name"], "bell");
        assert_eq!(f["effect"], "quantum");
        assert_eq!(f["params"][0]["type"], "Qubit");
        assert_eq!(f["params"][0]["linearity"], "linear");
        assert_eq!(v["effects"]["main"], "pure");
    }
}

fn const_json(c: &gala_gir::Const) -> serde_json::Value {
    use gala_gir::Const;
    use serde_json::json;
    match c {
        Const::Unit => json!({"kind": "unit"}),
        Const::Int(v) => json!({"kind": "int", "value": v}),
        Const::Float(v) => json!({"kind": "float", "value": v}),
        Const::Bool(v) => json!({"kind": "bool", "value": v}),
        Const::Str(v) => json!({"kind": "str", "value": v}),
        Const::Complex(re, im) => json!({"kind": "complex", "re": re, "im": im}),
        Const::Backend { kind, noise } => {
            json!({"kind": "backend", "backend": kind, "noise": noise})
        }
    }
}

fn const_from_json(v: &serde_json::Value) -> Result<gala_gir::Const, String> {
    use gala_gir::Const;
    let f = |k: &str| v[k].as_f64().ok_or_else(|| format!("bad constant field {k}"));
    Ok(match v["kind"].as_str().unwrap_or("") {
        "unit" => Const::Unit,
        "int" => Const::Int(v["value"].as_i64().ok_or("bad int")?),
        "float" => Const::Float(f("value")?),
        "bool" => Const::Bool(v["value"].as_bool().ok_or("bad bool")?),
        "str" => Const::Str(v["value"].as_str().ok_or("bad str")?.to_string()),
        "complex" => Const::Complex(f("re")?, f("im")?),
        "backend" => Const::Backend {
            kind: v["backend"].as_str().ok_or("bad backend")?.to_string(),
            noise: f("noise")?,
        },
        other => return Err(format!("unknown constant kind `{other}`")),
    })
}

fn bin_name(op: gala_ast::BinOp) -> &'static str {
    use gala_ast::BinOp::*;
    match op {
        Add => "Add",
        Sub => "Sub",
        Mul => "Mul",
        Div => "Div",
        Mod => "Mod",
        Eq => "Eq",
        Ne => "Ne",
        Lt => "Lt",
        Le => "Le",
        Gt => "Gt",
        Ge => "Ge",
        And => "And",
        Or => "Or",
        Range => "Range",
    }
}

fn un_name(op: gala_ast::UnOp) -> &'static str {
    match op {
        gala_ast::UnOp::Neg => "Neg",
        gala_ast::UnOp::Not => "Not",
    }
}

fn bin_from_name(name: &str) -> Result<gala_ast::BinOp, String> {
    use gala_ast::BinOp::*;
    Ok(match name {
        "Add" => Add,
        "Sub" => Sub,
        "Mul" => Mul,
        "Div" => Div,
        "Mod" => Mod,
        "Eq" => Eq,
        "Ne" => Ne,
        "Lt" => Lt,
        "Le" => Le,
        "Gt" => Gt,
        "Ge" => Ge,
        "And" => And,
        "Or" => Or,
        "Range" => Range,
        other => return Err(format!("unknown operator `{other}`")),
    })
}

fn un_from_name(name: &str) -> Result<gala_ast::UnOp, String> {
    match name {
        "Neg" => Ok(gala_ast::UnOp::Neg),
        "Not" => Ok(gala_ast::UnOp::Not),
        other => Err(format!("unknown unary operator `{other}`")),
    }
}

fn gate_from_name(name: &str) -> Result<gala_gir::GateKind, String> {
    use gala_gir::GateKind::*;
    Ok(match name {
        "h" => H,
        "x" => X,
        "y" => Y,
        "z" => Z,
        "s" => S,
        "t" => T,
        "rx" => Rx,
        "ry" => Ry,
        "rz" => Rz,
        "cx" => Cx,
        "cz" => Cz,
        "swap" => Swap,
        "cp" => Cp,
        other => return Err(format!("unknown gate `{other}`")),
    })
}

fn slots_from_json(x: &serde_json::Value) -> Result<Vec<u32>, String> {
    x.as_array()
        .ok_or("expected a slot list")?
        .iter()
        .map(|n| n.as_u64().map(|n| n as u32).ok_or_else(|| "bad slot".to_string()))
        .collect()
}

fn slot_field(x: &serde_json::Value, k: &str) -> Result<u32, String> {
    x[k].as_u64().map(|n| n as u32).ok_or_else(|| format!("missing slot `{k}`"))
}

fn usize_field(x: &serde_json::Value, k: &str) -> Result<usize, String> {
    x[k].as_u64().map(|n| n as usize).ok_or_else(|| format!("missing `{k}`"))
}

fn inst_from_json(i: &serde_json::Value) -> Result<gala_gir::Inst, String> {
    use gala_gir::Inst;
    let dst = || slot_field(i, "dst");
    Ok(match i["op"].as_str().unwrap_or("") {
        "const" => Inst::Const { dst: dst()?, value: const_from_json(&i["value"])? },
        "copy" => Inst::Copy { dst: dst()?, src: slot_field(i, "src")? },
        "bin" => Inst::Bin {
            dst: dst()?,
            op: bin_from_name(i["operator"].as_str().unwrap_or(""))?,
            a: slot_field(i, "a")?,
            b: slot_field(i, "b")?,
        },
        "un" => Inst::Un {
            dst: dst()?,
            op: un_from_name(i["operator"].as_str().unwrap_or(""))?,
            src: slot_field(i, "src")?,
        },
        "tuple" => Inst::Tuple { dst: dst()?, items: slots_from_json(&i["items"])? },
        "vec" => Inst::VecLit { dst: dst()?, items: slots_from_json(&i["items"])? },
        "index" => Inst::Index {
            dst: dst()?,
            base: slot_field(i, "base")?,
            index: slot_field(i, "index")?,
        },
        "len" => Inst::Len { dst: dst()?, src: slot_field(i, "src")? },
        "tuple_get" => {
            Inst::TupleGet { dst: dst()?, src: slot_field(i, "src")?, i: usize_field(i, "index")? }
        }
        "call" => Inst::Call {
            dsts: slots_from_json(&i["dsts"])?,
            func: usize_field(i, "func")?,
            args: slots_from_json(&i["args"])?,
        },
        "alloc_qubit" => Inst::AllocQubit { dst: dst()? },
        "alloc_register" => Inst::AllocRegister { dst: dst()?, n: usize_field(i, "size")? },
        "gate" => Inst::Gate {
            gate: gate_from_name(i["gate"].as_str().unwrap_or(""))?,
            targets: slots_from_json(&i["targets"])?,
            angle: i["angle"].as_u64().map(|n| n as u32),
            dsts: slots_from_json(&i["dsts"])?,
        },
        "measure" => Inst::Measure { dst: dst()?, target: slot_field(i, "target")? },
        "release" => Inst::Release { target: slot_field(i, "target")? },
        "print" => Inst::Print { args: slots_from_json(&i["args"])? },
        "run" => Inst::Run {
            dst: slot_field(i, "dst")?,
            func: usize_field(i, "func")?,
            shots: slot_field(i, "shots")?,
            backend: slot_field(i, "backend")?,
        },
        other => return Err(format!("unknown instruction `{other}`")),
    })
}

fn term_from_json(t: &serde_json::Value) -> Result<gala_gir::Term, String> {
    use gala_gir::Term;
    Ok(match t["op"].as_str().unwrap_or("") {
        "unreachable" => Term::Unreachable,
        "jump" => Term::Jump(usize_field(t, "target")? as u32),
        "branch" => Term::Branch {
            cond: slot_field(t, "cond")?,
            then_block: usize_field(t, "then")? as u32,
            else_block: usize_field(t, "else")? as u32,
        },
        "return" => Term::Return(t["value"].as_u64().map(|n| n as u32)),
        other => return Err(format!("unknown terminator `{other}`")),
    })
}

/// Rebuild a GIR program from `gir.v1` JSON produced by [`gir_json`]. Types and spans are not
/// needed to run a program, so they are not restored.
pub fn gir_from_json(v: &serde_json::Value) -> Result<gala_gir::Gir, String> {
    use gala_gir::{Block, Gir, GirFunc};
    use gala_types::Effect;
    if v["version"].as_i64() != Some(1) {
        return Err("expected gir.v1 (\"version\": 1)".to_string());
    }
    let mut funcs = Vec::new();
    for f in v["functions"].as_array().ok_or("missing functions")? {
        let effect = match f["effect"].as_str().unwrap_or("") {
            "pure" => Effect::Pure,
            "quantum" => Effect::Quantum,
            "prob" => Effect::Prob,
            other => return Err(format!("unknown effect `{other}`")),
        };
        let body = &f["body"];
        let mut blocks = Vec::new();
        for b in body["blocks"].as_array().ok_or("missing blocks")? {
            let insts = b["insts"]
                .as_array()
                .ok_or("missing insts")?
                .iter()
                .map(inst_from_json)
                .collect::<Result<Vec<_>, _>>()?;
            blocks.push(Block { insts, term: term_from_json(&b["term"])? });
        }
        // Parameter slots are assigned first, in order, by lowering.
        let params: Vec<u32> =
            (0..f["params"].as_array().map(|p| p.len()).unwrap_or(0) as u32).collect();
        funcs.push(GirFunc {
            name: f["name"].as_str().unwrap_or("").to_string(),
            effect,
            params,
            slot_count: body["slot_count"].as_u64().unwrap_or(0) as u32,
            blocks,
        });
    }
    let entry = funcs.iter().position(|f| f.name == "main");
    Ok(Gir { funcs, entry })
}

#[cfg(test)]
mod gir_round_trip {
    use super::*;

    #[test]
    fn gir_json_round_trips_to_an_identical_program() {
        let mut s = Sources::default();
        let src = "fn main() -> Measured<Bool> prob {\n let a = qubit()\n let b = qubit()\n h(a)\n let (a, b) = cx(a, b)\n drop(b)\n measure(a)\n}\n";
        let p = compile_source(src, &mut s).expect("compiles");
        let back = gir_from_json(&gir_json(&p)).expect("reads back");
        assert_eq!(back.dump(), p.gir.dump(), "same instructions after the round trip");
        let cfg = gala_sim::SimConfig { shots: 400, seed: 3, ..Default::default() };
        let a = gala_sim::run_entry(&p.gir, &cfg).unwrap();
        let b = gala_sim::run_entry(&back, &cfg).unwrap();
        assert_eq!(a.histogram, b.histogram);
    }
}
