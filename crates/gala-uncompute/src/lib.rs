//! Uncomputation analysis and synthesis for Gala.

use gala_ast::Ident;
use gala_diagnostics::{codes, Diagnostic, Diagnostics};
use gala_hir::*;
use gala_types::Ty;
use std::collections::{HashMap, HashSet};

/// Provenance tracking for linear values.
#[derive(Debug, Clone)]
pub struct Provenance {
    pub nodes: Vec<ProvenanceNode>,
}

#[derive(Debug, Clone)]
pub enum ProvenanceNode {
    Gate { gate: Ident, inputs: Vec<DefId> },
    Alloc { ty: Ty },
    Measure,
    External,
}

/// Result of uncomputation analysis.
#[derive(Debug, Clone)]
pub struct UncomputePlan {
    pub steps: Vec<UncomputeStep>,
}

#[derive(Debug, Clone)]
pub enum UncomputeStep {
    Adjoint { target: DefId, provenance: Provenance },
    Free { target: DefId },
    Measure { target: DefId },
}

/// Analyze provenance of linear values in a HIR function.
/// Walks the expression tree and tracks how each value was produced.
pub fn analyze_provenance(hir_fn: &HirFnDef) -> HashMap<DefId, Provenance> {
    let mut provenance: HashMap<DefId, Provenance> = HashMap::new();
    let mut env: HashMap<LocalId, DefId> = HashMap::new();

    // Bind parameters as External (unknown provenance)
    for param in &hir_fn.params {
        let def_id = local_def_id(hir_fn.def_id.crate_id.0, param.local_id);
        provenance.insert(def_id.clone(), Provenance { nodes: vec![ProvenanceNode::External] });
        env.insert(param.local_id, def_id);
    }

    analyze_block(&hir_fn.body, &hir_fn.def_id, &mut env, &mut provenance);

    provenance
}

fn analyze_block(
    block: &HirBlock,
    fn_def_id: &DefId,
    env: &mut HashMap<LocalId, DefId>,
    provenance: &mut HashMap<DefId, Provenance>,
) {
    for stmt in &block.stmts {
        match stmt {
            HirStmt::Let(l) => {
                let init_prov = analyze_expr(&l.init, fn_def_id, env, provenance);
                // Bind the let's local_id to a new DefId tracking the init's provenance
                let def_id = local_def_id(fn_def_id.crate_id.0, l.local_id);
                env.insert(l.local_id, def_id.clone());
                if let Some(init_prov) = init_prov {
                    provenance.insert(def_id, init_prov);
                }
            }
            HirStmt::Expr(e) => {
                analyze_expr(e, fn_def_id, env, provenance);
            }
            HirStmt::Return(Some(e)) => {
                let _ = analyze_expr(e, fn_def_id, env, provenance);
            }
            _ => {}
        }
    }
    if let Some(tail) = &block.tail {
        // For tail expressions, we need to handle the return
        match tail.as_ref() {
            HirExpr::Ident(_, def_id, _) => {
                // Return passes through the provenance unchanged
                let _ = def_id;
            }
            HirExpr::Call(_c) => {
                // For calls, mark the result and consume inputs
                analyze_expr(tail, fn_def_id, env, provenance);
            }
            _ => {
                analyze_expr(tail, fn_def_id, env, provenance);
            }
        }
    }
}

