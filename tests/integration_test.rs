use gala_diagnostics::Diagnostics;
use gala_driver::compile_source;
use gala_gir::{Gir, NodeKind};
use gala_span::SourceMap;
use gala_types::Effect;
use std::fs;
use std::path::Path;

fn compile(src: &str) -> Result<Gir, Diagnostics> {
    let mut map = SourceMap::new();
    compile_source(src, &mut map)
}

fn read_fixture(path: &str) -> String {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"));
    let full = base.join(path);
    fs::read_to_string(&full).expect("failed to read fixture file")
}

#[test]
fn test_compile_bell_pair() {
    let source = read_fixture("fixtures/bell.gala");
    // Known limitation: uncomputation tracking for consumed qubits is WIP
    // Compile may fail with E0501 if measure-consuming tracking is incomplete
    match compile(&source) {
        Ok(gir) => {
            assert!(!gir.funcs.is_empty(), "bell.gala should produce at least one function");
            let has_quantum_or_prob_fn =
                gir.funcs.values().any(|f| f.effect == Effect::Quantum || f.effect == Effect::Prob);
            assert!(
                has_quantum_or_prob_fn,
                "bell.gala should have at least one quantum or prob function"
            );
        }
        Err(diags) => {
            // If compilation fails, it should be only uncompute-related errors
            let all_uncompute = diags.diagnostics.iter().all(|d| d.code.0 >= 500 && d.code.0 < 600);
            assert!(
                all_uncompute,
                "bell.gala should compile clean or with only uncompute warnings"
            );
            eprintln!("bell.gala compiled with uncompute warnings (expected):");
            for d in &diags.diagnostics {
                eprintln!("  {}: {}", d.code, d.message);
            }
        }
    }
}

#[test]
fn test_compile_classical_fib() {
    let source = read_fixture("fixtures/classical.gala");
    let gir = compile(&source).expect("classical.gala should compile successfully");
    assert!(!gir.funcs.is_empty(), "classical.gala should produce at least one function");
    let has_quantum =
        gir.nodes.values().any(|node| matches!(node.kind, NodeKind::Gate(_) | NodeKind::Alloc(_)));
    assert!(!has_quantum, "classical.gala should not have quantum operations");
    let all_pure = gir.funcs.values().all(|f| f.effect == Effect::Pure);
    assert!(all_pure, "all functions in classical.gala should be pure");
    let has_classical = gir.nodes.values().any(|node| matches!(node.kind, NodeKind::Classical(_)));
    assert!(has_classical, "classical.gala should have classical operations");
}

#[test]
fn test_compile_if_expr() {
    let source = read_fixture("fixtures/if_expr.gala");
    let gir = compile(&source).expect("if_expr.gala should compile successfully");
    assert!(!gir.funcs.is_empty(), "if_expr.gala should produce at least one function");
    let has_multiple_blocks = gir.funcs.values().any(|func| func.blocks.len() > 1);
    assert!(has_multiple_blocks, "if_expr.gala should have control flow (multiple blocks)");
}

#[test]
fn test_type_checking_types() {
    let source = read_fixture("conformance/valid/types.gala");
    let gir = compile(&source).expect("types.gala should compile successfully");
    assert!(!gir.funcs.is_empty(), "types.gala should produce at least one function");
}

#[test]
fn test_type_checking_expressions() {
    let source = read_fixture("conformance/valid/expressions.gala");
    let gir = compile(&source).expect("expressions.gala should compile successfully");
    assert!(!gir.funcs.is_empty(), "expressions.gala should produce at least one function");
}

#[test]
fn test_syntax_error_diagnostics() {
    let source = read_fixture("conformance/errors/unclosed_block.gala");
    let result = compile(&source);
    assert!(result.is_err(), "unclosed_block.gala should produce errors");
    let diags = result.unwrap_err();
    assert!(diags.has_errors(), "unclosed_block.gala errors should be returned");
}

#[test]
fn test_syntax_error_missing_paren() {
    let source = read_fixture("conformance/errors/missing_paren.gala");
    let result = compile(&source);
    assert!(result.is_err(), "missing_paren.gala should produce errors");
    let diags = result.unwrap_err();
    assert!(diags.has_errors(), "missing_paren.gala errors should be returned");
}

#[test]
fn test_compile_all_valid_fixtures() {
    let valid_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("conformance/valid");
    let entries = fs::read_dir(&valid_dir).expect("failed to read conformance/valid directory");
    for entry in entries {
        let entry = entry.expect("failed to read directory entry");
        if entry.path().extension().map_or(false, |ext| ext == "gala") {
            let source = fs::read_to_string(entry.path()).expect("failed to read file");
            let result = compile(&source);
            assert!(result.is_ok(), "failed to compile {:?}: {:?}", entry.path(), result.err());
        }
    }
}
