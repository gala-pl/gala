//! Uncomputation analysis: may an ancilla be released with `drop`?
//!
//! Releasing a qubit that is still entangled with live qubits corrupts the computation. The fix
//! is to undo the gates that entangled it, which returns the ancilla to |0⟩ before release. That
//! is only possible when the undo is well defined:
//!
//! - every gate in the ancilla's causal cone is unitary (all gates are, by construction);
//! - every qubit in the cone is still live at the `drop` (a measured or released qubit cannot be
//!   reversed); and
//! - the cone does not pass through a call to a user function, whose internals are opaque here.
//!
//! The analysis walks each function in program order and records lineage events: a qubit keeps
//! its identity (its *lineage*) through gates, so a gate on `q` produces an event on `q`'s
//! lineage. At `drop(q)` the analysis collects the cone of lineages entangled with `q` since its
//! allocation and checks the conditions above. A refusal is `E0530` and names the blocking
//! qubit. The inverse gates themselves are applied from the recorded history when the program
//! runs (see `gala-gir`), so angles and branch paths are exact.

use gala_diagnostics::{codes, Diagnostic, Diagnostics};
use gala_hir::{Builtin, HirBlock, HirExpr, HirExprKind, HirPackage, HirPat, HirStmt};
use gala_span::Span;
use gala_types::TypeInfo;
use std::collections::{HashMap, HashSet};

/// Summary of an uncomputation run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UncomputeSummary {
    /// Number of `drop` calls checked.
    pub drops_checked: usize,
    /// Number of `drop` calls proven safe.
    pub drops_accepted: usize,
}

/// One step in a function's lineage history.
#[derive(Debug, Clone)]
enum Event {
    /// A qubit allocated by `qubit()` or `qubits<N>()`.
    Alloc { lineage: u32 },
    /// A qubit produced by a call to a user function (its history is unknown here).
    Opaque { lineage: u32, callee: String },
    /// A gate acting on one or two lineages.
    Gate { lineages: Vec<u32> },
    /// A destructive measurement.
    Measure { lineage: u32 },
    /// A release (`drop`) that succeeded.
    Release { lineage: u32 },
}

/// Analyse every `drop` in the package. `info` is accepted for future use by type-directed
/// analyses; the lineage walk itself needs only the HIR.
pub fn analyze(pkg: &HirPackage, _info: &TypeInfo) -> (UncomputeSummary, Diagnostics) {
    let mut out = (UncomputeSummary::default(), Diagnostics::new());
    for f in &pkg.fns {
        let Some(body) = &f.body else { continue };
        let mut a = Analyzer {
            pkg,
            names: HashMap::new(),
            events: Vec::new(),
            next_lineage: 0,
            lineage_name: Vec::new(),
            summary: UncomputeSummary::default(),
            diags: Diagnostics::new(),
            undone: HashSet::new(),
        };
        // Parameters are not allocated here, so their lineages have no Alloc event.
        for p in &f.params {
            let count = match &p.ty {
                gala_ast::Type::Qubit => 1,
                gala_ast::Type::Qubits(n) => gala_hir::const_int(n).unwrap_or(0).max(0) as usize,
                _ => 0,
            };
            if count > 0 {
                let ls: Vec<u32> = (0..count).map(|_| a.fresh(p.name.clone())).collect();
                a.names.insert(p.local, ls);
            }
        }
        a.block(body);
        out.0.drops_checked += a.summary.drops_checked;
        out.0.drops_accepted += a.summary.drops_accepted;
        out.1.extend(a.diags);
    }
    out
}

struct Analyzer<'a> {
    pkg: &'a HirPackage,
    /// Lineages of each linear local (a register has one lineage per element).
    names: HashMap<gala_hir::LocalId, Vec<u32>>,
    events: Vec<Event>,
    next_lineage: u32,
    lineage_name: Vec<String>,
    summary: UncomputeSummary,
    diags: Diagnostics,
    /// Gate events already undone by a release. They no longer entangle anything.
    undone: HashSet<usize>,
}

