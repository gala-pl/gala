//! Linearity checking: qubits are used exactly once.
//!
//! The checker walks each function's HIR and tracks every linear binding as `Live` or
//! `Consumed`. A linear value is consumed when it is moved: passed to a function, bound to
//! another name, measured, dropped, returned, or overwritten. Gates are the one exception to
//! "consumed": `h(q)` takes a qubit and returns a qubit, so as a statement it rebinds `q` to the
//! new value. Gates also accept register elements (`h(q[i])`), which borrows the register.
//!
//! Diagnostics:
//! - `E0412` use after consume
//! - `E0402` a linear value is dropped implicitly (not consumed before leaving scope)
//! - `E0403` the same qubit is passed twice in one call (aliasing)
//! - `E0405` a qubit is consumed on only some branches
//! - `E0412` (in loops) a qubit consumed inside a loop body is used on the next iteration

use crate::TypeInfo;
use gala_diagnostics::{codes, Diagnostic, Diagnostics};
use gala_hir::{Builtin, HirBlock, HirExpr, HirExprKind, HirPackage, HirPat, HirStmt, LocalId};
use gala_span::Span;
use std::collections::HashMap;

/// Summary of a linearity run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LinearSummary {
    /// Number of linear bindings that were tracked across all functions.
    pub tracked_locals: usize,
}

#[derive(Debug, Clone)]
enum State {
    Live { decl: Span, name: String },
    Consumed { at: Span, by: String },
}

/// Check linearity for every function in the package.
pub fn check_linearity(pkg: &HirPackage, info: &TypeInfo) -> (LinearSummary, Diagnostics) {
    let mut lin = Lin {
        info,
        pkg,
        state: HashMap::new(),
        scopes: Vec::new(),
        diags: Diagnostics::new(),
        tracked: 0,
    };
    for f in &pkg.fns {
        let Some(body) = &f.body else { continue };
        lin.state.clear();
        lin.scopes.clear();
        lin.scopes.push(Vec::new());
        for p in &f.params {
            if lin.is_linear(p.local) {
                lin.tracked += 1;
                lin.state.insert(p.local, State::Live { decl: p.span, name: p.name.clone() });
                lin.scopes[0].push(p.local);
            }
        }
        lin.block_inner(body);
        // Whatever is still live when the function returns was never consumed.
        let params = lin.scopes.pop().unwrap_or_default();
        lin.report_live(&params);
        lin.scopes.clear();
    }
    (LinearSummary { tracked_locals: lin.tracked }, lin.diags)
}

struct Lin<'a> {
    pkg: &'a HirPackage,
    info: &'a TypeInfo,
    state: HashMap<LocalId, State>,
    scopes: Vec<Vec<LocalId>>,
    diags: Diagnostics,
    tracked: usize,
}

impl<'a> Lin<'a> {
    fn is_linear(&self, l: LocalId) -> bool {
        self.info.local_ty.get(&l).is_some_and(|t| t.is_linear())
    }

    fn name_of(&self, l: LocalId) -> String {
        self.pkg.local_name(l).to_string()
    }

    fn report_live(&mut self, locals: &[LocalId]) {
        for l in locals {
            if let Some(State::Live { decl, name }) = self.state.get(l).cloned() {
                self.diags.push(
                    Diagnostic::error(
                        codes::IMPLICIT_DROP,
                        format!("qubit `{name}` is never consumed"),
                    )
                    .with_primary_label(decl, "dropped implicitly here")
                    .with_note(
                        "a qubit goes out of scope only after measure(), drop(), or being returned",
                    )
                    .with_help(format!(
                        "call `drop({name})` or `measure({name})` before the end of its scope"
                    )),
                );
                // Forget it so one leak is reported once and later code does not see a use-after.
                self.state.remove(l);
            }
        }
    }

    /// Walk a block with its own scope. Live linear locals declared in it must be consumed.
    fn block_inner(&mut self, b: &HirBlock) {
        self.scopes.push(Vec::new());
        for stmt in &b.stmts {
            self.stmt(stmt);
        }
        if let Some(t) = &b.tail {
            self.move_expr(t);
        }
        let scope = self.scopes.pop().unwrap_or_default();
        self.report_live(&scope);
        for l in scope {
            self.state.remove(&l);
        }
    }

