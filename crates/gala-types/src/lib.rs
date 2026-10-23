//! Type and effect checking for Gala's HIR.
//!
//! Three things are checked here, in order:
//! 1. **Types.** Classical types (`Int`, `Float`, `Bool`, tuples, `Vec`) and quantum types
//!    (`Qubit`, `Qubits<N>`, `Measured<T>`). Register sizes are checked where they are known.
//! 2. **Effects.** Each function's effect is the join of the operations it performs, on the
//!    lattice `pure ⊑ quantum ⊑ prob`. A declared effect must cover the body, and a call may only
//!    reach a callee whose effect is at or below the caller's.
//! 3. **Linearity** (see [`linearity`]): qubits are used exactly once.

pub mod linearity;

use gala_ast::{self as ast, BinOp, ConstExpr, Effect as AstEffect, Literal, TypeArg, UnOp};
use gala_diagnostics::{codes, Diagnostic, DiagnosticCode, Diagnostics};
use gala_hir::{
    Builtin, DefId, ExprId, HirBlock, HirExpr, HirExprKind, HirPackage, HirPat, HirStmt, LocalId,
};
use gala_span::Span;
use std::collections::HashMap;
use std::fmt;

pub use linearity::check_linearity;

/// A Gala type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Int,
    Float,
    Bool,
    Complex,
    Str,
    Unit,
    /// A single linear qubit.
    Qubit,
    /// A linear register of `N` qubits.
    Qubits(u64),
    /// A classical result carrying its distribution.
    Measured(Box<Ty>),
    /// A growable classical sequence (`Vec<T>` and fixed arrays).
    Vec(Box<Ty>),
    Tuple(Vec<Ty>),
    /// Trainable parameter bundle.
    Params,
    /// The type of `a..b`.
    Range,
    /// A quantum backend (see `backend.simulator()`).
    Backend,
    /// A type that could not be determined; suppresses cascading errors.
    Error,
}

impl Ty {
    /// True when values of this type are linear (may not be copied or implicitly dropped).
    pub fn is_linear(&self) -> bool {
        match self {
            Ty::Qubit | Ty::Qubits(_) => true,
            Ty::Tuple(items) => items.iter().any(Ty::is_linear),
            Ty::Vec(inner) | Ty::Measured(inner) => inner.is_linear(),
            _ => false,
        }
    }

    /// True for `Int` and `Float`.
    pub fn is_numeric(&self) -> bool {
        matches!(self, Ty::Int | Ty::Float)
    }
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Int => write!(f, "Int"),
            Ty::Float => write!(f, "Float"),
            Ty::Bool => write!(f, "Bool"),
            Ty::Complex => write!(f, "Complex"),
            Ty::Str => write!(f, "String"),
            Ty::Unit => write!(f, "Unit"),
            Ty::Qubit => write!(f, "Qubit"),
            Ty::Qubits(n) => write!(f, "Qubits<{n}>"),
            Ty::Measured(t) => write!(f, "Measured<{t}>"),
            Ty::Vec(t) => write!(f, "Vec<{t}>"),
            Ty::Tuple(items) => {
                let parts: Vec<String> = items.iter().map(|t| t.to_string()).collect();
                write!(f, "({})", parts.join(", "))
            }
            Ty::Params => write!(f, "Params"),
            Ty::Range => write!(f, "Range<Int>"),
            Ty::Backend => write!(f, "Backend"),
            Ty::Error => write!(f, "{{error}}"),
        }
    }
}

/// An effect on the lattice `pure ⊑ quantum ⊑ prob`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Effect {
    /// Deterministic classical computation.
    Pure,
    /// Unitary evolution; no measurement.
    Quantum,
    /// Performs measurement; classical randomness.
    Prob,
}

impl Effect {
    /// The effect written in source, if any.
    pub fn from_ast(e: AstEffect) -> Effect {
        match e {
            AstEffect::Pure => Effect::Pure,
            AstEffect::Quantum => Effect::Quantum,
            AstEffect::Prob => Effect::Prob,
        }
    }

    /// Lattice join.
    pub fn join(self, other: Effect) -> Effect {
        self.max(other)
    }
}

impl fmt::Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Effect::Pure => write!(f, "pure"),
            Effect::Quantum => write!(f, "quantum"),
            Effect::Prob => write!(f, "prob"),
        }
    }
}

/// The checked result for a package: types of every expression and local, and effects of every
/// function.
#[derive(Debug, Clone, Default)]
pub struct TypeInfo {
    /// Type of each expression.
    pub expr_ty: HashMap<ExprId, Ty>,
    /// Type of each local binding.
    pub local_ty: HashMap<LocalId, Ty>,
    /// Parameter and return types per function, indexed by `DefId`.
    pub sigs: Vec<FnSig>,
    /// Effective effect per function (declared, or inferred when not declared).
    pub effects: Vec<Effect>,
    /// Inferred effect per function, before any declaration.
    pub inferred: Vec<Effect>,
    /// Type of each top-level constant.
    pub const_ty: Vec<Ty>,
}

impl TypeInfo {
    /// Type of an expression, or `Ty::Error` when unknown.
    pub fn ty(&self, id: ExprId) -> &Ty {
        self.expr_ty.get(&id).unwrap_or(&Ty::Error)
    }
}

/// Parameter and return types of a function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnSig {
    pub params: Vec<Ty>,
    pub ret: Ty,
}

