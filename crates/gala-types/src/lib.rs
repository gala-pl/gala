//! Type and effect inference, linearity checking for Gala.

use gala_ast::{self, Literal, ConstExpr, Type, Effect as HEffect};
use gala_hir::*;
use gala_span::{Span, FileId};
use gala_diagnostics::{Diagnostic, Diagnostics, codes};
use ena::unify::{InPlace, UnifyKey, UnificationTable};
use std::collections::HashMap;

/// Type representation for inference.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ty {
    Var(TyVar),
    Int,
    Float,
    Bool,
    Complex,
    String,
    Unit,
    Params,
    Qubit,
    Qubits(u64),
    Measured(Box<Ty>),
    Tuple(Vec<Ty>),
    Array(Box<Ty>, u64),
    Fn { params: Vec<Ty>, ret: Box<Ty>, effect: Effect },
    Named(DefId, Vec<Ty>),
    TyError,
}

/// Type variable for unification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TyVar(pub u32);

impl UnifyKey for TyVar {
    type Value = ();
    fn index(&self) -> u32 { self.0 }
    fn from_index(i: u32) -> Self { TyVar(i) }
    fn tag() -> &'static str { "TyVar" }
}

/// Type inference error.
#[derive(Debug, Clone)]
pub enum TypeError {
    Mismatch(Ty, Ty),
    OccursCheck(TyVar, Ty),
    UnknownType,
}

/// Effect system: pure ⊑ quantum ⊑ prob.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Effect {
    Pure,
    Quantum,
    Prob,
}

impl Effect {
    pub fn subsumes(self, other: Effect) -> bool {
        self >= other
    }
    pub fn join(self, other: Effect) -> Effect {
        std::cmp::max(self, other)
    }
}

/// Inference context for type checking.
pub struct InferCtxt {
    type_vars: UnificationTable<InPlace<TyVar>>,
    next_var: u32,
    constraints: Vec<Constraint>,
    env: HashMap<DefId, Ty>,
    local_env: HashMap<LocalId, Ty>,
    fn_effects: HashMap<DefId, Effect>,
}

#[derive(Debug, Clone)]
enum Constraint {
    Equate(Ty, Ty),
    SubEffect(Effect, Effect),
}

impl InferCtxt {
    pub fn new() -> Self {
        InferCtxt {
            type_vars: UnificationTable::new(),
            next_var: 0,
            constraints: Vec::new(),
            env: HashMap::new(),
            local_env: HashMap::new(),
            fn_effects: HashMap::new(),
        }
    }

    pub fn bind_var(&mut self, def_id: DefId, ty: Ty) {
        self.env.insert(def_id, ty);
    }

    pub fn bind_local(&mut self, local_id: LocalId, ty: Ty) {
        self.local_env.insert(local_id, ty);
    }

    pub fn bind_fn_effect(&mut self, def_id: DefId, eff: Effect) {
        self.fn_effects.insert(def_id, eff);
    }

    fn lookup(&self, def_id: &DefId) -> Option<Ty> {
        if let Some(ty) = self.env.get(def_id) {
            return Some(ty.clone());
        }
        let local_id = LocalId(def_id.index);
        if let Some(ty) = self.local_env.get(&local_id) {
            return Some(ty.clone());
        }
        None
    }

    fn fresh_var(&mut self) -> Ty {
        let var = TyVar(self.next_var);
        self.next_var += 1;
        self.type_vars.new_key(());
        Ty::Var(var)
    }

    fn equate(&mut self, a: Ty, b: Ty) {
        self.constraints.push(Constraint::Equate(a, b));
    }

    fn constrain_effect(&mut self, actual: Effect, expected: Effect) {
        self.constraints.push(Constraint::SubEffect(actual, expected));
    }

