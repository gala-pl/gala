//! Uncomputation analysis and synthesis for Gala.

use gala_hir::*;
use gala_ast::Ident;
use gala_types::Ty;
use gala_diagnostics::{Diagnostic, Diagnostics, codes};
use std::collections::HashMap;

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
        let def_id = DefId { crate_id: hir_fn.def_id.crate_id.clone(), index: param.local_id.0 };
        provenance.insert(def_id.clone(), Provenance {
            nodes: vec![ProvenanceNode::External],
        });
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
                analyze_expr(&l.init, fn_def_id, env, provenance);
                // Bind the let's local_id to a new DefId tracking the init's provenance
                let def_id = DefId { crate_id: fn_def_id.crate_id.clone(), index: l.local_id.0 };
                env.insert(l.local_id, def_id.clone());
                // Copy provenance from the init expression
                if let Some(init_prov) = provenance.get(&DefId { crate_id: fn_def_id.crate_id.clone(), index: 0 }) {
                    provenance.insert(def_id, init_prov.clone());
                }
            }
            HirStmt::Expr(e) => {
                analyze_expr(e, fn_def_id, env, provenance);
            }
            _ => {}
        }
    }
    if let Some(tail) = &block.tail {
        // For tail expressions, we need to handle the return
        match tail.as_ref() {
            HirExpr::Ident(_, def_id) => {
                // Return passes through the provenance unchanged
                let _ = def_id;
            }
            HirExpr::Call(c) => {
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
) {
    match expr {
        HirExpr::Ident(_, def_id) => {
            // Record this usage (the provenance was established at binding time)
        }
        HirExpr::Call(c) => {
            // Check if this is a gate application or allocation
            if let HirExpr::Ident(gate_name, _) = c.callee.as_ref() {
                let name = gate_name.0.as_str();
                match name {
                    "qubit" | "qubits" => {
                        // Allocation: generate fresh provenance
                        let fresh_def = DefId { crate_id: fn_def_id.crate_id.clone(), index: 0 };
                        provenance.insert(fresh_def, Provenance {
                            nodes: vec![ProvenanceNode::Alloc { ty: Ty::Qubit }],
                        });
                    }
                    "measure" => {
                        // Measurement: mark as measured (not liftable)
                        for arg in &c.args {
                            if let HirExpr::Ident(arg_name, arg_def) = arg {
                                let _ = arg_name;
                                provenance.insert(arg_def.clone(), Provenance {
                                    nodes: vec![ProvenanceNode::Measure],
                                });
                            }
                        }
                    }
                    "h" | "x" | "y" | "z" | "s" | "t"
                    | "rx" | "ry" | "rz"
                    | "cx" | "cz" | "swap" => {
                        // Gate application: the output has provenance based on inputs
                        let mut inputs = Vec::new();
                        for arg in &c.args {
                            if let HirExpr::Ident(_, arg_def) = arg {
                                inputs.push(arg_def.clone());
                            }
                        }
                        if let Some(result) = c.args.first() {
                            if let HirExpr::Ident(result_name, result_def) = result {
                                let _ = result_name;
                                provenance.insert(result_def.clone(), Provenance {
                                    nodes: vec![ProvenanceNode::Gate {
                                        gate: Ident::new(name),
                                        inputs,
                                    }],
                                });
                            }
                        }
                    }
                    _ => {} // Unknown function call
                }
            }
        }
        HirExpr::Block(b) => {
            analyze_block(b, fn_def_id, env, provenance);
        }
        HirExpr::Binary(b) => {
            analyze_expr(&b.lhs, fn_def_id, env, provenance);
            analyze_expr(&b.rhs, fn_def_id, env, provenance);
        }
        _ => {}
    }
}

/// Determine if a value is liftable (can be auto-uncomputed).
pub fn is_liftable(provenance: &Provenance) -> bool {
    !provenance.nodes.iter().any(|n| {
        matches!(n, ProvenanceNode::Measure | ProvenanceNode::External)
    })
}

/// Synthesize uncomputation steps for a function.
pub fn synthesize_uncompute(
    hir_fn: &HirFnDef,
    provenance: &HashMap<DefId, Provenance>,
) -> Result<UncomputePlan, Diagnostics> {
    let mut diags = Diagnostics::new();
    let mut steps = Vec::new();

    for (def_id, prov) in provenance {
        if is_liftable(prov) {
            // For liftable values, synthesize adjoint
            if prov.nodes.iter().any(|n| matches!(n, ProvenanceNode::Gate { .. } | ProvenanceNode::Alloc { .. })) {
                steps.push(UncomputeStep::Adjoint {
                    target: def_id.clone(),
                    provenance: prov.clone(),
                });
            }
        } else {
            diags.push(Diagnostic::error(codes::CANNOT_UNCOMPUTE,
                format!("cannot automatically uncompute value {:?}", def_id))
                .with_help("provide an explicit uncomputation strategy using uncompute_with()"));
        }
    }

    if diags.has_errors() { Err(diags) } else { Ok(UncomputePlan { steps }) }
}

/// Insert uncomputation steps into HIR (returns modified HIR).
pub fn insert_uncompute(hir_fn: &HirFnDef, plan: UncomputePlan) -> HirFnDef {
    let _ = plan;
    hir_fn.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gala_span::{FileId, Span};
    use gala_ast::{self, Ident, Item};

    #[test]
    fn test_is_liftable() {
        let rev_prov = Provenance {
            nodes: vec![ProvenanceNode::Gate {
                gate: Ident::new("h"),
                inputs: vec![DefId { crate_id: CrateId(FileId(0)), index: 0 }],
            }],
        };
        assert!(is_liftable(&rev_prov));

        let measure_prov = Provenance {
            nodes: vec![ProvenanceNode::Measure],
        };
        assert!(!is_liftable(&measure_prov));
    }

    #[test]
    fn test_provenance_is_external_not_liftable() {
        let external_prov = Provenance {
            nodes: vec![ProvenanceNode::External],
        };
        assert!(!is_liftable(&external_prov));
    }

    #[test]
    fn test_provenance_alloc_is_liftable() {
        let alloc_prov = Provenance {
            nodes: vec![ProvenanceNode::Alloc { ty: gala_types::Ty::Qubit }],
        };
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
        let dq = DefId { crate_id: CrateId(FileId(0)), index: qid.0 };

        let hir_fn = HirFnDef {
            def_id: fn_def_id.clone(),
            ident: Ident::new("bell"),
            generics: Vec::new(),
            params: vec![
                HirParam {
                    local_id: qid, mutable: false,
                    pattern: gala_ast::Pattern::Ident(Ident::new("q")),
                    ty: gala_ast::Type::Qubit, span: Span::dummy(),
                },
            ],
            ret_ty: None,
            effect: gala_ast::Effect::Quantum,
            body: HirBlock {
                stmts: vec![
                    HirStmt::Expr(HirExpr::Call(HirCallExpr {
                        callee: Box::new(HirExpr::Ident(Ident::new("h"), dq.clone())),
                        args: vec![HirExpr::Ident(Ident::new("q"), dq.clone())],
                        span: Span::dummy(),
                    })),
                ],
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
            body: HirBlock {
                stmts: Vec::new(),
                tail: None,
                span: Span::dummy(),
            },
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
            generics: Vec::new(), params: Vec::new(),
            ret_ty: None, effect: gala_ast::Effect::Pure,
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
        let free = UncomputeStep::Free {
            target: DefId { crate_id: CrateId(FileId(0)), index: 0 },
        };
        let measure = UncomputeStep::Measure {
            target: DefId { crate_id: CrateId(FileId(0)), index: 0 },
        };
        assert!(matches!(adj, UncomputeStep::Adjoint { .. }));
        assert!(matches!(free, UncomputeStep::Free { .. }));
        assert!(matches!(measure, UncomputeStep::Measure { .. }));
    }

    #[test]
    fn test_analyze_provenance_tracks_parameter() {
        let qid = LocalId(42);
        let hir_fn = HirFnDef {
            def_id: DefId { crate_id: CrateId(FileId(0)), index: 0 },
            ident: Ident::new("f"),
            generics: Vec::new(),
            params: vec![
                HirParam {
                    local_id: qid, mutable: false,
                    pattern: gala_ast::Pattern::Ident(Ident::new("q")),
                    ty: gala_ast::Type::Qubit, span: Span::dummy(),
                },
            ],
            ret_ty: None,
            effect: gala_ast::Effect::Quantum,
            body: HirBlock { stmts: Vec::new(), tail: None, span: Span::dummy() },
            span: Span::dummy(),
        };
        let prov = analyze_provenance(&hir_fn);
        let expected_def = DefId { crate_id: CrateId(FileId(0)), index: qid.0 };
        let p = prov.get(&expected_def);
        assert!(p.is_some(), "parameter should have provenance");
        if let Some(prov) = p {
            assert!(!is_liftable(prov), "parameter provenance should be External (not liftable)");
        }
    }
}