use gala_parser::parse_file;
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

fn gen_fn_def() -> impl Strategy<Value = String> {
    (gen_ident(), gen_params(), gen_type_name(), gen_fn_body()).prop_map(
        |(name, params, ret, body)| format!("fn {}({}) -> {} {{\n{}}}", name, params, ret, body,),
    )
}

proptest! {
    #[test]
    fn test_parser_doesnt_crash(s in "\\PC*") {
        let mut map = SourceMap::new();
        let fid = map.add_file("<test>".into(), s.clone());
        let _ = parse_file(fid, &s, &mut map);
    }

    #[test]
    fn test_parse_valid_fn(fn_def in gen_fn_def()) {
        let mut map = SourceMap::new();
        let fid = map.add_file("<test>".into(), fn_def.clone());
        let result = parse_file(fid, &fn_def, &mut map);
        prop_assert!(result.is_ok(), "failed to parse valid fn: {:?}", result.err());
    }
}
