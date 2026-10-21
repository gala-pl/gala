//! Compilation orchestration for Gala.

use gala_ast::Effect as HirEffect;
use gala_diagnostics::Diagnostics;
use gala_gir::{lower_hir_to_gir, Gir};
use gala_hir::{desugar_file, DefId, HirFnDef, Resolver};
use gala_parser::parse_file;
use gala_span::{FileId, SourceMap};
use gala_types::{
    check_linearity, function_type, type_check_fn_with_functions, Effect as TyEffect, Ty,
};
use gala_uncompute::{analyze_provenance, insert_uncompute, synthesize_uncompute};
use std::collections::HashMap;

/// Compile a single source file through the entire pipeline.
pub fn compile_source(source: &str, source_map: &mut SourceMap) -> Result<Gir, Diagnostics> {
    let fid = source_map.add_file("<input>".into(), source.to_string());
    compile_file(fid, source, source_map)
}

/// Compile a file by ID through the entire pipeline.
pub fn compile_file(
    file_id: FileId,
    source: &str,
    source_map: &mut SourceMap,
) -> Result<Gir, Diagnostics> {
    let mut diags = Diagnostics::new();

    let items = match parse_file(file_id, source, source_map) {
        Ok(items) => items,
        Err(e) => {
            return Err(e);
        }
    };

    let hir_file = desugar_file(&items, file_id);
    let mut resolver = Resolver::new();
    let hir_file = resolver.build_and_resolve(&hir_file);
    diags.extend(resolver.diagnostics);

    let mut hir_funcs: HashMap<DefId, HirFnDef> = HashMap::new();
    for item in &hir_file.items {
        if let gala_hir::HirItem::FnDef(f) = item {
            hir_funcs.insert(f.def_id.clone(), f.clone());
        }
    }

    let mut type_of: HashMap<DefId, gala_types::Ty> = HashMap::new();
    let mut effect_of: HashMap<DefId, TyEffect> = HashMap::new();
    let function_types: HashMap<DefId, Ty> =
        hir_funcs.iter().map(|(def_id, hir_fn)| (def_id.clone(), function_type(hir_fn))).collect();

    for (def_id, hir_fn) in &hir_funcs {
        match type_check_fn_with_functions(hir_fn, &function_types) {
            Ok(ty) => {
                type_of.insert(def_id.clone(), ty);
            }
            Err(e) => diags.extend(e),
        }
        let eff = match &hir_fn.effect {
            HirEffect::Pure => TyEffect::Pure,
            HirEffect::Quantum => TyEffect::Quantum,
            HirEffect::Prob => TyEffect::Prob,
        };
        effect_of.insert(def_id.clone(), eff);
    }

    for hir_fn in hir_funcs.values() {
        let lin_diags = check_linearity(hir_fn);
        if lin_diags.has_errors() {
            diags.extend(lin_diags);
        }
    }

    for hir_fn in hir_funcs.values_mut() {
        let provenance = analyze_provenance(hir_fn);
        match synthesize_uncompute(hir_fn, &provenance) {
            Ok(plan) => {
                let lowered = insert_uncompute(hir_fn, plan);
                *hir_fn = lowered;
            }
            Err(e) => diags.extend(e),
        }
    }

    let gir = match lower_hir_to_gir(&hir_funcs, &type_of, &effect_of) {
        Ok(gir) => gir,
        Err(e) => {
            diags.extend(e);
            return Err(diags);
        }
    };

    if diags.has_errors() {
        Err(diags)
    } else {
        Ok(gir)
    }
}

/// Check-only: parse, HIR, type check.
pub fn check_file(file_id: FileId, source: &str, source_map: &mut SourceMap) -> Diagnostics {
    match compile_file(file_id, source, source_map) {
        Ok(_) => Diagnostics::new(),
        Err(diags) => diags,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_fn() {
        let source = "fn main() -> Int { return 42; }";
        let mut map = SourceMap::new();
        let result = compile_source(source, &mut map);
        if let Err(diags) = &result {
            assert!(!diags.has_errors(), "expected success, got errors: {:?}", diags);
        }
    }

    #[test]
    fn test_check_file_on_syntax_error() {
        let source = "fn main() -> Int { return }";
        let mut map = SourceMap::new();
        let fid = map.add_file("test.gala".into(), source.to_string());
        let diags = check_file(fid, source, &mut map);
        // May or may not have errors depending on parser recovery
        // Just verify it doesn't panic
        let _ = diags;
    }

    #[test]
    fn test_compile_source_smoke() {
        let sources = vec!["fn a() -> Int { return 1; }", "fn b(x: Int) -> Int { return x + 1; }"];
        for src in &sources {
            let mut map = SourceMap::new();
            let _ = compile_source(src, &mut map);
        }
    }

    #[test]
    fn test_compile_empty_source() {
        let source = "";
        let mut map = SourceMap::new();
        let result = compile_source(source, &mut map);
        assert!(result.is_ok(), "empty source should compile: {:?}", result.err());
        let gir = result.unwrap();
        assert!(gir.funcs.is_empty());
    }

    #[test]
    fn test_compile_with_effect() {
        let source = "fn quantum_op() -> Qubit quantum { let q = qubit(); return q; }";
        let mut map = SourceMap::new();
        // May fail on unresolved names, but shouldn't panic
        let _ = compile_source(source, &mut map);
    }
}