impl<'a> Analyzer<'a> {
    fn fresh(&mut self, name: String) -> u32 {
        let id = self.next_lineage;
        self.next_lineage += 1;
        self.lineage_name.push(name);
        id
    }

    fn block(&mut self, b: &HirBlock) {
        for s in &b.stmts {
            self.stmt(s);
        }
        if let Some(t) = &b.tail {
            self.eval(t);
        }
    }

    fn stmt(&mut self, s: &HirStmt) {
        match s {
            HirStmt::Let { pat, init, .. } => {
                let lines = self.eval(init).unwrap_or_default();
                self.bind(pat, lines);
            }
            HirStmt::Expr(e) => {
                self.eval(e);
            }
            HirStmt::Return { value, .. } => {
                if let Some(v) = value {
                    self.eval(v);
                }
            }
        }
    }

    /// Bind the names in `pat` to the lineages of the value. A tuple pattern takes one lineage per
    /// element when the counts agree, which is how `let (a, b) = cx(a, b)` keeps both qubits.
    fn bind(&mut self, pat: &HirPat, lines: Vec<u32>) {
        match pat {
            HirPat::Bind { local, .. } => {
                if !lines.is_empty() {
                    self.names.insert(*local, lines);
                }
            }
            HirPat::Tuple(items, _) => {
                if items.len() == lines.len() {
                    for (p, l) in items.iter().zip(lines) {
                        self.bind(p, vec![l]);
                    }
                }
            }
            HirPat::Wild | HirPat::Lit(_, _) => {}
        }
    }

    /// Lineage-level evaluation: returns the lineages carried by a qubit-valued expression, or
    /// `None` for classical values. Operations are recorded as events as they are evaluated.
    fn eval(&mut self, e: &HirExpr) -> Option<Vec<u32>> {
        match &e.kind {
            HirExprKind::Local(l, _) => self.names.get(l).cloned(),
            HirExprKind::Index { base, index } => {
                self.eval(index);
                let regs = self.eval(base)?;
                match literal(index) {
                    Some(i) if i >= 0 && (i as usize) < regs.len() => Some(vec![regs[i as usize]]),
                    // A non-literal index may name any element: keep the whole register.
                    _ => Some(regs),
                }
            }
            HirExprKind::Call { callee, args, generics, .. } => {
                self.call(callee, generics, args, e.span)
            }
            HirExprKind::Block(b) => {
                self.block(b);
                None
            }
            HirExprKind::If { cond, then_block, else_expr } => {
                self.eval(cond);
                self.block(then_block);
                if let Some(e2) = else_expr {
                    self.eval(e2);
                }
                None
            }
            HirExprKind::For { body, iterable, .. } => {
                self.eval(iterable);
                self.block(body);
                None
            }
            HirExprKind::While { cond, body } => {
                self.eval(cond);
                self.block(body);
                None
            }
            HirExprKind::Match { scrutinee, arms } => {
                self.eval(scrutinee);
                for arm in arms {
                    self.eval(&arm.body);
                }
                None
            }
            HirExprKind::Tuple(xs) => {
                let mut all = Vec::new();
                for x in xs {
                    all.extend(self.eval(x).unwrap_or_default());
                }
                if all.is_empty() {
                    None
                } else {
                    Some(all)
                }
            }
            HirExprKind::Array(xs) => {
                for x in xs {
                    self.eval(x);
                }
                None
            }
            HirExprKind::Unary { expr, .. } => {
                self.eval(expr);
                None
            }
            HirExprKind::Binary { lhs, rhs, .. } => {
                self.eval(lhs);
                self.eval(rhs);
                None
            }
            HirExprKind::Assign { target, value, .. } => {
                self.eval(value);
                self.eval(target);
                None
            }
            HirExprKind::Lit(_)
            | HirExprKind::Const(_)
            | HirExprKind::Fn(_)
            | HirExprKind::Builtin(_)
            | HirExprKind::Error => None,
        }
    }