/// Type-check a package. Returns the type information and all diagnostics.
pub fn check_package(pkg: &HirPackage) -> (TypeInfo, Diagnostics) {
    // Phase A: signatures.
    let mut sigs = Vec::with_capacity(pkg.fns.len());
    let mut diags = Diagnostics::new();
    for f in &pkg.fns {
        // A const-generic declaration has no value for its size parameter until it is
        // instantiated; each instance gets its own signature in the HIR.
        if f.generics.iter().any(|g| matches!(g, ast::GenericParam::Const { .. })) {
            sigs.push(FnSig { params: Vec::new(), ret: Ty::Unit });
            continue;
        }
        let params: Vec<Ty> = f.params.iter().map(|p| ast_ty(&p.ty, &mut diags, p.span)).collect();
        let ret = match &f.ret_ty {
            Some(t) => ast_ty(t, &mut diags, f.span),
            None => Ty::Unit,
        };
        if !f.generics.is_empty() {
            diags.push(
                Diagnostic::error(
                    codes::UNSUPPORTED_FEATURE,
                    "generic functions are not supported yet",
                )
                .with_primary_label(f.span, "generic parameters here")
                .with_help("write concrete register sizes such as `Qubits<4>`"),
            );
        }
        sigs.push(FnSig { params, ret });
    }

    let mut info = TypeInfo { sigs, ..TypeInfo::default() };
    info.effects = vec![Effect::Pure; pkg.fns.len()];
    info.inferred = vec![Effect::Pure; pkg.fns.len()];

    // Constants are classical in this version; check them once to learn their types.
    info.const_ty = vec![Ty::Error; pkg.consts.len()];
    for (i, c) in pkg.consts.iter().enumerate() {
        let mut ck = Checker::new(pkg, &mut info, false);
        let ty = ck.infer(&c.value);
        info.const_ty[i] = ty;
    }

    // Phase B: effect fixpoint. Inferred effects only grow, so this terminates.
    for _ in 0..(pkg.fns.len() + 2) {
        let mut changed = false;
        for (i, f) in pkg.fns.iter().enumerate() {
            let inferred = match &f.body {
                Some(body) => {
                    let mut ck = Checker::new(pkg, &mut info, false);
                    ck.check_fn_body(f.def, body);
                    ck.eff
                }
                None => intrinsic_effect(f.intrinsic),
            };
            if inferred != info.inferred[i] {
                info.inferred[i] = inferred;
                changed = true;
            }
            let effective = match f.declared_effect {
                Some(d) => Effect::from_ast(d),
                None => inferred,
            };
            if effective != info.effects[i] {
                info.effects[i] = effective;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    // Phase C: report errors using the settled effects.
    let mut runs = Vec::new();
    for f in &pkg.fns {
        if let Some(body) = &f.body {
            let sig = info.sigs[f.def.0 as usize].clone();
            let mut ck = Checker::new(pkg, &mut info, true);
            ck.declared = f.declared_effect.map(Effect::from_ast);
            ck.current_ret = sig.ret.clone();
            for (p, sig_ty) in f.params.iter().zip(sig.params) {
                ck.info.local_ty.insert(p.local, sig_ty);
            }
            ck.check_fn_body(f.def, body);
            diags.extend(std::mem::take(&mut ck.diags));
            runs.extend(std::mem::take(&mut ck.runs));
        }
    }

    // A base backend cannot run a program that branches on a measurement result (E0601).
    for (target, kind, span) in runs {
        if kind == Some(BackendKind::Base) && branches_on_measurement(pkg, &info, target) {
            diags.push(
                Diagnostic::error(
                    codes::BACKEND_CAPABILITY_MISMATCH,
                    format!(
                        "`{}` branches on a measurement, which the base-profile backend does not support",
                        pkg.fns[target].name
                    ),
                )
                .with_primary_label(span, "run on this backend")
                .with_help("use backend.simulator(), which supports mid-circuit measurement"),
            );
        }
    }

    (info, diags)
}

/// Effect of a compiler intrinsic.
fn intrinsic_effect(b: Option<Builtin>) -> Effect {
    match b {
        Some(Builtin::Measure) => Effect::Prob,
        Some(Builtin::Print) | Some(Builtin::Clone) | None => Effect::Pure,
        Some(_) => Effect::Quantum,
    }
}

/// Convert an AST type to a checked type. Unknown names are reported.
pub fn ast_ty(t: &ast::Type, diags: &mut Diagnostics, span: Span) -> Ty {
    match t {
        ast::Type::Qubit => Ty::Qubit,
        ast::Type::Qubits(size) => match gala_hir::const_int(size) {
            Some(n) if n >= 0 => Ty::Qubits(n as u64),
            _ => {
                diags.push(
                    Diagnostic::error(
                        codes::UNSUPPORTED_FEATURE,
                        "register size must be a literal integer",
                    )
                    .with_primary_label(span, "size is not a literal")
                    .with_help("generic register sizes are not supported yet; use `Qubits<4>`"),
                );
                Ty::Error
            }
        },
        ast::Type::Measured(inner) => Ty::Measured(Box::new(ast_ty(inner, diags, span))),
        ast::Type::Tuple(items) if items.is_empty() => Ty::Unit,
        ast::Type::Tuple(items) => {
            Ty::Tuple(items.iter().map(|i| ast_ty(i, diags, span)).collect())
        }
        ast::Type::Array(elem, _) => Ty::Vec(Box::new(ast_ty(elem, diags, span))),
        ast::Type::Named(name, args) => named_ty(name, args, diags, span),
        ast::Type::Path(path) => {
            let seg = &path.segments[0];
            let args: Vec<ast::Type> = seg
                .type_args
                .iter()
                .filter_map(|a| if let TypeArg::Type(t) = a { Some(t.clone()) } else { None })
                .collect();
            named_ty(seg.ident.as_str(), &args, diags, span)
        }
        ast::Type::Fn { .. } => {
            diags.push(
                Diagnostic::error(
                    codes::UNSUPPORTED_FEATURE,
                    "function types are not supported yet",
                )
                .with_primary_label(span, "here"),
            );
            Ty::Error
        }
    }
}

fn named_ty(name: &str, args: &[ast::Type], diags: &mut Diagnostics, span: Span) -> Ty {
    match name {
        "Int" => Ty::Int,
        "Float" => Ty::Float,
        "Bool" => Ty::Bool,
        "Complex" => Ty::Complex,
        "String" => Ty::Str,
        "Unit" => Ty::Unit,
        "Params" => Ty::Params,
        "Qubit" => Ty::Qubit,
        "Vec" => match args.first() {
            Some(t) => Ty::Vec(Box::new(ast_ty(t, diags, span))),
            None => {
                diags.push(
                    Diagnostic::error(codes::TYPE_MISMATCH, "`Vec` needs an element type")
                        .with_primary_label(span, "write `Vec<T>`"),
                );
                Ty::Error
            }
        },
        other => {
            diags.push(
                Diagnostic::error(codes::UNRESOLVED_NAME, format!("unknown type `{other}`"))
                    .with_primary_label(span, "not a type")
                    .with_help("the built-in types are Int, Float, Bool, Complex, String, Unit, Qubit, Qubits<N>, Measured<T>, Vec<T>"),
            );
            Ty::Error
        }
    }
}

/// The kind of backend a program targets, as written by the constructor call.
#[derive(Debug, Clone, PartialEq)]
pub enum BackendKind {
    Simulator,
    /// Simulator restricted to base-profile programs.
    Base,
    Noisy,
    Hardware,
}

fn backend_kind_of(b: Builtin) -> Option<BackendKind> {
    match b {
        Builtin::BackendSimulator => Some(BackendKind::Simulator),
        Builtin::BackendBase => Some(BackendKind::Base),
        Builtin::BackendNoisy => Some(BackendKind::Noisy),
        Builtin::BackendHardware => Some(BackendKind::Hardware),
        _ => None,
    }
}

/// Per-function checking state.
struct Checker<'p, 'i> {
    pkg: &'p HirPackage,
    info: &'i mut TypeInfo,
    /// Whether to record diagnostics (false during the effect fixpoint).
    report: bool,
    diags: Diagnostics,
    /// Effect accumulated so far in the current body.
    eff: Effect,
    /// Declared effect of the current function, if any.
    declared: Option<Effect>,
    /// Declared return type of the current function.
    current_ret: Ty,
    /// Mutability of each local, for assignment checks.
    mutable: HashMap<LocalId, bool>,
    /// Backend kind bound to each local initialized by a backend constructor.
    backend_kind: HashMap<LocalId, BackendKind>,
    /// `run` calls to check once every function's types are known: (target, backend, span).
    runs: Vec<(usize, Option<BackendKind>, Span)>,
}

impl<'p, 'i> Checker<'p, 'i> {
    fn new(pkg: &'p HirPackage, info: &'i mut TypeInfo, report: bool) -> Self {
        Checker {
            pkg,
            info,
            report,
            diags: Diagnostics::new(),
            eff: Effect::Pure,
            declared: None,
            current_ret: Ty::Unit,
            mutable: HashMap::new(),
            backend_kind: HashMap::new(),
            runs: Vec::new(),
        }
    }

    fn error(&mut self, code: DiagnosticCode, msg: impl Into<String>, span: Span, label: &str) {
        if self.report {
            self.diags.push(Diagnostic::error(code, msg).with_primary_label(span, label));
        }
    }

    fn check_fn_body(&mut self, def: DefId, body: &HirBlock) {
        let f = &self.pkg.fns[def.0 as usize];
        if self.current_ret == Ty::Unit && f.ret_ty.is_none() {
            self.current_ret = Ty::Unit;
        }
        self.eff = Effect::Pure;
        if !self.report {
            // Parameter types are needed during the fixpoint too.
            let sig = self.info.sigs[def.0 as usize].clone();
            for (p, t) in f.params.iter().zip(sig.params) {
                self.info.local_ty.insert(p.local, t);
            }
            self.current_ret = sig.ret;
        }
        for p in &f.params {
            self.mutable.insert(p.local, false);
        }
        let ty = self.block(body);
        let ret = self.current_ret.clone();
        self.expect(&ty, &ret, body.span, "function body");
    }

    fn expect(&mut self, actual: &Ty, expected: &Ty, span: Span, what: &str) {
        if actual == expected || *actual == Ty::Error || *expected == Ty::Error {
            return;
        }
        // Integer literals are accepted where a Float is expected (angles, for example).
        if *expected == Ty::Float && *actual == Ty::Int {
            return;
        }
        // A tuple of `n` qubits is a register of `n` qubits (the spec's Bell pair returns `(a, b)`
        // from a function declared `-> Qubits<2>`).
        if let (Ty::Qubits(n), Ty::Tuple(items)) = (expected, actual) {
            if items.len() as u64 == *n && items.iter().all(|t| *t == Ty::Qubit) {
                return;
            }
        }
        self.error(
            codes::TYPE_MISMATCH,
            format!("mismatched types: expected `{expected}`, found `{actual}`"),
            span,
            &format!("{what} has type `{actual}`"),
        );
    }

    /// A condition is a `Bool`, or a measurement result (`Measured<Bool>`), which branches on the
    /// sampled value (mid-circuit measurement).
    fn condition(&mut self, c: &Ty, span: Span) {
        if *c == Ty::Measured(Box::new(Ty::Bool)) {
            return;
        }
        self.expect(c, &Ty::Bool, span, "condition");
    }

    fn record(&mut self, id: ExprId, ty: Ty) -> Ty {
        self.info.expr_ty.insert(id, ty.clone());
        ty
    }

    fn block(&mut self, b: &HirBlock) -> Ty {
        for stmt in &b.stmts {
            self.stmt(stmt);
        }
        match &b.tail {
            Some(t) => self.infer(t),
            // A block that ends in `return` never falls through, so it has no type to compare.
            None if matches!(b.stmts.last(), Some(HirStmt::Return { .. })) => Ty::Error,
            None => Ty::Unit,
        }
    }

    fn stmt(&mut self, s: &HirStmt) {
        match s {
            HirStmt::Let { pat, ty, init, span } => {
                let init_ty = self.infer(init);
                if let (HirPat::Bind { local, .. }, HirExprKind::Call { callee, .. }) =
                    (pat, &init.kind)
                {
                    if let HirExprKind::Builtin(b) = callee.kind {
                        if let Some(kind) = backend_kind_of(b) {
                            self.backend_kind.insert(*local, kind);
                        }
                    }
                }
                let bound = match ty {
                    Some(t) => {
                        let declared = ast_ty(t, &mut self.diags, *span);
                        self.expect(&init_ty, &declared, *span, "initializer");
                        declared
                    }
                    None => init_ty,
                };
                self.bind(pat, &bound, *span);
            }
            HirStmt::Expr(e) => {
                self.infer(e);
            }
            HirStmt::Return { value, span } => {
                let ty = match value {
                    Some(v) => self.infer(v),
                    None => Ty::Unit,
                };
                let ret = self.current_ret.clone();
                self.expect(&ty, &ret, *span, "returned value");
            }
        }
    }

    fn bind(&mut self, pat: &HirPat, ty: &Ty, span: Span) {
        match pat {
            HirPat::Bind { local, mutable, .. } => {
                self.info.local_ty.insert(*local, ty.clone());
                self.mutable.insert(*local, *mutable);
            }
            HirPat::Wild | HirPat::Lit(_, _) => {}
            HirPat::Tuple(items, _) => match ty {
                Ty::Tuple(tys) if tys.len() == items.len() => {
                    for (p, t) in items.iter().zip(tys.clone()) {
                        self.bind(p, &t, span);
                    }
                }
                Ty::Error => {
                    for p in items {
                        self.bind(p, &Ty::Error, span);
                    }
                }
                other => {
                    self.error(
                        codes::TYPE_MISMATCH,
                        format!("tuple pattern cannot match `{other}`"),
                        span,
                        "pattern does not match this type",
                    );
                    for p in items {
                        self.bind(p, &Ty::Error, span);
                    }
                }
            },
        }
    }

    fn infer(&mut self, e: &HirExpr) -> Ty {
        let ty = self.infer_inner(e);
        self.record(e.id, ty)
    }

    fn infer_inner(&mut self, e: &HirExpr) -> Ty {
        match &e.kind {
            HirExprKind::Lit(l) => match l {
                Literal::Int(_) => Ty::Int,
                Literal::Float(_) => Ty::Float,
                Literal::Complex { .. } => Ty::Complex,
                Literal::Bool(_) => Ty::Bool,
                Literal::String(_) => Ty::Str,
                Literal::Unit => Ty::Unit,
            },
            HirExprKind::Local(l, _) => self.info.local_ty.get(l).cloned().unwrap_or(Ty::Error),
            HirExprKind::Const(idx) => self.info.const_ty.get(*idx).cloned().unwrap_or(Ty::Error),
            HirExprKind::Fn(_) | HirExprKind::Builtin(_) => {
                self.error(
                    codes::UNSUPPORTED_FEATURE,
                    "functions cannot be used as values yet",
                    e.span,
                    "call it with `(...)`",
                );
                Ty::Error
            }
            HirExprKind::Call { callee, generics, args, names } => {
                self.call(callee, generics, args, names, e.span)
            }
            HirExprKind::Unary { op, expr } => {
                let t = self.infer(expr);
                match op {
                    UnOp::Neg if t.is_numeric() || t == Ty::Error => t,
                    UnOp::Not if t == Ty::Bool || t == Ty::Error => Ty::Bool,
                    _ => {
                        self.error(
                            codes::TYPE_MISMATCH,
                            format!("operator cannot be applied to `{t}`"),
                            e.span,
                            "here",
                        );
                        Ty::Error
                    }
                }
            }
            HirExprKind::Binary { op, lhs, rhs } => {
                let l = self.infer(lhs);
                let r = self.infer(rhs);
                self.binary(*op, l, r, e.span)
            }
            HirExprKind::If { cond, then_block, else_expr } => {
                let c = self.infer(cond);
                self.condition(&c, cond.span);
                let then_ty = self.block(then_block);
                match else_expr {
                    None => Ty::Unit,
                    Some(else_e) => {
                        let else_ty = self.infer(else_e);
                        if then_ty == Ty::Error || else_ty == Ty::Error {
                            return Ty::Error;
                        }
                        if then_ty != else_ty {
                            self.error(
                                codes::TYPE_MISMATCH,
                                format!("`if` branches have different types: `{then_ty}` and `{else_ty}`"),
                                e.span,
                                "branches disagree",
                            );
                            return Ty::Error;
                        }
                        then_ty
                    }
                }
            }
            HirExprKind::For { pat, iterable, body } => {
                let it = self.infer(iterable);
                let elem = match it {
                    Ty::Range => Ty::Int,
                    Ty::Vec(t) => *t,
                    Ty::Error => Ty::Error,
                    other => {
                        self.error(
                            codes::TYPE_MISMATCH,
                            format!("`for` cannot iterate over `{other}`"),
                            iterable.span,
                            "not iterable",
                        );
                        Ty::Error
                    }
                };
                self.bind(pat, &elem, e.span);
                self.block(body);
                Ty::Unit
            }
            HirExprKind::While { cond, body } => {
                let c = self.infer(cond);
                self.condition(&c, cond.span);
                self.block(body);
                Ty::Unit
            }
            HirExprKind::Block(b) => self.block(b),
            HirExprKind::Tuple(items) => {
                let tys: Vec<Ty> = items.iter().map(|i| self.infer(i)).collect();
                Ty::Tuple(tys)
            }
            HirExprKind::Array(items) => {
                let tys: Vec<Ty> = items.iter().map(|i| self.infer(i)).collect();
                match tys.first() {
                    Some(first) => Ty::Vec(Box::new(first.clone())),
                    None => Ty::Vec(Box::new(Ty::Error)),
                }
            }
            HirExprKind::Index { base, index } => {
                let b = self.infer(base);
                let i = self.infer(index);
                self.expect(&i, &Ty::Int, index.span, "index");
                match b {
                    Ty::Qubits(n) => {
                        if let Some(k) = literal_int(index) {
                            if k < 0 || k as u64 >= n {
                                self.error(
                                    codes::QUBITS_SIZE_MISMATCH,
                                    format!("index {k} is out of range for `Qubits<{n}>`"),
                                    index.span,
                                    "out of range",
                                );
                            }
                        }
                        Ty::Qubit
                    }
                    Ty::Vec(t) => *t,
                    Ty::Error => Ty::Error,
                    other => {
                        self.error(
                            codes::TYPE_MISMATCH,
                            format!("`{other}` cannot be indexed"),
                            base.span,
                            "not indexable",
                        );
                        Ty::Error
                    }
                }
            }
            HirExprKind::Assign { target, op, value } => {
                let t = self.infer(target);
                let v = self.infer(value);
                if let HirExprKind::Local(l, name) = &target.kind {
                    if self.mutable.get(l) == Some(&false) {
                        self.error(
                            codes::IMMUTABLE_ASSIGN,
                            format!("cannot assign twice to immutable binding `{name}`"),
                            e.span,
                            "not declared `mut`",
                        );
                    }
                }
                match op {
                    Some(bop) => {
                        let r = self.binary(*bop, t.clone(), v, e.span);
                        self.expect(&r, &t, e.span, "result");
                    }
                    None => self.expect(&v, &t, e.span, "assigned value"),
                }
                Ty::Unit
            }
            HirExprKind::Match { scrutinee, arms } => {
                let s = self.infer(scrutinee);
                let mut result: Option<Ty> = None;
                for arm in arms {
                    self.bind(&arm.pat, &s, e.span);
                    if let HirPat::Lit(l, span) = &arm.pat {
                        let lit_ty = match l {
                            Literal::Int(_) => Ty::Int,
                            Literal::Bool(_) => Ty::Bool,
                            _ => Ty::Error,
                        };
                        self.expect(&lit_ty, &s, *span, "pattern");
                    }
                    if let Some(g) = &arm.guard {
                        let gt = self.infer(g);
                        self.expect(&gt, &Ty::Bool, g.span, "guard");
                    }
                    let bt = self.infer(&arm.body);
                    match &result {
                        None => result = Some(bt),
                        Some(prev) => {
                            if *prev != bt && bt != Ty::Error && *prev != Ty::Error {
                                self.error(
                                    codes::TYPE_MISMATCH,
                                    format!("match arms have different types: `{prev}` and `{bt}`"),
                                    arm.body.span,
                                    "arm type differs",
                                );
                            }
                        }
                    }
                }
                result.unwrap_or(Ty::Unit)
            }
            HirExprKind::Error => Ty::Error,
        }
    }

    fn binary(&mut self, op: BinOp, l: Ty, r: Ty, span: Span) -> Ty {
        if l == Ty::Error || r == Ty::Error {
            return match op {
                BinOp::Eq
                | BinOp::Ne
                | BinOp::Lt
                | BinOp::Le
                | BinOp::Gt
                | BinOp::Ge
                | BinOp::And
                | BinOp::Or => Ty::Bool,
                _ => Ty::Error,
            };
        }
        let numeric_pair =
            (l.is_numeric() && r.is_numeric()) && (l == r || l == Ty::Float || r == Ty::Float);
        let bad = |this: &mut Self| {
            this.error(
                codes::TYPE_MISMATCH,
                format!("operator cannot combine `{l}` and `{r}`"),
                span,
                "invalid operand types",
            );
            Ty::Error
        };
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                if numeric_pair {
                    if l == Ty::Float || r == Ty::Float {
                        Ty::Float
                    } else {
                        Ty::Int
                    }
                } else {
                    bad(self)
                }
            }
            BinOp::Mod => {
                if l == Ty::Int && r == Ty::Int {
                    Ty::Int
                } else {
                    bad(self)
                }
            }
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                if numeric_pair {
                    Ty::Bool
                } else {
                    bad(self)
                }
            }
            BinOp::Eq | BinOp::Ne => {
                if numeric_pair || (l == r && !l.is_linear()) {
                    Ty::Bool
                } else {
                    bad(self)
                }
            }
            BinOp::And | BinOp::Or => {
                if l == Ty::Bool && r == Ty::Bool {
                    Ty::Bool
                } else {
                    bad(self)
                }
            }
            BinOp::Range => {
                if l == Ty::Int && r == Ty::Int {
                    Ty::Range
                } else {
                    bad(self)
                }
            }
        }
    }

    fn call(
        &mut self,
        callee: &HirExpr,
        generics: &[TypeArg],
        args: &[HirExpr],
        names: &[Option<ast::Ident>],
        span: Span,
    ) -> Ty {
        match &callee.kind {
            HirExprKind::Builtin(Builtin::Run) => self.run_call(args, names, span),
            HirExprKind::Builtin(b) => self.builtin_call(*b, generics, args, span),
            HirExprKind::Fn(def) => {
                let def = *def;
                let f = &self.pkg.fns[def.0 as usize];
                let sig = self.info.sigs[def.0 as usize].clone();
                let callee_eff = self.info.effects[def.0 as usize];
                let name = f.name.as_str().to_string();
                if sig.params.len() != args.len() {
                    self.error(
                        codes::ARITY_MISMATCH,
                        format!(
                            "`{name}` takes {} arguments but {} were supplied",
                            sig.params.len(),
                            args.len()
                        ),
                        span,
                        "wrong number of arguments",
                    );
                }
                let param_names: Vec<String> = f.params.iter().map(|p| p.name.clone()).collect();
                for (i, arg) in args.iter().enumerate() {
                    let at = self.infer(arg);
                    let want = sig.params.get(i).cloned().unwrap_or(Ty::Error);
                    // A named argument must name the parameter in that position.
                    if let Some(Some(given)) = names.get(i) {
                        if param_names.get(i).map(String::as_str) != Some(given.as_str()) {
                            self.error(
                                codes::ARITY_MISMATCH,
                                format!(
                                    "`{name}` has no parameter named `{}` at this position",
                                    given
                                ),
                                arg.span,
                                "argument name",
                            );
                        }
                    }
                    self.expect(&at, &want, arg.span, "argument");
                }
                self.account_call(callee_eff, None, &name, span);
                sig.ret
            }
            _ => {
                self.error(
                    codes::TYPE_MISMATCH,
                    "this expression cannot be called",
                    callee.span,
                    "not a function",
                );
                for a in args {
                    self.infer(a);
                }
                Ty::Error
            }
        }
    }

    /// Account for a call to something with `callee_eff`: join into the body's effect, and report
    /// a violation of the caller's declared effect.
    fn account_call(
        &mut self,
        callee_eff: Effect,
        marker: Option<Builtin>,
        name: &str,
        span: Span,
    ) {
        self.eff = self.eff.join(callee_eff);
        let Some(declared) = self.declared else { return };
        if callee_eff <= declared {
            return;
        }
        let (code, msg) = match (callee_eff, declared) {
            (Effect::Prob, Effect::Quantum) => (
                codes::QUANTUM_CALLS_PROB,
                format!("`quantum` function cannot call `{name}`, which is `prob`"),
            ),
            (Effect::Quantum, Effect::Pure) => (
                codes::PURE_CALLS_QUANTUM,
                format!("`pure` function cannot call `{name}`, which is `quantum`"),
            ),
            (Effect::Prob, Effect::Pure) => (
                codes::EFFECT_MISMATCH,
                format!("`pure` function cannot call `{name}`, which is `prob`"),
            ),
            _ => (
                codes::EFFECT_MISMATCH,
                format!("`{name}` has effect `{callee_eff}`, not allowed here"),
            ),
        };
        let code = if marker == Some(Builtin::Measure) && declared != Effect::Prob {
            codes::MEASUREMENT_IN_QUANTUM
        } else {
            code
        };
        self.error(
            code,
            msg,
            span,
            &format!("`{name}` is `{callee_eff}`, this function is `{declared}`"),
        );
    }

    fn builtin_call(
        &mut self,
        b: Builtin,
        generics: &[TypeArg],
        args: &[HirExpr],
        span: Span,
    ) -> Ty {
        let arg_tys: Vec<Ty> = args.iter().map(|a| self.infer(a)).collect();
        let name = b.name();
        let expect_args = |this: &mut Self, want: usize| -> bool {
            if arg_tys.len() != want {
                this.error(
                    codes::ARITY_MISMATCH,
                    format!("`{name}` takes {want} arguments but {} were supplied", arg_tys.len()),
                    span,
                    "wrong number of arguments",
                );
                false
            } else {
                true
            }
        };
        match b {
            Builtin::Qubit => {
                expect_args(self, 0);
                self.account_call(Effect::Quantum, Some(b), name, span);
                Ty::Qubit
            }
            Builtin::Qubits => {
                let n = match generics.first() {
                    Some(TypeArg::Const(c)) => match const_value(c) {
                        Some(n) if n >= 0 => Some(n as u64),
                        _ => None,
                    },
                    _ => None,
                };
                let Some(n) = n else {
                    self.error(
                        codes::UNSUPPORTED_FEATURE,
                        "`qubits` needs a literal size, for example `qubits<4>()`",
                        span,
                        "missing size",
                    );
                    return Ty::Error;
                };
                expect_args(self, 0);
                self.account_call(Effect::Quantum, Some(b), name, span);
                Ty::Qubits(n)
            }
            Builtin::H | Builtin::X | Builtin::Y | Builtin::Z | Builtin::S | Builtin::T => {
                if expect_args(self, 1) {
                    self.expect(&arg_tys[0].clone(), &Ty::Qubit, args[0].span, "gate argument");
                }
                self.account_call(Effect::Quantum, Some(b), name, span);
                Ty::Qubit
            }
            Builtin::Rx | Builtin::Ry | Builtin::Rz => {
                if expect_args(self, 2) {
                    self.expect(&arg_tys[0].clone(), &Ty::Qubit, args[0].span, "gate argument");
                    self.expect(&arg_tys[1].clone(), &Ty::Float, args[1].span, "angle");
                }
                self.account_call(Effect::Quantum, Some(b), name, span);
                Ty::Qubit
            }
            Builtin::Cx | Builtin::Cz | Builtin::Swap => {
                if expect_args(self, 2) {
                    self.expect(&arg_tys[0].clone(), &Ty::Qubit, args[0].span, "gate argument");
                    self.expect(&arg_tys[1].clone(), &Ty::Qubit, args[1].span, "gate argument");
                }
                self.account_call(Effect::Quantum, Some(b), name, span);
                Ty::Tuple(vec![Ty::Qubit, Ty::Qubit])
            }
            Builtin::Cp => {
                if expect_args(self, 3) {
                    self.expect(&arg_tys[0].clone(), &Ty::Qubit, args[0].span, "gate argument");
                    self.expect(&arg_tys[1].clone(), &Ty::Qubit, args[1].span, "gate argument");
                    self.expect(&arg_tys[2].clone(), &Ty::Float, args[2].span, "angle");
                }
                self.account_call(Effect::Quantum, Some(b), name, span);
                Ty::Tuple(vec![Ty::Qubit, Ty::Qubit])
            }
            Builtin::Measure => {
                if !expect_args(self, 1) {
                    return Ty::Error;
                }
                self.account_call(Effect::Prob, Some(b), name, span);
                match &arg_tys[0] {
                    Ty::Qubit => Ty::Measured(Box::new(Ty::Bool)),
                    Ty::Qubits(_) => Ty::Measured(Box::new(Ty::Vec(Box::new(Ty::Bool)))),
                    Ty::Error => Ty::Error,
                    other => {
                        self.error(
                            codes::TYPE_MISMATCH,
                            format!("`measure` needs a qubit or register, found `{other}`"),
                            args[0].span,
                            "not a qubit",
                        );
                        Ty::Error
                    }
                }
            }
            Builtin::Drop => {
                if expect_args(self, 1) {
                    match &arg_tys[0] {
                        Ty::Qubit | Ty::Qubits(_) | Ty::Error => {}
                        other => self.error(
                            codes::TYPE_MISMATCH,
                            format!("`drop` needs a qubit or register, found `{other}`"),
                            args[0].span,
                            "not a qubit",
                        ),
                    }
                }
                self.account_call(Effect::Quantum, Some(b), name, span);
                Ty::Unit
            }
            Builtin::Print => {
                let _ = expect_args(self, 1);
                Ty::Unit
            }
            Builtin::Clone => {
                self.error(
                    codes::DUPLICATE_USE,
                    "quantum values cannot be cloned (no-cloning theorem)",
                    span,
                    "`clone` is not allowed",
                );
                Ty::Error
            }
            Builtin::Grad => {
                self.error(
                    codes::UNSUPPORTED_FEATURE,
                    "`grad` is not implemented yet",
                    span,
                    "not supported yet",
                );
                Ty::Error
            }
            Builtin::BackendSimulator | Builtin::BackendBase => {
                expect_args(self, 0);
                Ty::Backend
            }
            Builtin::BackendNoisy => {
                if expect_args(self, 1) {
                    self.expect(&arg_tys[0].clone(), &Ty::Float, args[0].span, "error probability");
                }
                Ty::Backend
            }
            Builtin::BackendHardware => {
                if expect_args(self, 1) {
                    self.expect(&arg_tys[0].clone(), &Ty::Str, args[0].span, "device name");
                }
                Ty::Backend
            }
            // Dispatched in `call`, which needs the argument names.
            Builtin::Run => Ty::Int,
        }
    }

    /// `run(f, on: backend, shots: n)`: `f` is a function name, not a value, and must be a
    /// probabilistic function with no parameters that returns `Measured<Bool>`.
    fn run_call(&mut self, args: &[HirExpr], names: &[Option<ast::Ident>], span: Span) -> Ty {
        let named =
            |n: &str| names.iter().position(|x| x.as_ref().is_some_and(|i| i.as_str() == n));
        let (Some(first), Some(on), Some(shots)) = (args.first(), named("on"), named("shots"))
        else {
            self.error(
                codes::ARITY_MISMATCH,
                "`run` takes a function, `on:` a backend, and `shots:` a count",
                span,
                "missing argument",
            );
            return Ty::Int;
        };
        let HirExprKind::Fn(def) = first.kind else {
            self.error(
                codes::TYPE_MISMATCH,
                "the first argument of `run` must be a function name",
                first.span,
                "not a function",
            );
            return Ty::Int;
        };
        let f = &self.pkg.fns[def.0 as usize];
        let name = f.name.as_str().to_string();
        let sig = self.info.sigs[def.0 as usize].clone();
        if !sig.params.is_empty() {
            self.error(
                codes::ARITY_MISMATCH,
                format!(
                    "`run` takes a function with no parameters, but `{name}` has {}",
                    sig.params.len()
                ),
                first.span,
                "has parameters",
            );
        }
        let measured_bool = Ty::Measured(Box::new(Ty::Bool));
        if sig.ret != measured_bool {
            self.error(
                codes::TYPE_MISMATCH,
                format!(
                    "`run` needs a function returning `Measured<Bool>`, but `{name}` returns `{}`",
                    sig.ret
                ),
                first.span,
                "wrong return type",
            );
        }
        if self.info.effects[def.0 as usize] != Effect::Prob {
            self.error(
                codes::EFFECT_MISMATCH,
                format!("`run` needs a `prob` function, but `{name}` is not"),
                first.span,
                "not probabilistic",
            );
        }
        let on_arg = &args[on];
        let on_ty = self.infer(on_arg);
        if on_ty != Ty::Backend && on_ty != Ty::Error {
            self.error(
                codes::TYPE_MISMATCH,
                format!("`on:` needs a backend, found `{on_ty}`"),
                on_arg.span,
                "not a backend",
            );
        }
        let shots_ty = self.infer(&args[shots]);
        self.expect(&shots_ty, &Ty::Int, args[shots].span, "shot count");
        let kind = match &on_arg.kind {
            HirExprKind::Local(l, _) => self.backend_kind.get(l).cloned(),
            HirExprKind::Call { callee, .. } => match callee.kind {
                HirExprKind::Builtin(b) => backend_kind_of(b),
                _ => None,
            },
            _ => None,
        };
        self.runs.push((def.0 as usize, kind, span));
        self.account_call(Effect::Prob, None, &name, span);
        Ty::Int
    }
}