    fn bind_pat(&mut self, pat: &HirPat, span: Span) {
        match pat {
            HirPat::Bind { local, name, .. } => {
                if self.is_linear(*local) {
                    self.tracked += 1;
                    self.state.insert(*local, State::Live { decl: span, name: name.clone() });
                    if let Some(s) = self.scopes.last_mut() {
                        s.push(*local);
                    }
                }
            }
            HirPat::Tuple(items, _) => {
                for p in items {
                    self.bind_pat(p, span);
                }
            }
            HirPat::Wild | HirPat::Lit(_, _) => {}
        }
    }

    fn stmt(&mut self, s: &HirStmt) {
        match s {
            HirStmt::Let { pat, init, span, .. } => {
                self.move_expr(init);
                self.bind_pat(pat, *span);
            }
            HirStmt::Expr(e) => self.expr_stmt(e),
            HirStmt::Return { value, .. } => {
                if let Some(v) = value {
                    self.move_expr(v);
                }
                // Everything still live in any enclosing scope leaks at this return.
                let live: Vec<LocalId> = self.scopes.iter().flatten().copied().collect();
                self.report_live(&live);
            }
        }
    }

    /// An expression evaluated for its effect. In-place gates rebind their qubit arguments.
    fn expr_stmt(&mut self, e: &HirExpr) {
        if let HirExprKind::Call { callee, args, .. } = &e.kind {
            if let HirExprKind::Builtin(b) = callee.kind {
                if b.is_gate() {
                    // A fresh qubit passed to a gate and not bound is lost with the gate's output.
                    for a in args {
                        let fresh =
                            !matches!(a.kind, HirExprKind::Local(_, _) | HirExprKind::Index { .. });
                        if fresh && self.info.ty(a.id).is_linear() {
                            self.diags.push(
                                Diagnostic::error(
                                    codes::IMPLICIT_DROP,
                                    "a qubit passed to a gate is lost",
                                )
                                .with_primary_label(a.span, "this qubit is never bound or dropped")
                                .with_help("bind it with `let`, and drop it later"),
                            );
                        }
                    }
                    // `h(q)` as a statement: the gate consumes `q` and gives back a new `q`.
                    self.gate_args(args, Some(e.span), b);
                    return;
                }
            }
        }
        if self.info.ty(e.id).is_linear() {
            self.diags.push(
                Diagnostic::error(
                    codes::IMPLICIT_DROP,
                    "a qubit value is discarded without being bound",
                )
                .with_primary_label(e.span, "result is not used")
                .with_help("bind the result with `let`, or pass it to `measure` or `drop`"),
            );
        }
        self.move_expr(e);
    }

    /// Arguments of a gate call. Qubit arguments are consumed and rebound to the gate's outputs
    /// when `rebind_span` is given (the in-place statement form); register elements are borrowed.
    fn gate_args(&mut self, args: &[HirExpr], rebind_span: Option<Span>, _b: Builtin) {
        self.check_aliasing(args);
        for a in args {
            match &a.kind {
                HirExprKind::Index { base, index } if is_local(base) => {
                    self.move_expr(index);
                }
                HirExprKind::Local(l, name) => {
                    let l = *l;
                    self.consume(l, name.clone(), a.span, "gate");
                    if let (Some(span), true) = (rebind_span, self.is_linear(l)) {
                        // The gate returns the qubit; the name now refers to the new value.
                        self.state.insert(l, State::Live { decl: span, name: name.clone() });
                    }
                }
                _ => self.move_expr(a),
            }
        }
    }

    fn check_aliasing(&mut self, args: &[HirExpr]) {
        let mut seen: Vec<LocalId> = Vec::new();
        for a in args {
            if let HirExprKind::Local(l, name) = &a.kind {
                if self.is_linear(*l) {
                    if seen.contains(l) {
                        self.diags.push(
                            Diagnostic::error(codes::ALIASING_QUBIT, format!("qubit `{name}` is passed twice in one call"))
                                .with_primary_label(a.span, "second use of the same qubit")
                                .with_note("two arguments naming one qubit would alias it, which quantum mechanics forbids")
                                .with_help("pass two distinct qubits"),
                        );
                    }
                    seen.push(*l);
                }
            }
        }
    }

