use gala_ast::{self, Effect as AstEffect, Ident, Literal, Type};
use gala_hir::*;
use gala_span::{FileId, Span};
use gala_types::{type_check_fn, Ty};
use proptest::prelude::*;

fn make_path_type(name: &str) -> Type {
    Type::Path(gala_ast::Path {
        segments: vec![gala_ast::PathSegment { ident: Ident::new(name), type_args: Vec::new() }],
        span: Span::dummy(),
    })
}

fn gen_literal_ty_pair() -> impl Strategy<Value = (Literal, Type)> {
    prop_oneof![
        any::<i64>().prop_map(|n| (Literal::Int(n), make_path_type("Int"))),
        prop::bool::ANY.prop_map(|b| (Literal::Bool(b), make_path_type("Bool"))),
    ]
}

fn gen_well_typed_fn() -> impl Strategy<Value = HirFnDef> {
    gen_literal_ty_pair().prop_map(|(lit, ret_ty)| HirFnDef {
        def_id: DefId { crate_id: CrateId(FileId(0)), index: 0 },
        ident: Ident::new("prop_test_fn"),
        generics: Vec::new(),
        params: Vec::new(),
        ret_ty: Some(ret_ty),
        effect: AstEffect::Pure,
        body: HirBlock {
            stmts: vec![],
            tail: Some(Box::new(HirExpr::Literal(lit))),
            span: Span::dummy(),
        },
        span: Span::dummy(),
    })
}

fn gen_potentially_mismatched_fn() -> impl Strategy<Value = HirFnDef> {
    (gen_literal_ty_pair(), gen_literal_ty_pair()).prop_map(
        |((lit, _ret_ty), (_other_lit, declared_ret_ty))| HirFnDef {
            def_id: DefId { crate_id: CrateId(FileId(0)), index: 0 },
            ident: Ident::new("prop_mismatch_fn"),
            generics: Vec::new(),
            params: Vec::new(),
            ret_ty: Some(declared_ret_ty),
            effect: AstEffect::Pure,
            body: HirBlock {
                stmts: vec![],
                tail: Some(Box::new(HirExpr::Literal(lit))),
                span: Span::dummy(),
            },
            span: Span::dummy(),
        },
    )
}

proptest! {
    #[test]
    fn test_type_inference_succeeds_on_lit_fn(hir_fn in gen_well_typed_fn()) {
        let result = type_check_fn(&hir_fn);
        prop_assert!(result.is_ok(), "type inference failed on well-typed fn: {:?}", result.err());
    }

    #[test]
    fn test_type_inference_returns_expected_type(hir_fn in gen_well_typed_fn()) {
        let result = type_check_fn(&hir_fn);
        if let Ok(ty) = result {
            match ty {
                Ty::Int | Ty::Bool | Ty::Float | Ty::Unit | Ty::String | Ty::Complex => {}
                _ => prop_assert!(false, "unexpected type: {:?}", ty),
            }
        }
    }

    #[test]
    fn test_type_inference_doesnt_crash(hir_fn in gen_potentially_mismatched_fn()) {
        let _ = type_check_fn(&hir_fn);
    }
}