/// True when a function's body branches on a measurement result (mid-circuit measurement).
fn branches_on_measurement(pkg: &HirPackage, info: &TypeInfo, def: usize) -> bool {
    fn block(b: &HirBlock, info: &TypeInfo) -> bool {
        b.stmts.iter().any(|s| match s {
            HirStmt::Let { init, .. } => expr(init, info),
            HirStmt::Expr(e) => expr(e, info),
            HirStmt::Return { value, .. } => value.as_ref().is_some_and(|v| expr(v, info)),
        }) || b.tail.as_ref().is_some_and(|t| expr(t, info))
    }
    fn expr(e: &HirExpr, info: &TypeInfo) -> bool {
        let measured_cond = |c: &HirExpr| *info.ty(c.id) == Ty::Measured(Box::new(Ty::Bool));
        match &e.kind {
            HirExprKind::If { cond, then_block, else_expr } => {
                measured_cond(cond)
                    || expr(cond, info)
                    || block(then_block, info)
                    || else_expr.as_ref().is_some_and(|x| expr(x, info))
            }
            HirExprKind::While { cond, body } => {
                measured_cond(cond) || expr(cond, info) || block(body, info)
            }
            HirExprKind::For { iterable, body, .. } => expr(iterable, info) || block(body, info),
            HirExprKind::Block(b) => block(b, info),
            HirExprKind::Call { args, .. } => args.iter().any(|a| expr(a, info)),
            _ => false,
        }
    }
    pkg.fns[def].body.as_ref().is_some_and(|b| block(b, info))
}