fn analyze_expr(
    expr: &HirExpr,
    fn_def_id: &DefId,
    env: &mut HashMap<LocalId, DefId>,
    provenance: &mut HashMap<DefId, Provenance>,
) -> Option<Provenance> {
    match expr {
        HirExpr::Ident(_, def_id, _) => provenance.get(def_id).cloned(),
        HirExpr::Call(c) => {
            for arg in &c.args {
                let _ = analyze_expr(arg, fn_def_id, env, provenance);
            }
            // Check if this is a gate application or allocation
            if let HirExpr::Ident(gate_name, _, _) = c.callee.as_ref() {
                let name = gate_name.0.as_str();
                match name {
                    "qubit" | "qubits" => {
                        return Some(Provenance {
                            nodes: vec![ProvenanceNode::Alloc { ty: Ty::Qubit }],
                        });
                    }
                    "measure" => {
                        // Measurement: mark as measured (not liftable)
                        for arg in &c.args {
                            if let HirExpr::Ident(arg_name, arg_def, _) = arg {
                                let _ = arg_name;
                                provenance.insert(
                                    arg_def.clone(),
                                    Provenance { nodes: vec![ProvenanceNode::Measure] },
                                );
                            }
                        }
                        return Some(Provenance { nodes: vec![ProvenanceNode::Measure] });
                    }
                    "h" | "x" | "y" | "z" | "s" | "t" | "rx" | "ry" | "rz" | "cx" | "cz"
                    | "swap" => {
                        // Gate application: the output has provenance based on inputs
                        let mut inputs = Vec::new();
                        for arg in &c.args {
                            if let HirExpr::Ident(_, arg_def, _) = arg {
                                inputs.push(arg_def.clone());
                            }
                        }
                        if inputs.iter().any(|input| {
                            provenance.get(input).is_some_and(|input_provenance| {
                                input_provenance.nodes.iter().any(|node| {
                                    matches!(
                                        node,
                                        ProvenanceNode::Measure | ProvenanceNode::External
                                    )
                                })
                            })
                        }) {
                            return Some(Provenance { nodes: vec![ProvenanceNode::External] });
                        }
                        return Some(Provenance {
                            nodes: vec![ProvenanceNode::Gate { gate: Ident::new(name), inputs }],
                        });
                    }
                    _ => return Some(Provenance { nodes: vec![ProvenanceNode::External] }),
                }
            }
            None
        }
        HirExpr::Block(b) => {
            analyze_block(b, fn_def_id, env, provenance);
            b.tail.as_deref().and_then(|tail| analyze_expr(tail, fn_def_id, env, provenance))
        }
        HirExpr::Binary(b) => {
            let _ = analyze_expr(&b.lhs, fn_def_id, env, provenance);
            let _ = analyze_expr(&b.rhs, fn_def_id, env, provenance);
            None
        }
        _ => None,
    }
}

/// Determine if a value is liftable (can be auto-uncomputed).
pub fn is_liftable(provenance: &Provenance) -> bool {
    !provenance
        .nodes
        .iter()
        .any(|n| matches!(n, ProvenanceNode::Measure | ProvenanceNode::External))
}

/// Synthesize uncomputation steps for a function.
pub fn synthesize_uncompute(
    hir_fn: &HirFnDef,
    provenance: &HashMap<DefId, Provenance>,
) -> Result<UncomputePlan, Diagnostics> {
    let mut diags = Diagnostics::new();
    let mut steps = Vec::new();

    for stmt in &hir_fn.body.stmts {
        if let HirStmt::Return(Some(expr)) = stmt {
            reject_external_return(expr, provenance, &mut diags);
        }
    }
    if let Some(tail) = &hir_fn.body.tail {
        reject_external_return(tail, provenance, &mut diags);
    }

    let returned = returned_values(hir_fn);
    let mut entries: Vec<_> = provenance.iter().collect();
    entries.sort_by(|(left, _), (right, _)| {
        (right.crate_id.0 .0, right.index).cmp(&(left.crate_id.0 .0, left.index))
    });
    for (def_id, prov) in entries {
        // Inputs and measured values are consumed by the surrounding program;
        // they are not temporary ancillas that this pass should try to clean.
        // Only synthesized reversible values receive an adjoint step.
        if !returned.contains(def_id)
            && is_liftable(prov)
            && prov
                .nodes
                .iter()
                .any(|n| matches!(n, ProvenanceNode::Gate { .. } | ProvenanceNode::Alloc { .. }))
        {
            steps.push(UncomputeStep::Adjoint { target: def_id.clone(), provenance: prov.clone() });
        }
    }

    if diags.has_errors() {
        Err(diags)
    } else {
        Ok(UncomputePlan { steps })
    }
}

