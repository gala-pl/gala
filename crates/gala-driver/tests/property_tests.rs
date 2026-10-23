//! Property tests for the whole pipeline: generated classical programs compile, run on the
//! simulator, and return the same value as a reference evaluation in Rust.

use gala_driver::{compile_source, Sources};
use gala_sim::{run_entry, SimConfig, Value};
use proptest::prelude::*;

#[derive(Debug, Clone)]
struct Expr {
    src: String,
    value: i64,
}

fn expr_strategy() -> impl Strategy<Value = Expr> {
    let leaf = (-50i64..50).prop_map(|v| Expr { src: format!("({v})"), value: v });
    leaf.prop_recursive(4, 32, 2, |inner| {
        (inner.clone(), inner, prop::sample::select(vec!['+', '-', '*'])).prop_map(|(a, b, op)| {
            let (value, sym) = match op {
                '+' => (a.value.wrapping_add(b.value), "+"),
                '-' => (a.value.wrapping_sub(b.value), "-"),
                _ => (a.value.wrapping_mul(b.value), "*"),
            };
            Expr { src: format!("({} {sym} {})", a.src, b.src), value }
        })
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn generated_arithmetic_matches_reference(e in expr_strategy()) {
        let src = format!("fn main() -> Int {{ {} }}", e.src);
        let mut sources = Sources::default();
        let program = compile_source(&src, &mut sources)
            .map_err(|d| TestCaseError::fail(format!("{src}: {:?}", d.errors())))?;
        let report = run_entry(&program.gir, &SimConfig::default())
            .map_err(|err| TestCaseError::fail(format!("{src}: {err}")))?;
        prop_assert_eq!(report.first_result, Some(Value::Int(e.value)), "{}", src);
    }

    #[test]
    fn comparisons_agree_with_reference(a in -100i64..100, b in -100i64..100) {
        let src = format!("fn main() -> Bool {{ ({a}) < ({b}) }}");
        let mut sources = Sources::default();
        let program = compile_source(&src, &mut sources)
            .map_err(|d| TestCaseError::fail(format!("{src}: {:?}", d.errors())))?;
        let report = run_entry(&program.gir, &SimConfig::default())
            .map_err(|err| TestCaseError::fail(format!("{src}: {err}")))?;
        prop_assert_eq!(report.first_result, Some(Value::Bool(a < b)));
    }
}