    fn call(
        &mut self,
        callee: &HirExpr,
        generics: &[gala_ast::TypeArg],
        args: &[HirExpr],
        span: Span,
    ) -> Option<Vec<u32>> {
        match &callee.kind {
            HirExprKind::Builtin(Builtin::Qubit) => {
                let l = self.fresh("ancilla".into());
                self.events.push(Event::Alloc { lineage: l });
                Some(vec![l])
            }
            HirExprKind::Builtin(Builtin::Qubits) => {
                let n = register_size(generics);
                let mut out = Vec::new();
                for i in 0..n {
                    let l = self.fresh(format!("ancilla[{i}]"));
                    self.events.push(Event::Alloc { lineage: l });
                    out.push(l);
                }
                Some(out)
            }
            HirExprKind::Builtin(b) if b.is_gate() => {
                let mut lines = Vec::new();
                for a in args {
                    lines.extend(self.eval(a).unwrap_or_default());
                }
                if !lines.is_empty() {
                    self.events.push(Event::Gate { lineages: lines.clone() });
                }
                Some(lines)
            }
            HirExprKind::Builtin(Builtin::Measure) => {
                for a in args {
                    for l in self.eval(a).unwrap_or_default() {
                        self.events.push(Event::Measure { lineage: l });
                    }
                }
                None
            }
            HirExprKind::Builtin(Builtin::Drop) => {
                for a in args {
                    for l in self.eval(a).unwrap_or_default() {
                        self.check_drop(l, span);
                    }
                }
                None
            }
            HirExprKind::Fn(def) => {
                let func = self.pkg.func(*def);
                let name = func.map(|f| f.name.as_str().to_string()).unwrap_or_default();
                for a in args {
                    for l in self.eval(a).unwrap_or_default() {
                        // Passing a qubit to a user function hides what it does to it.
                        self.events.push(Event::Opaque { lineage: l, callee: name.clone() });
                    }
                }
                // A qubit returned by the callee has no allocation here, so its history is unknown.
                let count = match func.and_then(|f| f.ret_ty.as_ref()) {
                    Some(gala_ast::Type::Qubit) => 1,
                    Some(gala_ast::Type::Qubits(n)) => {
                        gala_hir::const_int(n).unwrap_or(0).max(0) as usize
                    }
                    _ => 0,
                };
                if count == 0 {
                    return None;
                }
                Some((0..count).map(|_| self.fresh(format!("result of `{name}`"))).collect())
            }
            _ => {
                for a in args {
                    self.eval(a);
                }
                None
            }
        }
    }

    /// Decide whether the lineage `x` can be released now.
    fn check_drop(&mut self, x: u32, span: Span) {
        self.summary.drops_checked += 1;
        let alloc_idx = self
            .events
            .iter()
            .position(|ev| matches!(ev, Event::Alloc { lineage } if *lineage == x));
        let outcome = match alloc_idx {
            None => Err(format!(
                "`{}` was not allocated in this function, so its state is unknown",
                self.lineage_name[x as usize]
            )),
            Some(start) => self.cone(x, start),
        };
        match outcome {
            Ok(cone_events) => {
                self.summary.drops_accepted += 1;
                self.undone.extend(cone_events);
                self.events.push(Event::Release { lineage: x });
            }
            Err(why) => {
                self.diags.push(
                    Diagnostic::error(codes::CANNOT_UNCOMPUTE, "cannot uncompute automatically")
                        .with_primary_label(span, "this ancilla cannot be released")
                        .with_note(why)
                        .with_help("consume the qubits it depends on last, or uncompute by hand"),
                );
            }
        }
    }