fn returned_values(hir_fn: &HirFnDef) -> HashSet<DefId> {
    let mut returned = HashSet::new();
    for stmt in &hir_fn.body.stmts {
        if let HirStmt::Return(Some(expr)) = stmt {
            collect_returned_values(expr, &mut returned);
        }
    }
    if let Some(tail) = &hir_fn.body.tail {
        collect_returned_values(tail, &mut returned);
    }
    returned
}

fn collect_returned_values(expr: &HirExpr, returned: &mut HashSet<DefId>) {
    match expr {
        HirExpr::Ident(_, def_id, _) => {
            returned.insert(def_id.clone());
        }
        HirExpr::Block(block) => {
            if let Some(tail) = &block.tail {
                collect_returned_values(tail, returned);
            }
        }
        HirExpr::If(if_expr) => {
            if let Some(tail) = &if_expr.then_branch.tail {
                collect_returned_values(tail, returned);
            }
            if let Some(else_branch) = &if_expr.else_branch {
                collect_returned_values(else_branch, returned);
            }
        }
        _ => {}
    }
}

fn reject_external_return(
    expr: &HirExpr,
    provenance: &HashMap<DefId, Provenance>,
    diags: &mut Diagnostics,
) {
    match expr {
        HirExpr::Ident(_, def_id, _)
            if provenance.get(def_id).is_some_and(|prov| {
                prov.nodes.iter().any(|node| matches!(node, ProvenanceNode::External))
            }) =>
        {
            diags.push(Diagnostic::error(
                codes::CANNOT_UNCOMPUTE,
                "cannot automatically uncompute an externally-owned linear value returned from a quantum function",
            ));
        }
        HirExpr::Block(block) => {
            if let Some(tail) = &block.tail {
                reject_external_return(tail, provenance, diags);
            }
        }
        HirExpr::If(if_expr) => {
            if let Some(tail) = &if_expr.then_branch.tail {
                reject_external_return(tail, provenance, diags);
            }
            if let Some(else_branch) = &if_expr.else_branch {
                reject_external_return(else_branch, provenance, diags);
            }
        }
        _ => {}
    }
}

/// Insert the currently representable uncomputation steps into HIR.
///
/// Unary self-inverse gates and explicit frees are representable in the
/// existing HIR. Gates whose adjoints require an angle or a missing inverse
/// builtin are left for the explicit strategy path rather than being guessed.
pub fn insert_uncompute(hir_fn: &HirFnDef, plan: UncomputePlan) -> HirFnDef {
    let mut lowered = hir_fn.clone();
    let cleanup: Vec<_> = plan.steps.iter().filter_map(|step| step_to_expr(step, hir_fn)).collect();
    if cleanup.is_empty() {
        return lowered;
    }

    let mut statements = Vec::with_capacity(lowered.body.stmts.len() + cleanup.len());
    for statement in std::mem::take(&mut lowered.body.stmts) {
        if matches!(statement, HirStmt::Return(_)) {
            statements.extend(cleanup.iter().cloned().map(HirStmt::Expr));
        }
        statements.push(statement);
    }
    if lowered.body.tail.is_some() {
        statements.extend(cleanup.iter().cloned().map(HirStmt::Expr));
    }
    lowered.body.stmts = statements;
    lowered
}