    fn consume(&mut self, l: LocalId, name: String, at: Span, by: &str) {
        if !self.is_linear(l) {
            return;
        }
        match self.state.get(&l).cloned() {
            Some(State::Live { .. }) | None => {
                self.state.insert(l, State::Consumed { at, by: by.to_string() });
            }
            Some(State::Consumed { at: prev, by: prev_by }) => {
                self.diags.push(
                    Diagnostic::error(codes::USE_AFTER_CONSUME, format!("qubit `{name}` used after it was consumed"))
                        .with_primary_label(at, "used here")
                        .with_secondary_label(prev, format!("consumed by {prev_by} here"))
                        .with_note("a qubit is a linear resource: measuring, dropping, or passing it ends its life")
                        .with_help("use the value returned by the operation, or the classical result of measure()"),
                );
            }
        }
    }

    /// Evaluate an expression in a value (move) context.
    fn move_expr(&mut self, e: &HirExpr) {
        match &e.kind {
            HirExprKind::Local(l, name) => {
                let (l, name) = (*l, name.clone());
                self.consume(l, name, e.span, "use");
            }
            HirExprKind::Lit(_)
            | HirExprKind::Const(_)
            | HirExprKind::Fn(_)
            | HirExprKind::Builtin(_)
            | HirExprKind::Error => {}
            HirExprKind::Call { callee, args, .. } => match callee.kind {
                HirExprKind::Builtin(b) if b.is_gate() => self.gate_args(args, None, b),
                HirExprKind::Builtin(Builtin::Measure) | HirExprKind::Builtin(Builtin::Drop) => {
                    for a in args {
                        if matches!(&a.kind, HirExprKind::Index { base, .. } if is_local(base)) {
                            self.diags.push(
                                    Diagnostic::error(codes::UNSUPPORTED_FEATURE, "a single qubit cannot be measured or dropped out of a register yet")
                                        .with_primary_label(a.span, "register element")
                                        .with_help("consume the whole register instead"),
                                );
                        } else {
                            self.move_expr(a);
                        }
                    }
                }
                _ => {
                    self.check_aliasing(args);
                    for a in args {
                        self.move_expr(a);
                    }
                }
            },
            HirExprKind::Unary { expr, .. } => self.move_expr(expr),
            HirExprKind::Binary { lhs, rhs, .. } => {
                self.move_expr(lhs);
                self.move_expr(rhs);
            }
            HirExprKind::If { cond, then_block, else_expr } => {
                self.move_expr(cond);
                let before = self.state.clone();
                self.block_inner(then_block);
                let after_then = self.state.clone();
                self.state = before.clone();
                if let Some(e2) = else_expr {
                    self.move_expr(e2);
                }
                let after_else = self.state.clone();
                self.merge(&before, &[after_then, after_else], e.span);
            }
            HirExprKind::Match { scrutinee, arms } => {
                self.move_expr(scrutinee);
                let before = self.state.clone();
                let mut outcomes = Vec::new();
                for arm in arms {
                    self.state = before.clone();
                    self.scopes.push(Vec::new());
                    self.bind_pat(&arm.pat, arm.body.span);
                    if let Some(g) = &arm.guard {
                        self.move_expr(g);
                    }
                    self.move_expr(&arm.body);
                    let scope = self.scopes.pop().unwrap_or_default();
                    self.report_live(&scope);
                    outcomes.push(self.state.clone());
                }
                if outcomes.is_empty() {
                    self.state = before.clone();
                } else {
                    self.merge(&before, &outcomes, e.span);
                }
            }
            HirExprKind::For { pat, iterable, body } => {
                self.move_expr(iterable);
                let before = self.state.clone();
                self.scopes.push(Vec::new());
                self.bind_pat(pat, e.span);
                self.block_inner(body);
                let scope = self.scopes.pop().unwrap_or_default();
                self.report_live(&scope);
                self.loop_check(&before, e.span);
            }
            HirExprKind::While { cond, body } => {
                self.move_expr(cond);
                let before = self.state.clone();
                self.block_inner(body);
                self.loop_check(&before, e.span);
            }
            HirExprKind::Block(b) => self.block_inner(b),
            HirExprKind::Tuple(items) | HirExprKind::Array(items) => {
                for i in items {
                    self.move_expr(i);
                }
            }
            HirExprKind::Index { base, index } => {
                if is_local(base) {
                    // Reading a register element as a value would move one qubit out of the
                    // register. That is not supported yet; borrowing it through a gate is.
                    self.diags.push(
                        Diagnostic::error(
                            codes::UNSUPPORTED_FEATURE,
                            "moving one qubit out of a register is not supported yet",
                        )
                        .with_primary_label(e.span, "register element used as a value")
                        .with_help("apply gates to `q[i]` in place, or bind the whole register"),
                    );
                }
                self.move_expr(index);
            }
            HirExprKind::Assign { target, value, .. } => {
                self.move_expr(value);
                if let HirExprKind::Local(l, name) = &target.kind {
                    let (l, name) = (*l, name.clone());
                    if self.is_linear(l) {
                        if let Some(State::Live { .. }) = self.state.get(&l) {
                            self.diags.push(
                                Diagnostic::error(
                                    codes::IMPLICIT_DROP,
                                    format!("assigning over live qubit `{name}` drops it"),
                                )
                                .with_primary_label(e.span, "previous value is lost")
                                .with_help("consume the old value first with `measure` or `drop`"),
                            );
                        }
                        self.state.insert(l, State::Live { decl: e.span, name });
                    }
                }
            }
        }
    }