/// Integer value of a literal expression, if it is one.
fn literal_int(e: &HirExpr) -> Option<i64> {
    match &e.kind {
        HirExprKind::Lit(Literal::Int(v)) => Some(*v),
        _ => None,
    }
}

fn const_value(c: &ConstExpr) -> Option<i64> {
    gala_hir::const_int(c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gala_hir::lower_file;
    use gala_parser::parse_source;
    use gala_span::FileId;

    fn check(src: &str) -> Diagnostics {
        let out = parse_source(FileId(0), src);
        assert!(!out.diagnostics.has_errors(), "parse: {:?}", out.diagnostics);
        let (pkg, mut d) = lower_file(FileId(0), out.items);
        assert!(!d.has_errors(), "lower: {d:?}");
        let (_, td) = check_package(&pkg);
        d.extend(td);
        let (_, ld) = check_linearity(&pkg, &check_package(&pkg).0);
        d.extend(ld);
        d
    }

    fn codes_of(d: &Diagnostics) -> Vec<String> {
        d.errors().iter().map(|e| e.code.to_string()).collect()
    }

    fn assert_clean(src: &str) {
        let d = check(src);
        assert!(!d.has_errors(), "expected clean, got {:?}", codes_of(&d));
    }

    fn assert_code(src: &str, code: &str) {
        let d = check(src);
        assert!(codes_of(&d).iter().any(|c| c == code), "expected {code}, got {:?}", codes_of(&d));
    }

    #[test]
    fn bell_pair_from_spec_is_well_typed() {
        assert_clean(
            "fn bell() -> Qubits<2> quantum {\n let a = qubit()\n let b = qubit()\n h(a)\n let (a, b) = cx(a, b)\n (a, b)\n}\n",
        );
    }

    #[test]
    fn coin_flip_is_prob_and_clean() {
        assert_clean("fn coin() -> Measured<Bool> prob { let q = qubit()\n h(q)\n measure(q) }");
    }

    #[test]
    fn classical_arithmetic_types() {
        assert_clean("fn f(x: Int) -> Int pure { let y = x * 2 + 1\n y }");
        assert_code("fn f() -> Int { true }", "E0201");
        assert_code("fn f() -> Bool { 1 + true }", "E0201");
    }

    #[test]
    fn arity_mismatch_is_e0204() {
        assert_code("fn g(a: Int) -> Int { a }\nfn f() -> Int { g(1, 2) }", "E0204");
    }

    #[test]
    fn pure_calling_quantum_is_e0302() {
        assert_code("fn f(q: Qubit) -> Qubit pure { h(q) }", "E0302");
    }

    #[test]
    fn quantum_calling_prob_is_e0303() {
        assert_code(
            "fn m(q: Qubit) -> Measured<Bool> prob { measure(q) }\nfn f(q: Qubit) -> Measured<Bool> quantum { m(q) }",
            "E0303",
        );
    }

    #[test]
    fn measure_in_quantum_is_e0304() {
        assert_code("fn f() -> Measured<Bool> quantum { let q = qubit()\n measure(q) }", "E0304");
    }

    #[test]
    fn declared_effect_is_inferred_when_absent() {
        let out = parse_source(
            FileId(0),
            "fn g() -> Measured<Bool> { let q = qubit()\n measure(q) }\nfn f() -> Int { 1 }",
        );
        let (pkg, _) = lower_file(FileId(0), out.items);
        let (info, _) = check_package(&pkg);
        assert_eq!(info.effects[0], Effect::Prob);
        assert_eq!(info.effects[1], Effect::Pure);
    }

    #[test]
    fn effects_propagate_through_calls_in_fixpoint() {
        let out = parse_source(FileId(0), "fn a() -> Qubit { b() }\nfn b() -> Qubit { qubit() }");
        let (pkg, _) = lower_file(FileId(0), out.items);
        let (info, _) = check_package(&pkg);
        assert_eq!(info.effects[0], Effect::Quantum, "a inherits quantum from b");
    }

    #[test]
    fn register_size_literal_is_checked() {
        assert_code("fn f() -> Qubits<3> quantum { let q = qubits<2>()\n q }", "E0201");
    }

    #[test]
    fn constant_index_out_of_range_is_e0206() {
        assert_code("fn f() quantum { let q = qubits<2>()\n h(q[5])\n drop(q) }", "E0206");
    }

    #[test]
    fn cloning_is_e0401() {
        assert_code("fn f(q: Qubit) -> Qubit quantum { clone(q) }", "E0401");
    }

    #[test]
    fn assignment_to_immutable_is_e0203() {
        assert_code("fn f() -> Int { let x = 1\n x = 2\n x }", "E0203");
        assert_clean("fn f() -> Int { let mut x = 1\n x = 2\n x }");
    }

    #[test]
    fn match_on_int_literals_checks() {
        assert_clean("fn f(n: Int) -> String pure { let r = match n {\n 0 => \"zero\",\n _ => \"other\",\n }\n r }");
    }

    #[test]
    fn block_ending_in_return_has_no_fallthrough_type() {
        assert_clean("fn main() -> Int {\n    let x = 1\n    return x;\n}\n");
    }

    #[test]
    fn unsupported_grad_is_reported() {
        assert_code("fn f() -> Int { grad(f, wrt: 1) }", "E0207");
    }
}