fn step_to_expr(step: &UncomputeStep, hir_fn: &HirFnDef) -> Option<HirExpr> {
    let (target, gate) = match step {
        UncomputeStep::Free { target } => (target, "drop"),
        UncomputeStep::Adjoint { target, provenance } => {
            let ProvenanceNode::Gate { gate, .. } = provenance.nodes.last()? else {
                return match provenance.nodes.last() {
                    Some(ProvenanceNode::Alloc { .. }) => step_to_drop(target, hir_fn),
                    _ => None,
                };
            };
            let inverse = match gate.0.as_str() {
                "h" | "x" | "y" | "z" => gate.0.as_str(),
                _ => return None,
            };
            (target, inverse)
        }
        UncomputeStep::Measure { target } => return step_to_drop(target, hir_fn),
    };
    let builtin = builtin_for_name(gate)?;
    Some(HirExpr::Call(HirCallExpr {
        callee: Box::new(HirExpr::Ident(
            Ident::new(gate),
            builtin_def_id(hir_fn.def_id.crate_id.0, builtin),
            false,
        )),
        args: vec![value_expr(target)],
        span: hir_fn.span,
    }))
}

fn step_to_drop(target: &DefId, hir_fn: &HirFnDef) -> Option<HirExpr> {
    Some(HirExpr::Call(HirCallExpr {
        callee: Box::new(HirExpr::Ident(
            Ident::new("drop"),
            builtin_def_id(hir_fn.def_id.crate_id.0, BuiltinDef::Drop),
            false,
        )),
        args: vec![value_expr(target)],
        span: hir_fn.span,
    }))
}

fn value_expr(def_id: &DefId) -> HirExpr {
    HirExpr::Ident(Ident::new(format!("_local{}", def_id.index)), def_id.clone(), false)
}