    /// Merge the states produced by alternative branches. A local that is live on one path and
    /// consumed on another is an error, because later code cannot know which holds.
    fn merge(
        &mut self,
        before: &HashMap<LocalId, State>,
        outcomes: &[HashMap<LocalId, State>],
        span: Span,
    ) {
        let mut merged = HashMap::new();
        for (l, prior) in before {
            let states: Vec<&State> = outcomes.iter().filter_map(|o| o.get(l)).collect();
            let any_consumed = states.iter().any(|s| matches!(s, State::Consumed { .. }));
            let all_consumed = states.iter().all(|s| matches!(s, State::Consumed { .. }));
            if any_consumed && !all_consumed {
                let name = self.name_of(*l);
                self.diags.push(
                    Diagnostic::error(codes::BRANCH_LINEARITY_MISMATCH, format!("qubit `{name}` is consumed on only some branches"))
                        .with_primary_label(span, "branches disagree about the qubit")
                        .with_note("after the branch, the qubit is either live or gone, and the compiler cannot tell which")
                        .with_help("consume it on every branch, or on none"),
                );
            }
            let chosen = states
                .iter()
                .find(|s| matches!(s, State::Consumed { .. }))
                .or_else(|| states.first())
                .map(|s| (*s).clone())
                .unwrap_or_else(|| prior.clone());
            merged.insert(*l, chosen);
        }
        self.state = merged;
    }

    /// A value consumed inside a loop body is used again on the next iteration.
    fn loop_check(&mut self, before: &HashMap<LocalId, State>, span: Span) {
        let now = self.state.clone();
        for (l, prior) in before {
            if let (State::Live { .. }, Some(State::Consumed { at, .. })) = (prior, now.get(l)) {
                let name = self.name_of(*l);
                self.diags.push(
                    Diagnostic::error(
                        codes::USE_AFTER_CONSUME,
                        format!("qubit `{name}` is consumed inside a loop"),
                    )
                    .with_primary_label(span, "the loop body runs again")
                    .with_secondary_label(*at, "consumed here, on every iteration")
                    .with_help("create the qubit inside the loop body, or restructure the loop"),
                );
                self.state.insert(*l, prior.clone());
            }
        }
    }
}