    pub fn solve(&mut self) -> Result<(), Vec<TypeError>> {
        let mut errors = Vec::new();
        for constraint in self.constraints.drain(..) {
            match constraint {
                Constraint::Equate(a, b) => {
                    if a != b {
                        errors.push(TypeError::Mismatch(a, b));
                    }
                }
                Constraint::SubEffect(actual, expected) => {
                    if !actual.subsumes(expected) {
                        errors.push(TypeError::Mismatch(Ty::TyError, Ty::TyError));
                    }
                }
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// Check effect boundary: a function with effect `fn_eff` calls a callee
/// with effect `callee_eff`.
pub fn check_effect_boundary(
    fn_eff: gala_ast::Effect,
    callee_eff: Effect,
    span: Span,
    diags: &mut Diagnostics,
) {
    let fn_e = match fn_eff {
        gala_ast::Effect::Pure => Effect::Pure,
        gala_ast::Effect::Quantum => Effect::Quantum,
        gala_ast::Effect::Prob => Effect::Prob,
    };
    if !fn_e.subsumes(callee_eff) {
        let msg = format!(
            "cannot call {:?} function from {:?} context",
            callee_eff, fn_e
        );
        diags.push(Diagnostic::error(codes::TYPE_MISMATCH, msg)
            .with_primary_label(span, "effect mismatch here")
            .with_help("change the calling function's effect or restructure the code"));
    }
}

/// Type-check a function definition.
pub fn type_check_fn(hir_fn: &HirFnDef) -> Result<Ty, Diagnostics> {
    let mut cx = InferCtxt::new();
    let mut diags = Diagnostics::new();
    let fn_eff = effect_to_ty_effect(&hir_fn.effect);

    for param in &hir_fn.params {
        let param_ty = ast_type_to_ty(&param.ty);
        cx.bind_local(param.local_id, param_ty);
    }

    let body_ty = check_block(&mut cx, &hir_fn.body, &fn_eff, &mut diags);

    if let Some(ret) = &hir_fn.ret_ty {
        let expected_ret = ast_type_to_ty(ret);
        cx.equate(body_ty.clone(), expected_ret);
    }

    if let Err(errors) = cx.solve() {
        for error in errors {
            diags.push(Diagnostic::error(codes::TYPE_MISMATCH, format!("type mismatch: {:?}", error)));
        }
    }

    if diags.has_errors() { Err(diags) } else { Ok(body_ty) }
}

fn effect_to_ty_effect(e: &gala_ast::Effect) -> Effect {
    match e {
        gala_ast::Effect::Pure => Effect::Pure,
        gala_ast::Effect::Quantum => Effect::Quantum,
        gala_ast::Effect::Prob => Effect::Prob,
    }
}

fn check_block(
    cx: &mut InferCtxt,
    block: &HirBlock,
    fn_eff: &Effect,
    diags: &mut Diagnostics,
) -> Ty {
    let mut last_ty = Ty::Unit;
    for stmt in &block.stmts {
        match stmt {
            HirStmt::Let(let_stmt) => {
                let init_ty = check_hir_expr(cx, &let_stmt.init, fn_eff, diags);
                cx.bind_local(let_stmt.local_id, init_ty);
            }
            HirStmt::Expr(e) => {
                last_ty = check_hir_expr(cx, e, fn_eff, diags);
            }
            HirStmt::Return(e) => {
                if let Some(e) = e {
                    last_ty = check_hir_expr(cx, e, fn_eff, diags);
                }
            }
            _ => {}
        }
    }
    if let Some(tail) = &block.tail {
        last_ty = check_hir_expr(cx, tail, fn_eff, diags);
    }
    last_ty
}

fn check_hir_expr(
    cx: &mut InferCtxt,
    expr: &HirExpr,
    fn_eff: &Effect,
    diags: &mut Diagnostics,
) -> Ty {
    match expr {
        HirExpr::Literal(l) => match l {
            Literal::Int(_) => Ty::Int,
            Literal::Float(_) => Ty::Float,
            Literal::Bool(_) => Ty::Bool,
            Literal::String(_) => Ty::String,
            Literal::Complex { .. } => Ty::Complex,
            Literal::Unit => Ty::Unit,
        },
        HirExpr::Ident(_, def_id) => {
            if let Some(ty) = cx.lookup(def_id) {
                ty
            } else {
                diags.push(Diagnostic::error(codes::UNKNOWN_TYPE, format!("unknown variable {:?}", def_id)));
                Ty::TyError
            }
        }
        HirExpr::Binary(b) => {
            let lhs = check_hir_expr(cx, &b.lhs, fn_eff, diags);
            let rhs = check_hir_expr(cx, &b.rhs, fn_eff, diags);
            match b.op {
                gala_ast::BinOp::Add | gala_ast::BinOp::Sub
                | gala_ast::BinOp::Mul | gala_ast::BinOp::Div | gala_ast::BinOp::Mod => {
                    cx.equate(lhs.clone(), rhs.clone());
                    lhs
                }
                gala_ast::BinOp::Eq | gala_ast::BinOp::Ne
                | gala_ast::BinOp::Lt | gala_ast::BinOp::Le
                | gala_ast::BinOp::Gt | gala_ast::BinOp::Ge => {
                    cx.equate(lhs.clone(), rhs.clone());
                    Ty::Bool
                }
                gala_ast::BinOp::And | gala_ast::BinOp::Or => {
                    cx.equate(lhs.clone(), Ty::Bool);
                    cx.equate(rhs.clone(), Ty::Bool);
                    Ty::Bool
                }
                _ => Ty::TyError,
            }
        }
        HirExpr::Block(b) => check_block(cx, b, fn_eff, diags),
        HirExpr::Call(c) => {
            let _callee_ty = check_hir_expr(cx, &c.callee, fn_eff, diags);
            let arg_tys: Vec<Ty> = c.args.iter()
                .map(|a| check_hir_expr(cx, a, fn_eff, diags))
                .collect();
            let ret_ty = cx.fresh_var();
            let fn_ty = Ty::Fn {
                params: arg_tys,
                ret: Box::new(ret_ty.clone()),
                effect: Effect::Pure,
            };
            cx.equate(_callee_ty, fn_ty);
            ret_ty
        }
        HirExpr::If(i) => {
            let cond_ty = check_hir_expr(cx, &i.cond, fn_eff, diags);
            cx.equate(cond_ty, Ty::Bool);
            let then_ty = check_block(cx, &i.then_branch, fn_eff, diags);
            let else_ty = i.else_branch.as_ref()
                .map(|e| check_hir_expr(cx, e, fn_eff, diags))
                .unwrap_or(Ty::Unit);
            cx.equate(then_ty.clone(), else_ty.clone());
            then_ty
        }
        HirExpr::Return(e) => {
            if let Some(e) = e {
                check_hir_expr(cx, e, fn_eff, diags)
            } else {
                Ty::Unit
            }
        }
        HirExpr::Unary(u) => {
            let inner = check_hir_expr(cx, &u.expr, fn_eff, diags);
            match u.op {
                gala_ast::UnOp::Neg => {
                    cx.equate(inner.clone(), Ty::Int);
                    Ty::Int
                }
                gala_ast::UnOp::Not => {
                    cx.equate(inner.clone(), Ty::Bool);
                    Ty::Bool
                }
            }
        }
        _ => Ty::TyError,
    }
}

/// Convert an AST type annotation to a Ty.
pub fn ast_type_to_ty(ty: &Type) -> Ty {
    match ty {
        Type::Path(p) => match p.segments[0].ident.as_str() {
            "Bool" => Ty::Bool,
            "Int" => Ty::Int,
            "Float" => Ty::Float,
            "Complex" => Ty::Complex,
            "String" => Ty::String,
            "Unit" => Ty::Unit,
            "Params" => Ty::Params,
            _ => Ty::TyError,
        },
        Type::Qubit => Ty::Qubit,
        Type::Qubits(c) => match &**c {
            ConstExpr::Int(n) => Ty::Qubits(*n as u64),
            _ => Ty::TyError,
        },
        Type::Measured(t) => Ty::Measured(Box::new(ast_type_to_ty(t))),
        Type::Tuple(ts) => Ty::Tuple(ts.iter().map(|t| ast_type_to_ty(t)).collect()),
        Type::Array(t, n) => match &**n {
            ConstExpr::Int(n) => Ty::Array(Box::new(ast_type_to_ty(t)), *n as u64),
            _ => Ty::TyError,
        },
        Type::Named(_s, ts) => Ty::Named(DefId { crate_id: CrateId(FileId(0)), index: 0 },
            ts.iter().map(|t| ast_type_to_ty(t)).collect()),
        Type::Fn { params, ret, .. } => Ty::Fn {
            params: params.iter().map(|t| ast_type_to_ty(t)).collect(),
            ret: Box::new(ast_type_to_ty(ret)),
            effect: Effect::Pure,
        },
    }
}

/// Unify two const-generic sizes.
pub fn unify_qubits_size(a: u64, b: u64) -> Result<u64, Diagnostics> {
    if a == b {
        Ok(a)
    } else {
        let mut diags = Diagnostics::new();
        diags.push(Diagnostic::error(codes::TYPE_MISMATCH,
            format!("qubit register size mismatch: {} vs {}", a, b)));
        Err(diags)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Linearity Checker
// ═══════════════════════════════════════════════════════════════════════════════

/// Check if a type is linear (must be used exactly once).
pub fn is_linear_type(ty: &Ty) -> bool {
    match ty {
        Ty::Qubit | Ty::Qubits(_) => true,
        Ty::Measured(_) => false,  // Measured<T> is classical (copyable)
        Ty::Tuple(ts) => ts.iter().any(is_linear_type),
        Ty::Array(t, _) => is_linear_type(t),
        Ty::Fn { .. } => false,
        Ty::Var(_) => true,  // Conservative: assume linear if unknown
        _ => false,
    }
}

/// State of a linear value in the checker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinearState {
    /// Value has been consumed and cannot be used again.
    Consumed,
    /// Value is still available for use.
    Available,
}

/// Check linearity for a function definition.
/// Uses type info from the type checker to distinguish quantum from classical values.
pub fn check_linearity(hir_fn: &HirFnDef) -> Diagnostics {
    let mut diags = Diagnostics::new();

    // Build the type env from parameter types (inferred types would come from type checker)
    let mut type_env: HashMap<LocalId, Ty> = HashMap::new();
    for param in &hir_fn.params {
        let param_ty = ast_type_to_ty(&param.ty);
        type_env.insert(param.local_id, param_ty);
    }

    let mut state: HashMap<(bool, u32), LinearState> = HashMap::new();
    // bool: true = DefId, false = LocalId
    // u32: the index

    check_block_linearity(&hir_fn.body, &mut state, &type_env, &mut diags);

    diags
}

fn track_id(state: &HashMap<(bool, u32), LinearState>, is_def: bool, idx: u32) -> LinearState {
    state.get(&(is_def, idx)).copied().unwrap_or(LinearState::Available)
}

fn mark_consumed(state: &mut HashMap<(bool, u32), LinearState>, is_def: bool, idx: u32) {
    state.insert((is_def, idx), LinearState::Consumed);
}

fn is_quantum_type(ty: &Ty) -> bool {
    matches!(ty, Ty::Qubit | Ty::Qubits(_))
}

fn check_block_linearity(
    block: &HirBlock,
    state: &mut HashMap<(bool, u32), LinearState>,
    type_env: &HashMap<LocalId, Ty>,
    diags: &mut Diagnostics,
) {
    for stmt in &block.stmts {
        match stmt {
            HirStmt::Let(l) => {
                check_expr_linearity(&l.init, state, type_env, diags);
                // The let binding makes the value available (it's fresh)
                if let Some(ty) = type_env.get(&l.local_id) {
                    if is_quantum_type(ty) {
                        state.insert((false, l.local_id.0), LinearState::Available);
                    }
                }
            }
            HirStmt::Expr(e) => {
                check_expr_linearity(e, state, type_env, diags);
            }
            _ => {}
        }
    }
    if let Some(tail) = &block.tail {
        check_expr_linearity(tail, state, type_env, diags);
    }
}

fn check_expr_linearity(
    expr: &HirExpr,
    state: &mut HashMap<(bool, u32), LinearState>,
    type_env: &HashMap<LocalId, Ty>,
    diags: &mut Diagnostics,
) {
    match expr {
        HirExpr::Ident(_, def_id) => {
            let key = (true, def_id.index);
            let lin_state = track_id(state, true, def_id.index);
            if lin_state == LinearState::Consumed {
                diags.push(Diagnostic::error(codes::USE_AFTER_CONSUME,
                    format!("value {:?} used after it was consumed", def_id))
                    .with_help("qubits are linear resources — they must be used exactly once")
                    .with_note("move the use before the consuming operation or restructure"));
            }
            // Check if this is a quantum type by looking up in local env
            let local_id = LocalId(def_id.index);
            if let Some(ty) = type_env.get(&local_id) {
                if is_quantum_type(ty) {
                    mark_consumed(state, true, def_id.index);
                }
            }
        }
        HirExpr::Binary(b) => {
            check_expr_linearity(&b.lhs, state, type_env, diags);
            check_expr_linearity(&b.rhs, state, type_env, diags);
        }
        HirExpr::Block(b) => {
            check_block_linearity(b, state, type_env, diags);
        }
        HirExpr::Call(c) => {
            check_expr_linearity(&c.callee, state, type_env, diags);
            for arg in &c.args {
                check_expr_linearity(arg, state, type_env, diags);
            }
        }
        HirExpr::If(i) => {
            check_expr_linearity(&i.cond, state, type_env, diags);
            // Save state before branches
            let saved_state = state.clone();
            check_block_linearity(&i.then_branch, state, type_env, diags);
            if let Some(else_) = &i.else_branch {
                // For simplicity, use the saved state for the else branch
                let mut else_state = saved_state.clone();
                check_expr_linearity(else_, &mut else_state, type_env, diags);
                // Merge: a value is available only if available in both branches
                for (k, v) in &saved_state {
                    if *v == LinearState::Available && else_state.get(k).copied().unwrap_or(LinearState::Consumed) == LinearState::Available {
                        state.insert(*k, LinearState::Available);
                    } else {
                        state.insert(*k, LinearState::Consumed);
                    }
                }
            }
        }
        HirExpr::Return(e) => {
            if let Some(e) = e {
                check_expr_linearity(e, state, type_env, diags);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gala_ast::Ident;

    #[test]
    fn test_effect_subsumption() {
        assert!(Effect::Quantum.subsumes(Effect::Pure));
        assert!(Effect::Prob.subsumes(Effect::Quantum));
        assert!(Effect::Prob.subsumes(Effect::Pure));
        assert!(!Effect::Pure.subsumes(Effect::Quantum));
    }

    #[test]
    fn test_effect_join() {
        assert_eq!(Effect::Pure.join(Effect::Quantum), Effect::Quantum);
        assert_eq!(Effect::Quantum.join(Effect::Prob), Effect::Prob);
    }

    #[test]
    fn test_ast_type_to_ty() {
        assert_eq!(ast_type_to_ty(&Type::Path(gala_ast::Path {
            segments: vec![gala_ast::PathSegment { ident: Ident::new("Int"), type_args: Vec::new() }],
            span: Span::dummy(),
        })), Ty::Int);
        assert_eq!(ast_type_to_ty(&Type::Qubit), Ty::Qubit);
        assert_eq!(ast_type_to_ty(&Type::Qubits(Box::new(ConstExpr::Int(4)))), Ty::Qubits(4));
    }

    #[test]
    fn test_literal_types() {
        assert_eq!(literal_type(&Literal::Int(42)), Ty::Int);
        assert_eq!(literal_type(&Literal::Float(3.14)), Ty::Float);
        assert_eq!(literal_type(&Literal::Bool(true)), Ty::Bool);
        assert_eq!(literal_type(&Literal::String("hi".into())), Ty::String);
        assert_eq!(literal_type(&Literal::Unit), Ty::Unit);
    }

    fn literal_type(l: &Literal) -> Ty {
        match l {
            Literal::Int(_) => Ty::Int,
            Literal::Float(_) => Ty::Float,
            Literal::Bool(_) => Ty::Bool,
            Literal::String(_) => Ty::String,
            Literal::Complex { .. } => Ty::Complex,
            Literal::Unit => Ty::Unit,
        }
    }

    #[test]
    fn test_unify_qubits_same() {
        assert_eq!(unify_qubits_size(4, 4).unwrap(), 4);
    }

    #[test]
    fn test_unify_qubits_different() {
        assert!(unify_qubits_size(4, 8).is_err());
    }

    #[test]
    fn test_check_effect_boundary_valid() {
        let mut diags = Diagnostics::new();
        check_effect_boundary(gala_ast::Effect::Quantum, Effect::Pure, Span::dummy(), &mut diags);
        assert!(!diags.has_errors());
    }

    #[test]
    fn test_check_effect_boundary_violation() {
        let mut diags = Diagnostics::new();
        check_effect_boundary(gala_ast::Effect::Pure, Effect::Quantum, Span::dummy(), &mut diags);
        assert!(diags.has_errors());
    }

    #[test]
    fn test_infer_ctxt_fresh_var() {
        let mut cx = InferCtxt::new();
        let v1 = cx.fresh_var();
        let v2 = cx.fresh_var();
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_bind_and_lookup() {
        let mut cx = InferCtxt::new();
        let def_id = DefId { crate_id: CrateId(FileId(0)), index: 42 };
        cx.bind_var(def_id.clone(), Ty::Int);
        assert_eq!(cx.lookup(&def_id), Some(Ty::Int));
    }

    #[test]
    fn test_bind_and_lookup_local() {
        let mut cx = InferCtxt::new();
        let local = LocalId(7);
        cx.bind_local(local, Ty::Bool);
        let def_id = DefId { crate_id: CrateId(FileId(0)), index: 7 };
        assert_eq!(cx.lookup(&def_id), Some(Ty::Bool));
    }

    #[test]
    fn test_type_check_fn_simple() {
        let hir_fn = HirFnDef {
            def_id: DefId { crate_id: CrateId(FileId(0)), index: 0 },
            ident: Ident::new("main"),
            generics: Vec::new(),
            params: Vec::new(),
            ret_ty: Some(gala_ast::Type::Path(gala_ast::Path {
                segments: vec![gala_ast::PathSegment { ident: Ident::new("Int"), type_args: Vec::new() }],
                span: Span::dummy(),
            })),
            effect: gala_ast::Effect::Pure,
            body: gala_hir::HirBlock {
                stmts: vec![gala_hir::HirStmt::Return(Some(Box::new(gala_hir::HirExpr::Literal(Literal::Int(42)))))],
                tail: None,
                span: Span::dummy(),
            },
            span: Span::dummy(),
        };
        let result = type_check_fn(&hir_fn);
        assert!(result.is_ok(), "type check failed: {:?}", result.err());
        let ty = result.unwrap();
        assert_eq!(ty, Ty::Int);
    }

    #[test]
    fn test_type_check_binary() {
        let hir_fn = HirFnDef {
            def_id: DefId { crate_id: CrateId(FileId(0)), index: 0 },
            ident: Ident::new("add"),
            generics: Vec::new(),
            params: Vec::new(),
            ret_ty: None,
            effect: gala_ast::Effect::Pure,
            body: gala_hir::HirBlock {
                stmts: Vec::new(),
                tail: Some(Box::new(gala_hir::HirExpr::Binary(gala_hir::HirBinaryExpr {
                    lhs: Box::new(gala_hir::HirExpr::Literal(Literal::Int(2))),
                    op: gala_ast::BinOp::Add,
                    rhs: Box::new(gala_hir::HirExpr::Literal(Literal::Int(3))),
                    span: Span::dummy(),
                }))),
                span: Span::dummy(),
            },
            span: Span::dummy(),
        };
        let result = type_check_fn(&hir_fn);
        assert!(result.is_ok());
        let ty = result.unwrap();
        assert_eq!(ty, Ty::Int);
    }

    #[test]
    fn test_is_linear_type() {
        assert!(is_linear_type(&Ty::Qubit));
        assert!(is_linear_type(&Ty::Qubits(4)));
        assert!(!is_linear_type(&Ty::Int));
        assert!(!is_linear_type(&Ty::Bool));
        assert!(!is_linear_type(&Ty::String));
        assert!(!is_linear_type(&Ty::Float));
        assert!(!is_linear_type(&Ty::Measured(Box::new(Ty::Bool))));
    }

    #[test]
    fn test_linearity_use_after_consume() {
        let cid = CrateId(FileId(0));
        let qid = LocalId(0);
        let dq = DefId { crate_id: cid.clone(), index: qid.0 };
        let hir_fn = HirFnDef {
            def_id: DefId { crate_id: cid.clone(), index: 0 },
            ident: Ident::new("bad"),
            generics: Vec::new(),
            params: vec![
                HirParam { local_id: qid, mutable: false,
                    pattern: gala_ast::Pattern::Ident(Ident::new("q")),
                    ty: gala_ast::Type::Qubit, span: Span::dummy() },
            ],
            ret_ty: None, effect: gala_ast::Effect::Quantum,
            body: gala_hir::HirBlock {
                stmts: vec![
                    gala_hir::HirStmt::Expr(gala_hir::HirExpr::Ident(Ident::new("q"), dq.clone())),
                    gala_hir::HirStmt::Expr(gala_hir::HirExpr::Ident(Ident::new("q"), dq.clone())),
                ],
                tail: None, span: Span::dummy(),
            },
            span: Span::dummy(),
        };
        let diags = check_linearity(&hir_fn);
        assert!(diags.has_errors(), "should catch use-after-consume of qubit");
    }

    #[test]
    fn test_linearity_classical_copy_ok() {
        let cid = CrateId(FileId(0));
        let xid = LocalId(0);
        let dx = DefId { crate_id: cid.clone(), index: xid.0 };
        let hir_fn = HirFnDef {
            def_id: DefId { crate_id: cid.clone(), index: 0 }, ident: Ident::new("ok"),
            generics: Vec::new(),
            params: vec![
                HirParam { local_id: xid, mutable: false,
                    pattern: gala_ast::Pattern::Ident(Ident::new("x")),
                    ty: gala_ast::Type::Path(gala_ast::Path {
                        segments: vec![gala_ast::PathSegment { ident: Ident::new("Int"), type_args: Vec::new() }],
                        span: Span::dummy() }),
                    span: Span::dummy() },
            ],
            ret_ty: None, effect: gala_ast::Effect::Pure,
            body: gala_hir::HirBlock {
                stmts: vec![
                    gala_hir::HirStmt::Expr(gala_hir::HirExpr::Ident(Ident::new("x"), dx.clone())),
                    gala_hir::HirStmt::Expr(gala_hir::HirExpr::Ident(Ident::new("x"), dx.clone())),
                ],
                tail: None, span: Span::dummy(),
            },
            span: Span::dummy(),
        };
        let diags = check_linearity(&hir_fn);
        assert!(!diags.has_errors(), "classical values should be copyable");
    }

    #[test]
    fn test_linearity_call_consumes_args() {
        let cid = CrateId(FileId(0));
        let qid = LocalId(0);
        let dq = DefId { crate_id: cid.clone(), index: qid.0 };
        let dh = DefId { crate_id: cid.clone(), index: 99 };
        let hir_fn = HirFnDef {
            def_id: DefId { crate_id: cid.clone(), index: 1 }, ident: Ident::new("test"),
            generics: Vec::new(),
            params: vec![
                HirParam { local_id: qid, mutable: false,
                    pattern: gala_ast::Pattern::Ident(Ident::new("q")),
                    ty: gala_ast::Type::Qubit, span: Span::dummy() },
            ],
            ret_ty: None, effect: gala_ast::Effect::Quantum,
            body: gala_hir::HirBlock {
                stmts: vec![
                    gala_hir::HirStmt::Expr(gala_hir::HirExpr::Call(gala_hir::HirCallExpr {
                        callee: Box::new(gala_hir::HirExpr::Ident(Ident::new("h"), dh)),
                        args: vec![gala_hir::HirExpr::Ident(Ident::new("q"), dq.clone())],
                        span: Span::dummy(),
                    })),
                    gala_hir::HirStmt::Expr(gala_hir::HirExpr::Ident(Ident::new("q"), dq.clone())),
                ],
                tail: None, span: Span::dummy(),
            },
            span: Span::dummy(),
        };
        let diags = check_linearity(&hir_fn);
        assert!(diags.has_errors(), "qubit used after call should be an error");
    }
}