use gala_driver::compile_source;
use gala_span::SourceMap;
use proptest::prelude::*;

fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "fn" | "let"
            | "mut"
            | "if"
            | "else"
            | "for"
            | "in"
            | "while"
            | "return"
            | "import"
            | "as"
            | "struct"
            | "enum"
            | "trait"
            | "impl"
            | "type"
            | "const"
            | "pure"
            | "quantum"
            | "prob"
            | "qubit"
            | "qubits"
            | "measure"
            | "reverse"
            | "adjoint"
            | "control"
            | "grad"
            | "drop"
            | "true"
            | "false"
    )
}

fn gen_ident() -> impl Strategy<Value = String> {
    "[a-z]{3,10}".prop_filter("not a keyword", |s| !is_keyword(s))
}

fn gen_type_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(vec!["Int", "Float", "Bool", "String", "Unit"])
}

fn gen_literal() -> impl Strategy<Value = String> {
    prop_oneof![
        any::<i64>().prop_map(|n| n.to_string()),
        prop::bool::ANY.prop_map(|b| b.to_string()),
    ]
}

fn gen_params() -> impl Strategy<Value = String> {
    prop::collection::vec(
        (gen_ident(), gen_type_name()).prop_map(|(name, ty)| format!("{}: {}", name, ty)),
        0..2,
    )
    .prop_map(|params| params.join(", "))
}

fn gen_fn_body() -> impl Strategy<Value = String> {
    gen_literal().prop_map(|v| format!("    return {};\n", v))
}

fn gen_valid_program() -> impl Strategy<Value = String> {
    (gen_ident(), gen_params(), gen_type_name(), gen_fn_body()).prop_map(
        |(name, params, ret, body)| format!("fn {}({}) -> {} {{\n{}}}", name, params, ret, body),
    )
}

fn has_obvious_syntax_error(source: &str) -> bool {
    source.is_empty() || source.chars().all(|c| c.is_whitespace())
}

proptest! {
    #[test]
    fn test_compilation_doesnt_crash(s in "\\PC{0,500}") {
        let mut map = SourceMap::new();
        let _ = compile_source(&s, &mut map);
    }

    #[test]
    fn test_compilation_determinism(program in gen_valid_program()) {
        let mut map1 = SourceMap::new();
        let mut map2 = SourceMap::new();
        let result1 = compile_source(&program, &mut map1);
        let result2 = compile_source(&program, &mut map2);

        match (&result1, &result2) {
            (Ok(gir1), Ok(gir2)) => {
                prop_assert_eq!(gir1.funcs.len(), gir2.funcs.len(),
                    "determinism: different number of functions");
                let node_counts = |gir: &gala_gir::Gir| -> Vec<usize> {
                    gir.funcs
                        .values()
                        .map(|f| f.blocks.values().map(|b| b.nodes.len()).sum())
                        .collect()
                };
                let mut nodes1 = node_counts(gir1);
                let mut nodes2 = node_counts(gir2);
                nodes1.sort_unstable();
                nodes2.sort_unstable();
                prop_assert_eq!(nodes1, nodes2,
                    "determinism: different number of nodes across functions");
            }
            (Err(diags1), Err(diags2)) => {
                let errors1 = diags1.has_errors();
                let errors2 = diags2.has_errors();
                prop_assert_eq!(errors1, errors2,
                    "determinism: one run had errors, the other didn't");
            }
            _ => {
                panic!("determinism violation: one run succeeded, the other failed");
            }
        }
    }

    #[test]
    fn test_compile_generated_program(program in gen_valid_program()) {
        if has_obvious_syntax_error(&program) {
            return Ok(());
        }
        let mut map = SourceMap::new();
        let result = compile_source(&program, &mut map);
        if let Err(ref diags) = result {
            let errors = diags.errors();
            prop_assert!(errors.len() <= 3,
                "expected at most a few errors for generated program, got {}",
                errors.len());
        }
    }
}