    /// Walk the cone of entanglement of `x` since its allocation. Returns the gate events that
    /// the inverse must undo, or the reason the cone cannot be undone.
    fn cone(&self, x: u32, start: usize) -> Result<Vec<usize>, String> {
        let mut cone: HashSet<u32> = HashSet::from([x]);
        let mut gates = Vec::new();
        for (k, ev) in self.events.iter().enumerate().skip(start) {
            if self.undone.contains(&k) {
                continue;
            }
            match ev {
                Event::Gate { lineages } => {
                    if lineages.iter().any(|l| cone.contains(l)) {
                        cone.extend(lineages.iter().copied());
                        gates.push(k);
                    }
                }
                Event::Opaque { lineage, callee } => {
                    if cone.contains(lineage) {
                        return Err(format!(
                            "`{}` was passed through a call to `{callee}`, whose operations are not visible here",
                            self.lineage_name[*lineage as usize]
                        ));
                    }
                }
                Event::Measure { lineage } | Event::Release { lineage } => {
                    if *lineage != x && cone.contains(lineage) {
                        let name = &self.lineage_name[*lineage as usize];
                        let what = if matches!(ev, Event::Measure { .. }) {
                            "measured"
                        } else {
                            "released"
                        };
                        return Err(format!(
                            "`{}` is entangled with `{name}`, which was {what}, so the inverse circuit cannot be applied",
                            self.lineage_name[x as usize]
                        ));
                    }
                }
                Event::Alloc { .. } => {}
            }
        }
        Ok(gates)
    }
}

/// Register size for `qubits<N>()` from the call's explicit generic argument (checked upstream).
fn register_size(generics: &[gala_ast::TypeArg]) -> usize {
    match generics.first() {
        Some(gala_ast::TypeArg::Const(c)) => gala_hir::const_int(c).unwrap_or(0).max(0) as usize,
        _ => 0,
    }
}

fn literal(e: &HirExpr) -> Option<i64> {
    match &e.kind {
        HirExprKind::Lit(gala_ast::Literal::Int(v)) => Some(*v),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gala_hir::lower_file;
    use gala_parser::parse_source;
    use gala_span::FileId;

    fn run(src: &str) -> (UncomputeSummary, Vec<String>) {
        let out = parse_source(FileId(0), src);
        assert!(!out.diagnostics.has_errors(), "parse: {:?}", out.diagnostics);
        let (pkg, _) = lower_file(FileId(0), out.items);
        let (info, _) = gala_types::check_package(&pkg);
        let (summary, diags) = analyze(&pkg, &info);
        (summary, diags.errors().iter().map(|d| d.code.to_string()).collect())
    }

    #[test]
    fn unentangled_ancilla_is_accepted() {
        let (s, errs) = run("fn f() quantum { let a = qubit()\n h(a)\n h(a)\n drop(a) }");
        assert!(errs.is_empty(), "{errs:?}");
        assert_eq!((s.drops_checked, s.drops_accepted), (1, 1));
    }

    #[test]
    fn ancilla_entangled_with_live_qubit_is_accepted() {
        // The partner `d` is still live at the drop, so the inverse can be applied.
        let (_, errs) = run("fn f(d: Qubit) -> Qubit quantum {\n let a = qubit()\n cx(d, a)\n cx(d, a)\n drop(a)\n d }");
        assert!(errs.is_empty(), "{errs:?}");
    }

    #[test]
    fn partner_measured_before_drop_is_e0530() {
        let (_, errs) = run(
            "fn f(d: Qubit) -> Measured<Bool> prob {\n let a = qubit()\n cx(d, a)\n let r = measure(d)\n drop(a)\n r }",
        );
        assert_eq!(errs, vec!["E0530".to_string()]);
    }

    #[test]
    fn passing_through_user_call_is_e0530() {
        let (_, errs) = run(
            "fn g(q: Qubit) -> Qubit quantum { q }\nfn f() quantum {\n let a = qubit()\n let b = g(a)\n drop(b) }",
        );
        assert!(errs.contains(&"E0530".to_string()), "{errs:?}");
    }

    #[test]
    fn register_drop_counts_each_element() {
        let (s, errs) = run("fn f() quantum { let q = qubits<3>()\n drop(q) }");
        assert!(errs.is_empty(), "{errs:?}");
        assert_eq!(s.drops_checked, 3);
    }
}
