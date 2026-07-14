use gala_fmt::format_source;
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
        |(name, params, ret, body)| format!("fn {}({}) -> {} {{\n{}}}", name, params, ret, body),
    )
}

fn gen_valid_program() -> impl Strategy<Value = String> {
    prop::collection::vec(gen_fn_def(), 1..3).prop_map(|fns| fns.join("\n"))
}

proptest! {
    #[test]
    fn test_formatter_doesnt_crash(s in "\\PC*") {
        let _ = format_source(&s);
    }

    #[test]
    fn test_formatter_idempotent_on_valid(program in gen_valid_program()) {
        let formatted = format_source(&program);
        let formatted2 = format_source(&formatted);
        prop_assert_eq!(formatted, formatted2, "formatter is not idempotent");
    }

    #[test]
    fn test_formatter_third_pass_stable(program in gen_valid_program()) {
        let formatted = format_source(&program);
        let formatted2 = format_source(&formatted);
        let formatted3 = format_source(&formatted2);
        prop_assert_eq!(formatted2, formatted3, "formatter not stable on third pass");
    }
}