fn is_local(e: &HirExpr) -> bool {
    matches!(e.kind, HirExprKind::Local(_, _))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gala_hir::lower_file;
    use gala_parser::parse_source;
    use gala_span::FileId;

    fn lin(src: &str) -> Vec<String> {
        let out = parse_source(FileId(0), src);
        assert!(!out.diagnostics.has_errors(), "parse: {:?}", out.diagnostics);
        let (pkg, d) = lower_file(FileId(0), out.items);
        assert!(!d.has_errors(), "lower: {d:?}");
        let (info, td) = crate::check_package(&pkg);
        assert!(
            !td.has_errors(),
            "types: {:?}",
            td.errors().iter().map(|e| e.code.to_string()).collect::<Vec<_>>()
        );
        let (_, ld) = check_linearity(&pkg, &info);
        ld.errors().iter().map(|e| e.code.to_string()).collect()
    }

    fn assert_clean(src: &str) {
        assert!(lin(src).is_empty(), "expected no linearity errors, got {:?}", lin(src));
    }

    fn assert_code(src: &str, code: &str) {
        let got = lin(src);
        assert!(got.iter().any(|c| c == code), "expected {code}, got {got:?}");
    }

    #[test]
    fn use_after_consume_is_e0412() {
        assert_code("fn f(q: Qubit) -> Qubit { let a = q\n let b = q\n b }", "E0412");
    }

    #[test]
    fn use_after_measure_is_e0412() {
        assert_code(
            "fn f() -> Measured<Bool> prob { let a = qubit()\n let r = measure(a)\n h(a)\n r }",
            "E0412",
        );
    }

    #[test]
    fn unconsumed_qubit_is_implicit_drop_e0402() {
        assert_code("fn f() quantum { let q = qubit() }", "E0402");
    }

    #[test]
    fn explicit_drop_is_clean() {
        assert_clean("fn f() quantum { let q = qubit()\n drop(q) }");
    }

    #[test]
    fn measure_consumes_and_is_clean() {
        assert_clean("fn f() -> Measured<Bool> prob { let q = qubit()\n h(q)\n measure(q) }");
    }

    #[test]
    fn same_qubit_twice_in_call_is_aliasing_e0403() {
        assert_code("fn f() quantum { let a = qubit()\n cx(a, a)\n drop(a) }", "E0403");
    }

    #[test]
    fn in_place_gates_rebind_and_are_clean() {
        assert_clean("fn f() quantum { let a = qubit()\n let b = qubit()\n h(a)\n cx(a, b)\n drop(a)\n drop(b) }");
    }

    #[test]
    fn discarded_linear_result_is_e0402() {
        assert_code("fn f() quantum { h(qubit()); }", "E0402");
    }

    #[test]
    fn consumed_on_one_branch_only_is_e0405() {
        assert_code("fn f(c: Bool) quantum { let q = qubit()\n if c { drop(q) } }", "E0405");
    }

    #[test]
    fn consumed_on_both_branches_is_clean() {
        assert_clean(
            "fn f(c: Bool) quantum { let q = qubit()\n if c { drop(q) } else { drop(q) } }",
        );
    }

    #[test]
    fn consuming_outer_qubit_inside_loop_is_e0412() {
        assert_code("fn f() quantum { let q = qubit()\n for i in 0..3 { drop(q) } }", "E0412");
    }

    #[test]
    fn creating_qubit_inside_loop_is_clean() {
        assert_clean("fn f() quantum { for i in 0..3 { let q = qubit()\n h(q)\n drop(q) } }");
    }

    #[test]
    fn returning_a_qubit_moves_it_out() {
        assert_clean("fn f() -> Qubit quantum { let q = qubit()\n h(q) }");
    }

    #[test]
    fn early_return_leaks_live_qubit() {
        assert_code(
            "fn f(c: Bool) -> Int quantum { let q = qubit()\n if c { return 1 }\n drop(q)\n 0 }",
            "E0402",
        );
    }

    #[test]
    fn unconsumed_parameter_is_e0402() {
        assert_code("fn f(q: Qubit) quantum { h(q); }", "E0402");
    }

    #[test]
    fn index_into_register_borrows_for_gates() {
        assert_clean("fn f() quantum { let q = qubits<3>()\n h(q[0])\n cx(q[0], q[1])\n drop(q) }");
    }
}
