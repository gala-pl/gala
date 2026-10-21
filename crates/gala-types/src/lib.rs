//! Type and effect inference, linearity checking for Gala.

use ena::unify::{InPlace, UnificationTable, UnifyKey};
use gala_ast::{self, ConstExpr, Literal, Type};
use gala_diagnostics::{codes, Diagnostic, Diagnostics};
use gala_hir::*;
use gala_span::{FileId, Span};
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
    fn index(&self) -> u32 {
        self.0
    }
    fn from_index(i: u32) -> Self {
        TyVar(i)
    }
    fn tag() -> &'static str {
        "TyVar"
    }
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
    // Effect-subsumption scaffolding: constructed via `constrain_effect`, solved in `solve`.
    #[allow(dead_code)]
    SubEffect(Effect, Effect),
}

impl Default for InferCtxt {
    fn default() -> Self {
        Self::new()
    }
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
        if let Some(ty) = builtin_type(def_id.index) {
            return Some(ty);
        }
        if let Some(local_id) = local_id_from_def_id(def_id) {
            if let Some(ty) = self.local_env.get(&local_id) {
                return Some(ty.clone());
            }
        } else if let Some(ty) = self.local_env.get(&LocalId(def_id.index)) {
            // Keep direct unit-test HIR construction working even when it has
            // not passed through the resolver's local-ID encoding.
            return Some(ty.clone());
        }
        if let Some(ty) = self.env.get(def_id) {
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

    #[allow(dead_code)]
    fn constrain_effect(&mut self, actual: Effect, expected: Effect) {
        self.constraints.push(Constraint::SubEffect(actual, expected));
    }

    pub fn solve(&mut self) -> Result<(), Vec<TypeError>> {
        let mut errors = Vec::new();
        let mut substitutions = HashMap::new();
        for constraint in self.constraints.drain(..) {
            match constraint {
                Constraint::Equate(a, b) => {
                    unify_types(a, b, &mut substitutions, &mut errors);
                }
                Constraint::SubEffect(actual, expected) => {
                    if !actual.subsumes(expected) {
                        errors.push(TypeError::Mismatch(Ty::TyError, Ty::TyError));
                    }
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn resolve_type(ty: Ty, substitutions: &HashMap<TyVar, Ty>) -> Ty {
    match ty {
        Ty::Var(var) => substitutions
            .get(&var)
            .cloned()
            .map(|resolved| resolve_type(resolved, substitutions))
            .unwrap_or(Ty::Var(var)),
        other => other,
    }
}

fn occurs_in(var: TyVar, ty: &Ty, substitutions: &HashMap<TyVar, Ty>) -> bool {
    match resolve_type(ty.clone(), substitutions) {
        Ty::Var(other) => var == other,
        Ty::Measured(inner) => occurs_in(var, &inner, substitutions),
        Ty::Tuple(items) => items.iter().any(|item| occurs_in(var, item, substitutions)),
        Ty::Array(inner, _) => occurs_in(var, &inner, substitutions),
        Ty::Fn { params, ret, .. } => {
            params.iter().any(|param| occurs_in(var, param, substitutions))
                || occurs_in(var, &ret, substitutions)
        }
        Ty::Named(_, args) => args.iter().any(|arg| occurs_in(var, arg, substitutions)),
        _ => false,
    }
}

fn unify_types(
    lhs: Ty,
    rhs: Ty,
    substitutions: &mut HashMap<TyVar, Ty>,
    errors: &mut Vec<TypeError>,
) {
    let lhs = resolve_type(lhs, substitutions);
    let rhs = resolve_type(rhs, substitutions);
    if lhs == rhs || matches!(&lhs, Ty::TyError) || matches!(&rhs, Ty::TyError) {
        return;
    }

    match (lhs, rhs) {
        (Ty::Var(var), ty) | (ty, Ty::Var(var)) => {
            if occurs_in(var, &ty, substitutions) {
                errors.push(TypeError::OccursCheck(var, ty));
            } else {
                substitutions.insert(var, ty);
            }
        }
        (Ty::Measured(lhs), Ty::Measured(rhs)) => {
            unify_types(*lhs, *rhs, substitutions, errors);
        }
        (Ty::Tuple(lhs), Ty::Tuple(rhs)) if lhs.len() == rhs.len() => {
            for (lhs, rhs) in lhs.into_iter().zip(rhs) {
                unify_types(lhs, rhs, substitutions, errors);
            }
        }
        (Ty::Qubits(size), Ty::Tuple(items)) | (Ty::Tuple(items), Ty::Qubits(size))
            if size as usize == items.len() =>
        {
            for item in items {
                unify_types(Ty::Qubit, item, substitutions, errors);
            }
        }
        (Ty::Array(lhs, lhs_len), Ty::Array(rhs, rhs_len)) if lhs_len == rhs_len => {
            unify_types(*lhs, *rhs, substitutions, errors);
        }
        (
            Ty::Fn { params: lhs_params, ret: lhs_ret, effect: lhs_effect },
            Ty::Fn { params: rhs_params, ret: rhs_ret, effect: rhs_effect },
        ) if lhs_params.len() == rhs_params.len() && lhs_effect == rhs_effect => {
            for (lhs, rhs) in lhs_params.into_iter().zip(rhs_params) {
                unify_types(lhs, rhs, substitutions, errors);
            }
            unify_types(*lhs_ret, *rhs_ret, substitutions, errors);
        }
        (Ty::Named(lhs_id, lhs_args), Ty::Named(rhs_id, rhs_args))
            if lhs_id == rhs_id && lhs_args.len() == rhs_args.len() =>
        {
            for (lhs, rhs) in lhs_args.into_iter().zip(rhs_args) {
                unify_types(lhs, rhs, substitutions, errors);
            }
        }
        (lhs, rhs) => errors.push(TypeError::Mismatch(lhs, rhs)),
    }
}

/// Types of compiler-provided quantum operations.  Builtins are represented
/// by reserved definition IDs by name resolution, so they can participate in
/// ordinary call checking without being special-cased by every caller.
fn builtin_type(index: u32) -> Option<Ty> {
    let builtin = index.checked_sub(10_000)?;
    let (params, ret, effect) = match builtin {
        0 => (vec![], Ty::Qubit, Effect::Quantum),
        1 => (vec![Ty::Int], Ty::Qubits(1), Effect::Quantum),
        2 => (vec![Ty::Qubit], Ty::Measured(Box::new(Ty::Bool)), Effect::Prob),
        3 => (vec![Ty::Qubit], Ty::Unit, Effect::Quantum),
        4..=8 => (vec![Ty::Qubit], Ty::Qubit, Effect::Quantum),
        9..=11 => (vec![Ty::Qubit, Ty::Float], Ty::Qubit, Effect::Quantum),
        12..=14 => {
            (vec![Ty::Qubit, Ty::Qubit], Ty::Tuple(vec![Ty::Qubit, Ty::Qubit]), Effect::Quantum)
        }
        _ => return None,
    };
    Some(Ty::Fn { params, ret: Box::new(ret), effect })
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
        let msg = format!("cannot call {:?} function from {:?} context", callee_eff, fn_e);
        diags.push(
            Diagnostic::error(codes::TYPE_MISMATCH, msg)
                .with_primary_label(span, "effect mismatch here")
                .with_help("change the calling function's effect or restructure the code"),
        );
    }
}

/// Type-check a function definition.
pub fn type_check_fn(hir_fn: &HirFnDef) -> Result<Ty, Diagnostics> {
    type_check_fn_with_functions(hir_fn, &HashMap::new())
}

/// Type-check a function with the signatures visible in its module.
pub fn type_check_fn_with_functions(
    hir_fn: &HirFnDef,
    functions: &HashMap<DefId, Ty>,
) -> Result<Ty, Diagnostics> {
    let mut cx = InferCtxt::new();
    let mut diags = Diagnostics::new();
    let fn_eff = effect_to_ty_effect(&hir_fn.effect);

    for (def_id, ty) in functions {
        cx.bind_var(def_id.clone(), ty.clone());
    }

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
            diags.push(Diagnostic::error(
                codes::TYPE_MISMATCH,
                format!("type mismatch: {:?}", error),
            ));
        }
    }

    if diags.has_errors() {
        Err(diags)
    } else {
        Ok(body_ty)
    }
}

/// Construct the callable type of a HIR function for module-level call
/// checking.
pub fn function_type(hir_fn: &HirFnDef) -> Ty {
    Ty::Fn {
        params: hir_fn.params.iter().map(|param| ast_type_to_ty(&param.ty)).collect(),
        ret: Box::new(hir_fn.ret_ty.as_ref().map(ast_type_to_ty).unwrap_or(Ty::Unit)),
        effect: effect_to_ty_effect(&hir_fn.effect),
    }
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
                let mut cursor = 0;
                bind_pattern_type(cx, &let_stmt.pattern, &let_stmt.local_ids, &mut cursor, init_ty);
            }
            HirStmt::Expr(e) => {
                last_ty = check_hir_expr(cx, e, fn_eff, diags);
            }
            HirStmt::Return(Some(e)) => {
                last_ty = check_hir_expr(cx, e, fn_eff, diags);
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
        HirExpr::Ident(_, def_id, _) => {
            if let Some(ty) = cx.lookup(def_id) {
                ty
            } else {
                diags.push(Diagnostic::error(
                    codes::UNKNOWN_TYPE,
                    format!("unknown variable {:?}", def_id),
                ));
                Ty::TyError
            }
        }
        HirExpr::Binary(b) => {
            let lhs = check_hir_expr(cx, &b.lhs, fn_eff, diags);
            let rhs = check_hir_expr(cx, &b.rhs, fn_eff, diags);
            match b.op {
                gala_ast::BinOp::Add
                | gala_ast::BinOp::Sub
                | gala_ast::BinOp::Mul
                | gala_ast::BinOp::Div
                | gala_ast::BinOp::Mod => {
                    cx.equate(lhs.clone(), rhs.clone());
                    lhs
                }
                gala_ast::BinOp::Eq
                | gala_ast::BinOp::Ne
                | gala_ast::BinOp::Lt
                | gala_ast::BinOp::Le
                | gala_ast::BinOp::Gt
                | gala_ast::BinOp::Ge => {
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
        HirExpr::Tuple(items) => {
            Ty::Tuple(items.iter().map(|item| check_hir_expr(cx, item, fn_eff, diags)).collect())
        }
        HirExpr::Array(items) => {
            let item_tys: Vec<_> =
                items.iter().map(|item| check_hir_expr(cx, item, fn_eff, diags)).collect();
            let Some(first) = item_tys.first().cloned() else { return Ty::TyError };
            for item_ty in item_tys.iter().skip(1) {
                cx.equate(first.clone(), item_ty.clone());
            }
            Ty::Array(Box::new(first), items.len() as u64)
        }
        HirExpr::Call(c) => {
            let callee_ty = check_hir_expr(cx, &c.callee, fn_eff, diags);
            let arg_tys: Vec<Ty> =
                c.args.iter().map(|a| check_hir_expr(cx, a, fn_eff, diags)).collect();
            match callee_ty {
                Ty::Fn { params, ret, effect } => {
                    if params.len() != arg_tys.len() {
                        diags.push(Diagnostic::error(
                            codes::TYPE_MISMATCH,
                            format!(
                                "function expects {} argument(s), found {}",
                                params.len(),
                                arg_tys.len()
                            ),
                        ));
                    }
                    for (expected, actual) in params.into_iter().zip(arg_tys) {
                        cx.equate(expected, actual);
                    }
                    if !fn_eff.subsumes(effect) {
                        diags.push(Diagnostic::error(
                            codes::EFFECT_MISMATCH,
                            "effectful operation is not allowed in this function",
                        ));
                    }
                    *ret
                }
                other => {
                    let ret_ty = cx.fresh_var();
                    cx.equate(
                        other,
                        Ty::Fn {
                            params: arg_tys,
                            ret: Box::new(ret_ty.clone()),
                            effect: Effect::Pure,
                        },
                    );
                    ret_ty
                }
            }
        }
        HirExpr::If(i) => {
            let cond_ty = check_hir_expr(cx, &i.cond, fn_eff, diags);
            cx.equate(cond_ty, Ty::Bool);
            let then_ty = check_block(cx, &i.then_branch, fn_eff, diags);
            let else_ty = i
                .else_branch
                .as_ref()
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
            "Vec" => Ty::Named(
                DefId { crate_id: CrateId(FileId(0)), index: 1 },
                p.segments[0]
                    .type_args
                    .iter()
                    .filter_map(|arg| match arg {
                        gala_ast::TypeArg::Type(ty) => Some(ast_type_to_ty(ty)),
                        gala_ast::TypeArg::Const(_) => None,
                    })
                    .collect(),
            ),
            _ => Ty::TyError,
        },
        Type::Qubit => Ty::Qubit,
        Type::Qubits(c) => match &**c {
            ConstExpr::Int(n) => Ty::Qubits(*n as u64),
            _ => Ty::TyError,
        },
        Type::Measured(t) => Ty::Measured(Box::new(ast_type_to_ty(t))),
        Type::Tuple(ts) => Ty::Tuple(ts.iter().map(ast_type_to_ty).collect()),
        Type::Array(t, n) => match &**n {
            ConstExpr::Int(n) => Ty::Array(Box::new(ast_type_to_ty(t)), *n as u64),
            _ => Ty::TyError,
        },
        Type::Named(_s, ts) => Ty::Named(
            DefId { crate_id: CrateId(FileId(0)), index: 0 },
            ts.iter().map(ast_type_to_ty).collect(),
        ),
        Type::Fn { params, ret, .. } => Ty::Fn {
            params: params.iter().map(ast_type_to_ty).collect(),
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
        diags.push(Diagnostic::error(
            codes::TYPE_MISMATCH,
            format!("qubit register size mismatch: {} vs {}", a, b),
        ));
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
        Ty::Measured(_) => false, // Measured<T> is classical (copyable)
        Ty::Tuple(ts) => ts.iter().any(is_linear_type),
        Ty::Array(t, _) => is_linear_type(t),
        Ty::Fn { .. } => false,
        Ty::Var(_) => true, // Conservative: assume linear if unknown
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
    let mut state: HashMap<(bool, u32), LinearState> = HashMap::new();
    for param in &hir_fn.params {
        let param_ty = ast_type_to_ty(&param.ty);
        type_env.insert(param.local_id, param_ty.clone());
        if is_linear_type(&param_ty) {
            state.insert((true, param.local_id.0), LinearState::Available);
        }
    }

    // bool: true = DefId, false = LocalId
    // u32: the index

    check_block_linearity(&hir_fn.body, &mut state, &mut type_env, &mut diags);

    for ((_, local_id), status) in state {
        if status == LinearState::Available {
            diags.push(
                Diagnostic::error(
                    codes::QUBIT_NOT_CONSUMED,
                    format!("linear value {:?} is not consumed", local_id),
                )
                .with_help("return, measure, or explicitly drop every linear value"),
            );
        }
    }

    diags
}

fn rebind_in_place_linearity(call: &HirCallExpr, state: &mut HashMap<(bool, u32), LinearState>) {
    let HirExpr::Ident(name, def_id, _) = call.callee.as_ref() else { return };
    // Only compiler-resolved gate builtins have in-place linear semantics.
    // A user function named `h` still consumes its argument and may return a
    // distinct value.
    if !(10_004..=10_014).contains(&def_id.index) {
        return;
    }
    if !matches!(
        name.0.as_str(),
        "h" | "x" | "y" | "z" | "s" | "t" | "rx" | "ry" | "rz" | "cx" | "cz" | "swap"
    ) {
        return;
    }
    for argument in &call.args {
        if let HirExpr::Ident(_, def_id, _) = argument {
            let local_id = local_id_from_def_id(def_id).unwrap_or(LocalId(def_id.index));
            state.insert((true, local_id.0), LinearState::Available);
        }
    }
}

fn track_id(state: &HashMap<(bool, u32), LinearState>, is_def: bool, idx: u32) -> LinearState {
    state.get(&(is_def, idx)).copied().unwrap_or(LinearState::Available)
}

fn mark_consumed(state: &mut HashMap<(bool, u32), LinearState>, is_def: bool, idx: u32) {
    state.insert((is_def, idx), LinearState::Consumed);
}

fn bind_pattern_type(
    cx: &mut InferCtxt,
    pattern: &gala_ast::Pattern,
    local_ids: &[LocalId],
    cursor: &mut usize,
    ty: Ty,
) {
    match pattern {
        gala_ast::Pattern::Ident(_) => {
            if let Some(local_id) = local_ids.get(*cursor).copied() {
                cx.bind_local(local_id, ty);
            }
            *cursor += 1;
        }
        gala_ast::Pattern::Tuple(patterns) => {
            let mut element_tys = Vec::with_capacity(patterns.len());
            for pattern in patterns {
                let element_ty = cx.fresh_var();
                bind_pattern_type(cx, pattern, local_ids, cursor, element_ty.clone());
                element_tys.push(element_ty);
            }
            cx.equate(ty, Ty::Tuple(element_tys));
        }
        gala_ast::Pattern::Struct { fields, .. } => {
            for (_, pattern) in fields {
                let field_ty = cx.fresh_var();
                bind_pattern_type(cx, pattern, local_ids, cursor, field_ty);
            }
        }
        gala_ast::Pattern::Wildcard | gala_ast::Pattern::Literal(_) => {}
    }
}

fn check_block_linearity(
    block: &HirBlock,
    state: &mut HashMap<(bool, u32), LinearState>,
    type_env: &mut HashMap<LocalId, Ty>,
    diags: &mut Diagnostics,
) {
    for stmt in &block.stmts {
        match stmt {
            HirStmt::Let(l) => {
                check_expr_linearity(&l.init, state, type_env, diags);
                let ty =
                    l.ty.as_ref()
                        .map(ast_type_to_ty)
                        .or_else(|| inferred_expr_type(&l.init, type_env))
                        .unwrap_or(Ty::TyError);
                let local_ids =
                    if l.local_ids.is_empty() { vec![l.local_id] } else { l.local_ids.clone() };
                let leaf_tys = match ty {
                    Ty::Tuple(items) => items,
                    other => vec![other],
                };
                for (index, local_id) in local_ids.into_iter().enumerate() {
                    let leaf_ty = leaf_tys.get(index).cloned().unwrap_or(Ty::TyError);
                    type_env.insert(local_id, leaf_ty.clone());
                    // The let binding makes the value available (it's fresh).
                    if is_linear_type(&leaf_ty) {
                        state.insert((true, local_id.0), LinearState::Available);
                    }
                }
            }
            HirStmt::Expr(e) => {
                check_expr_linearity(e, state, type_env, diags);
                if let HirExpr::Call(call) = e {
                    rebind_in_place_linearity(call, state);
                }
            }
            HirStmt::Return(Some(e)) => {
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
    type_env: &mut HashMap<LocalId, Ty>,
    diags: &mut Diagnostics,
) {
    match expr {
        HirExpr::Ident(_, def_id, _) => {
            let local_id = local_id_from_def_id(def_id).unwrap_or(LocalId(def_id.index));
            let lin_state = track_id(state, true, local_id.0);
            if lin_state == LinearState::Consumed {
                diags.push(
                    Diagnostic::error(
                        codes::USE_AFTER_CONSUME,
                        format!("value {:?} used after it was consumed", def_id),
                    )
                    .with_help("qubits are linear resources — they must be used exactly once")
                    .with_note("move the use before the consuming operation or restructure"),
                );
            }
            // Check if this is a quantum type by looking up in local env
            if let Some(ty) = type_env.get(&local_id) {
                if is_linear_type(ty) {
                    mark_consumed(state, true, local_id.0);
                }
            }
        }
        HirExpr::Binary(b) => {
            check_expr_linearity(&b.lhs, state, type_env, diags);
            check_expr_linearity(&b.rhs, state, type_env, diags);
        }
        HirExpr::Tuple(items) | HirExpr::Array(items) => {
            for item in items {
                check_expr_linearity(item, state, type_env, diags);
            }
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
            let saved_types = type_env.clone();
            check_block_linearity(&i.then_branch, state, type_env, diags);
            let then_state = state.clone();
            reject_branch_leaks(&saved_state, &then_state, diags);

            let mut else_state = saved_state.clone();
            let mut else_types = saved_types.clone();
            if let Some(else_) = &i.else_branch {
                check_expr_linearity(else_, &mut else_state, &mut else_types, diags);
                reject_branch_leaks(&saved_state, &else_state, diags);
            }

            for key in saved_state.keys() {
                let then_status = then_state.get(key).copied().unwrap_or(LinearState::Consumed);
                let else_status = else_state.get(key).copied().unwrap_or(LinearState::Consumed);
                if then_status != else_status {
                    diags.push(
                        Diagnostic::error(
                            codes::QUBIT_NOT_CONSUMED,
                            format!("linear value {:?} is consumed in only one branch", key.1),
                        )
                        .with_help("consume the value exactly once on every control-flow path"),
                    );
                }
            }

            state.clear();
            for key in saved_state.keys().chain(then_state.keys()).chain(else_state.keys()) {
                let before = saved_state.get(key).copied().unwrap_or(LinearState::Consumed);
                let then_status = then_state.get(key).copied().unwrap_or(before);
                let else_status = else_state.get(key).copied().unwrap_or(before);
                let merged = if then_status == LinearState::Available
                    && else_status == LinearState::Available
                {
                    LinearState::Available
                } else {
                    LinearState::Consumed
                };
                state.insert(*key, merged);
            }
        }
        HirExpr::Return(Some(e)) => {
            check_expr_linearity(e, state, type_env, diags);
        }
        _ => {}
    }
}

fn reject_branch_leaks(
    before: &HashMap<(bool, u32), LinearState>,
    branch: &HashMap<(bool, u32), LinearState>,
    diags: &mut Diagnostics,
) {
    for (key, status) in branch {
        if !before.contains_key(key) && *status == LinearState::Available {
            diags.push(Diagnostic::error(
                codes::QUBIT_NOT_CONSUMED,
                format!("linear value {:?} does not leave its branch consumed", key.1),
            ));
        }
    }
}

fn inferred_expr_type(expr: &HirExpr, type_env: &HashMap<LocalId, Ty>) -> Option<Ty> {
    match expr {
        HirExpr::Ident(_, def_id, _) => {
            let local_id = local_id_from_def_id(def_id).unwrap_or(LocalId(def_id.index));
            type_env.get(&local_id).cloned()
        }
        HirExpr::Call(call) => match call.callee.as_ref() {
            HirExpr::Ident(name, _, _) => match name.0.as_str() {
                "qubit" => Some(Ty::Qubit),
                "qubits" => Some(Ty::Qubits(1)),
                "measure" => Some(Ty::Measured(Box::new(Ty::Bool))),
                "h" | "x" | "y" | "z" | "s" | "t" | "rx" | "ry" | "rz" | "cx" | "cz" | "swap" => {
                    if matches!(name.0.as_str(), "cx" | "cz" | "swap") {
                        Some(Ty::Tuple(vec![Ty::Qubit, Ty::Qubit]))
                    } else {
                        Some(Ty::Qubit)
                    }
                }
                _ => None,
            },
            _ => None,
        },
        _ => None,
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
        assert_eq!(
            ast_type_to_ty(&Type::Path(gala_ast::Path {
                segments: vec![gala_ast::PathSegment {
                    ident: Ident::new("Int"),
                    type_args: Vec::new()
                }],
                span: Span::dummy(),
            })),
            Ty::Int
        );
        assert_eq!(ast_type_to_ty(&Type::Qubit), Ty::Qubit);
        assert_eq!(ast_type_to_ty(&Type::Qubits(Box::new(ConstExpr::Int(4)))), Ty::Qubits(4));
    }

    #[test]
    fn test_literal_types() {
        assert_eq!(literal_type(&Literal::Int(42)), Ty::Int);
        assert_eq!(literal_type(&Literal::Float(2.5)), Ty::Float);
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
                segments: vec![gala_ast::PathSegment {
                    ident: Ident::new("Int"),
                    type_args: Vec::new(),
                }],
                span: Span::dummy(),
            })),
            effect: gala_ast::Effect::Pure,
            is_extern: false,
            body: gala_hir::HirBlock {
                stmts: vec![gala_hir::HirStmt::Return(Some(Box::new(gala_hir::HirExpr::Literal(
                    Literal::Int(42),
                ))))],
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
            is_extern: false,
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
            params: vec![HirParam {
                local_id: qid,
                mutable: false,
                pattern: gala_ast::Pattern::Ident(Ident::new("q")),
                ty: gala_ast::Type::Qubit,
                span: Span::dummy(),
            }],
            ret_ty: None,
            effect: gala_ast::Effect::Quantum,
            is_extern: false,
            body: gala_hir::HirBlock {
                stmts: vec![
                    gala_hir::HirStmt::Expr(gala_hir::HirExpr::Ident(
                        Ident::new("q"),
                        dq.clone(),
                        false,
                    )),
                    gala_hir::HirStmt::Expr(gala_hir::HirExpr::Ident(
                        Ident::new("q"),
                        dq.clone(),
                        false,
                    )),
                ],
                tail: None,
                span: Span::dummy(),
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
            def_id: DefId { crate_id: cid.clone(), index: 0 },
            ident: Ident::new("ok"),
            generics: Vec::new(),
            params: vec![HirParam {
                local_id: xid,
                mutable: false,
                pattern: gala_ast::Pattern::Ident(Ident::new("x")),
                ty: gala_ast::Type::Path(gala_ast::Path {
                    segments: vec![gala_ast::PathSegment {
                        ident: Ident::new("Int"),
                        type_args: Vec::new(),
                    }],
                    span: Span::dummy(),
                }),
                span: Span::dummy(),
            }],
            ret_ty: None,
            effect: gala_ast::Effect::Pure,
            is_extern: false,
            body: gala_hir::HirBlock {
                stmts: vec![
                    gala_hir::HirStmt::Expr(gala_hir::HirExpr::Ident(
                        Ident::new("x"),
                        dx.clone(),
                        false,
                    )),
                    gala_hir::HirStmt::Expr(gala_hir::HirExpr::Ident(
                        Ident::new("x"),
                        dx.clone(),
                        false,
                    )),
                ],
                tail: None,
                span: Span::dummy(),
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
            def_id: DefId { crate_id: cid.clone(), index: 1 },
            ident: Ident::new("test"),
            generics: Vec::new(),
            params: vec![HirParam {
                local_id: qid,
                mutable: false,
                pattern: gala_ast::Pattern::Ident(Ident::new("q")),
                ty: gala_ast::Type::Qubit,
                span: Span::dummy(),
            }],
            ret_ty: None,
            effect: gala_ast::Effect::Quantum,
            is_extern: false,
            body: gala_hir::HirBlock {
                stmts: vec![
                    gala_hir::HirStmt::Expr(gala_hir::HirExpr::Call(gala_hir::HirCallExpr {
                        callee: Box::new(gala_hir::HirExpr::Ident(Ident::new("h"), dh, false)),
                        args: vec![gala_hir::HirExpr::Ident(Ident::new("q"), dq.clone(), false)],
                        span: Span::dummy(),
                    })),
                    gala_hir::HirStmt::Expr(gala_hir::HirExpr::Ident(
                        Ident::new("q"),
                        dq.clone(),
                        false,
                    )),
                ],
                tail: None,
                span: Span::dummy(),
            },
            span: Span::dummy(),
        };
        let diags = check_linearity(&hir_fn);
        assert!(diags.has_errors(), "qubit used after call should be an error");
    }
}