fn builtin_for_name(name: &str) -> Option<BuiltinDef> {
    Some(match name {
        "h" => BuiltinDef::HGate,
        "x" => BuiltinDef::XGate,
        "y" => BuiltinDef::YGate,
        "z" => BuiltinDef::ZGate,
        "drop" => BuiltinDef::Drop,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gala_ast::{self, Ident};
    use gala_span::{FileId, Span};

    #[test]
    fn test_is_liftable() {
        let rev_prov = Provenance {
            nodes: vec![ProvenanceNode::Gate {
                gate: Ident::new("h"),
                inputs: vec![DefId { crate_id: CrateId(FileId(0)), index: 0 }],
            }],
        };
        assert!(is_liftable(&rev_prov));

        let measure_prov = Provenance { nodes: vec![ProvenanceNode::Measure] };
        assert!(!is_liftable(&measure_prov));
    }

    #[test]
    fn test_provenance_is_external_not_liftable() {
        let external_prov = Provenance { nodes: vec![ProvenanceNode::External] };
        assert!(!is_liftable(&external_prov));
    }

    #[test]
    fn test_provenance_alloc_is_liftable() {
        let alloc_prov =
            Provenance { nodes: vec![ProvenanceNode::Alloc { ty: gala_types::Ty::Qubit }] };
        assert!(is_liftable(&alloc_prov));
    }

    #[test]
    fn test_uncompute_plan_creation() {
        let plan = UncomputePlan { steps: Vec::new() };
        assert!(plan.steps.is_empty());
    }

    #[test]
    fn test_analyze_provenance_for_fn_with_gates() {
        let fn_def_id = DefId { crate_id: CrateId(FileId(0)), index: 0 };
        let qid = LocalId(0);
        let dq = local_def_id(FileId(0), qid);

        let hir_fn = HirFnDef {
            def_id: fn_def_id.clone(),
            ident: Ident::new("bell"),
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
            body: HirBlock {
                stmts: vec![HirStmt::Expr(HirExpr::Call(HirCallExpr {
                    callee: Box::new(HirExpr::Ident(Ident::new("h"), dq.clone(), false)),
                    args: vec![HirExpr::Ident(Ident::new("q"), dq.clone(), false)],
                    span: Span::dummy(),
                }))],
                tail: None,
                span: Span::dummy(),
            },
            span: Span::dummy(),
        };

        let prov = analyze_provenance(&hir_fn);
        // The parameter has External provenance
        assert!(prov.contains_key(&dq));
    }

    #[test]
    fn test_analyze_provenance_empty_fn() {
        let hir_fn = HirFnDef {
            def_id: DefId { crate_id: CrateId(FileId(0)), index: 0 },
            ident: Ident::new("empty"),
            generics: Vec::new(),
            params: Vec::new(),
            ret_ty: None,
            effect: gala_ast::Effect::Pure,
            is_extern: false,
            body: HirBlock { stmts: Vec::new(), tail: None, span: Span::dummy() },
            span: Span::dummy(),
        };
        let prov = analyze_provenance(&hir_fn);
        assert!(prov.is_empty());
    }

    #[test]
    fn test_synthesize_uncompute_empty() {
        let hir_fn = HirFnDef {
            def_id: DefId { crate_id: CrateId(FileId(0)), index: 0 },
            ident: Ident::new("f"),
            generics: Vec::new(),
            params: Vec::new(),
            ret_ty: None,
            effect: gala_ast::Effect::Pure,
            is_extern: false,
            body: HirBlock { stmts: Vec::new(), tail: None, span: Span::dummy() },
            span: Span::dummy(),
        };
        let prov = HashMap::new();
        let result = synthesize_uncompute(&hir_fn, &prov);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(plan.steps.is_empty());
    }

    #[test]
    fn test_uncompute_step_variants() {
        let adj = UncomputeStep::Adjoint {
            target: DefId { crate_id: CrateId(FileId(0)), index: 0 },
            provenance: Provenance { nodes: Vec::new() },
        };
        let free = UncomputeStep::Free { target: DefId { crate_id: CrateId(FileId(0)), index: 0 } };
        let measure =
            UncomputeStep::Measure { target: DefId { crate_id: CrateId(FileId(0)), index: 0 } };
        assert!(matches!(adj, UncomputeStep::Adjoint { .. }));
        assert!(matches!(free, UncomputeStep::Free { .. }));
        assert!(matches!(measure, UncomputeStep::Measure { .. }));
    }

    #[test]
    fn test_insert_uncompute_adds_cleanup_before_return() {
        let target = local_def_id(FileId(0), LocalId(7));
        let hir_fn = HirFnDef {
            def_id: DefId { crate_id: CrateId(FileId(0)), index: 0 },
            ident: Ident::new("f"),
            generics: Vec::new(),
            params: Vec::new(),
            ret_ty: None,
            effect: gala_ast::Effect::Quantum,
            is_extern: false,
            body: HirBlock {
                stmts: vec![HirStmt::Return(Some(Box::new(value_expr(&target))))],
                tail: None,
                span: Span::dummy(),
            },
            span: Span::dummy(),
        };
        let lowered = insert_uncompute(
            &hir_fn,
            UncomputePlan { steps: vec![UncomputeStep::Free { target }] },
        );
        assert_eq!(lowered.body.stmts.len(), 2);
        assert!(matches!(lowered.body.stmts[0], HirStmt::Expr(HirExpr::Call(_))));
        assert!(matches!(lowered.body.stmts[1], HirStmt::Return(_)));
    }

    #[test]
    fn test_analyze_provenance_tracks_parameter() {
        let qid = LocalId(42);
        let hir_fn = HirFnDef {
            def_id: DefId { crate_id: CrateId(FileId(0)), index: 0 },
            ident: Ident::new("f"),
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
            body: HirBlock { stmts: Vec::new(), tail: None, span: Span::dummy() },
            span: Span::dummy(),
        };
        let prov = analyze_provenance(&hir_fn);
        let expected_def = local_def_id(FileId(0), qid);
        let p = prov.get(&expected_def);
        assert!(p.is_some(), "parameter should have provenance");
        if let Some(prov) = p {
            assert!(!is_liftable(prov), "parameter provenance should be External (not liftable)");
        }
    }
}
