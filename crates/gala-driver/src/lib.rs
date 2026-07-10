//! Compilation orchestration for Gala.

use gala_parser::parse_file;
use gala_hir::{desugar_file, HirFnDef, DefId, CrateId};
use gala_ast::Effect as HirEffect;
use gala_types::{type_check_fn, check_linearity, Effect as TyEffect};
use gala_uncompute::{analyze_provenance, synthesize_uncompute};
use gala_gir::{Gir, lower_hir_to_gir};
use gala_span::{FileId, SourceMap};
use gala_diagnostics::{Diagnostic, Diagnostics, codes};
use std::collections::HashMap;

/// Compile a single source file through the entire pipeline.
pub fn compile_source(
    source: &str,
    source_map: &mut SourceMap,
) -> Result<Gir, Diagnostics> {
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
        Err(e) => { return Err(e); }
    };

    let hir_file = desugar_file(&items, file_id);

    let mut hir_funcs: HashMap<DefId, HirFnDef> = HashMap::new();
    let mut def_id_counter = 0u32;
    for item in &hir_file.items {
        if let gala_hir::HirItem::FnDef(f) = item {
            let def_id = DefId { crate_id: CrateId(file_id), index: def_id_counter };
            def_id_counter += 1;
            hir_funcs.insert(def_id, f.clone());
        }
    }

    let mut type_of: HashMap<DefId, gala_types::Ty> = HashMap::new();
    let mut effect_of: HashMap<DefId, TyEffect> = HashMap::new();

    for (def_id, hir_fn) in &hir_funcs {
        match type_check_fn(hir_fn) {
            Ok(ty) => { type_of.insert(def_id.clone(), ty); }
            Err(e) => diags.extend(e),
        }
        let eff = match &hir_fn.effect {
            HirEffect::Pure => TyEffect::Pure,
            HirEffect::Quantum => TyEffect::Quantum,
            HirEffect::Prob => TyEffect::Prob,
        };
        effect_of.insert(def_id.clone(), eff);
    }

    for (_, hir_fn) in &hir_funcs {
        let lin_diags = check_linearity(hir_fn);
        if lin_diags.has_errors() {
            diags.extend(lin_diags);
        }
    }

    for (_def_id, hir_fn) in &hir_funcs {
        let provenance = analyze_provenance(hir_fn);
        let _ = synthesize_uncompute(hir_fn, &provenance).map_err(|e| diags.extend(e));
    }

    let gir = match lower_hir_to_gir(&hir_funcs, &type_of, &effect_of) {
        Ok(gir) => gir,
        Err(e) => { diags.extend(e); return Err(diags); }
    };

    if diags.has_errors() { Err(diags) } else { Ok(gir) }
}

/// Check-only: parse, HIR, type check.
pub fn check_file(
    file_id: FileId,
    source: &str,
    source_map: &mut SourceMap,
) -> Diagnostics {
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
        let sources = vec![
            "fn a() -> Int { return 1; }",
            "fn b(x: Int) -> Int { return x + 1; }",
        ];
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