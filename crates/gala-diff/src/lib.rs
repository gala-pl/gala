//! Differentiation lowering for Gala (parameter shift + reverse mode).

use gala_ast::Ident;
use gala_diagnostics::{codes, Diagnostic, Diagnostics};
use gala_gir::{
    Block, ClassicalOp, Constant, FuncId, GateKind, Gir, GirFunc, GirNode, NodeId, NodeKind,
    Terminator, TypeKind, ValueId,
};
use gala_span::Span;
use gala_types::Effect;
use std::collections::HashMap;

/// Lower reverse-mode gradient functions for pure GIR functions.
///
/// Each source parameter gets a sibling function named `name_grad_index`.
/// The sibling contains the original straight-line body followed by symbolic
/// derivative nodes and returns the selected parameter's derivative.
pub fn lower_gradients(gir: &mut Gir) {
    let source_ids: Vec<FuncId> = gir
        .funcs
        .values()
        .filter(|func| func.effect == Effect::Pure && !func.params.is_empty())
        .map(|func| func.id)
        .collect();

    for source_id in source_ids {
        let param_count = gir.funcs.get(&source_id).map_or(0, |func| func.params.len());
        for param_idx in 0..param_count {
            if let Ok((lowered, _)) = reverse_mode_autodiff_gir(gir, source_id, &[param_idx]) {
                *gir = lowered;
            }
        }
    }
}

/// Create a parameter-shifted copy of a GIR module.
///
/// Rotation gates whose angle is the selected function parameter receive an
/// explicit angle-plus-shift classical node. This preserves the parameter
/// shift in the IR instead of merely changing a function name.
pub fn param_shift_gradient_gir(
    gir: &Gir,
    func_id: FuncId,
    param_idx: usize,
    shift: f64,
) -> Result<Gir, Diagnostics> {
    let mut shifted = gir.clone();
    let Some(func) = shifted.funcs.get(&func_id).cloned() else {
        return Err(unsupported("parameter-shift source function does not exist"));
    };
    let Some(parameter) = func.params.get(param_idx).copied() else {
        return Err(unsupported("parameter-shift index is outside the function parameters"));
    };

    let mut next_node = shifted.nodes.keys().map(|id| id.0).max().unwrap_or(0).saturating_add(1);
    let mut next_value = shifted
        .nodes
        .values()
        .flat_map(|node| node.results.iter())
        .map(|value| value.0)
        .max()
        .unwrap_or(0)
        .saturating_add(1);

    let block_ids: Vec<_> = func.blocks.keys().copied().collect();
    for block_id in block_ids {
        let node_ids = shifted
            .funcs
            .get(&func_id)
            .and_then(|source| source.blocks.get(&block_id))
            .map(|block| block.nodes.clone())
            .unwrap_or_default();
        let mut insertions = Vec::new();

        for (index, node_id) in node_ids.iter().enumerate() {
            let Some(node) = shifted.nodes.get(node_id).cloned() else {
                return Err(unsupported("parameter-shift source references a missing GIR node"));
            };
            let is_rotation =
                matches!(node.kind, NodeKind::Gate(GateKind::Rx | GateKind::Ry | GateKind::Rz));
            if !is_rotation || node.operands.get(1).copied() != Some(parameter) {
                continue;
            }

            let shift_value = ValueId(next_value);
            next_value = next_value.saturating_add(1);
            let shift_node_id = NodeId(next_node);
            next_node = next_node.saturating_add(1);
            let add_value = ValueId(next_value);
            next_value = next_value.saturating_add(1);
            let add_node_id = NodeId(next_node);
            next_node = next_node.saturating_add(1);

            shifted.nodes.insert(
                shift_node_id,
                GirNode {
                    id: shift_node_id,
                    kind: NodeKind::Constant(Constant::Float(shift)),
                    operands: Vec::new(),
                    results: vec![shift_value],
                    span: node.span,
                },
            );
            shifted.nodes.insert(
                add_node_id,
                GirNode {
                    id: add_node_id,
                    kind: NodeKind::Classical(ClassicalOp::Add),
                    operands: vec![parameter, shift_value],
                    results: vec![add_value],
                    span: node.span,
                },
            );
            if let Some(gate) = shifted.nodes.get_mut(node_id) {
                if let Some(angle) = gate.operands.get_mut(1) {
                    *angle = add_value;
                }
            }
            insertions.push((index, vec![shift_node_id, add_node_id]));
        }

        if let Some(block) =
            shifted.funcs.get_mut(&func_id).and_then(|source| source.blocks.get_mut(&block_id))
        {
            let mut offset = 0;
            for (index, new_nodes) in insertions {
                let insertion_index = index + offset;
                block.nodes.splice(insertion_index..insertion_index, new_nodes.clone());
                offset += new_nodes.len();
            }
        }
    }

    Ok(shifted)
}

/// Build the two circuits required by the parameter-shift rule.
pub fn param_shift_circuits(
    gir: &Gir,
    func_id: FuncId,
    param_idx: usize,
    shift: f64,
) -> Result<(Gir, Gir), Diagnostics> {
    Ok((
        param_shift_gradient_gir(gir, func_id, param_idx, shift)?,
        param_shift_gradient_gir(gir, func_id, param_idx, -shift)?,
    ))
}

/// Reverse-mode autodiff over a straight-line classical GIR function.
///
/// The returned module contains one new gradient function per requested
/// parameter and the IDs of those functions in the same order as wrt.
pub fn reverse_mode_autodiff_gir(
    gir: &Gir,
    func_id: FuncId,
    wrt: &[usize],
) -> Result<(Gir, Vec<FuncId>), Diagnostics> {
    let mut lowered = gir.clone();
    let Some(source) = gir.funcs.get(&func_id) else {
        return Err(unsupported("autodiff source function does not exist"));
    };
    if source.blocks.len() != 1 {
        return Err(unsupported("autodiff currently requires a single basic block"));
    }
    let Some(source_block) = source.blocks.values().next() else {
        return Err(unsupported("autodiff source function has no basic block"));
    };
    let Some(Terminator::Return(Some(output_value))) = &source_block.terminator else {
        return Err(unsupported("autodiff source must return a value"));
    };
    let source_nodes = source_block.nodes.clone();

    let mut next_node = lowered.nodes.keys().map(|id| id.0).max().unwrap_or(0).saturating_add(1);
    let mut next_value = lowered
        .nodes
        .values()
        .flat_map(|node| node.results.iter())
        .map(|value| value.0)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    let mut next_func = lowered.funcs.keys().map(|id| id.0).max().unwrap_or(0).saturating_add(1);
    let mut gradient_ids = Vec::new();

    for &param_idx in wrt {
        if param_idx >= source.params.len() {
            return Err(unsupported("autodiff parameter index is outside the function parameters"));
        }

        // Prefer the declared parameter/return types over the presence of a
        // float constant in the body. A function such as `fn f(x: Float) ->
        // Float { x * x }` has no float literal, but its derivative still
        // needs float zero/one constants.
        let derivative_is_float = source
            .param_tys
            .get(param_idx)
            .and_then(|type_id| gir.types.get(type_id))
            .is_some_and(|ty| matches!(&ty.kind, TypeKind::Float))
            || source
                .ret_ty
                .and_then(|type_id| gir.types.get(&type_id))
                .is_some_and(|ty| matches!(&ty.kind, TypeKind::Float))
            || source_nodes.iter().any(|node_id| {
                matches!(
                    gir.nodes.get(node_id).map(|node| &node.kind),
                    Some(NodeKind::Constant(Constant::Float(_)))
                )
            });

        let mut block = source_block.clone();
        let mut derivatives = HashMap::<ValueId, ValueId>::new();
        let zero = append_constant(
            &mut lowered,
            &mut block,
            &mut next_node,
            &mut next_value,
            if derivative_is_float { Constant::Float(0.0) } else { Constant::Int(0) },
        );
        let one = append_constant(
            &mut lowered,
            &mut block,
            &mut next_node,
            &mut next_value,
            if derivative_is_float { Constant::Float(1.0) } else { Constant::Int(1) },
        );
        for (index, parameter) in source.params.iter().enumerate() {
            derivatives.insert(*parameter, if index == param_idx { one } else { zero });
        }

        for node_id in &source_nodes {
            let Some(node) = gir.nodes.get(node_id) else {
                return Err(unsupported("autodiff source references a missing GIR node"));
            };
            let Some(result) = node.results.first().copied() else {
                continue;
            };
            let derivative = match &node.kind {
                NodeKind::Constant(_) => zero,
                NodeKind::Classical(op) => {
                    let lhs = required_operand(&node.operands, 0)?;
                    let lhs_derivative = derivatives.get(&lhs).copied().unwrap_or(zero);
                    if !matches!(op, ClassicalOp::Neg | ClassicalOp::Not)
                        && node.operands.get(1).is_none()
                    {
                        return Err(unsupported(
                            "autodiff classical operation is missing an operand",
                        ));
                    }
                    match op {
                        ClassicalOp::Add => {
                            let rhs = required_operand(&node.operands, 1)?;
                            emit_binary(
                                &mut lowered,
                                &mut block,
                                &mut next_node,
                                &mut next_value,
                                ClassicalOp::Add,
                                lhs_derivative,
                                derivatives.get(&rhs).copied().unwrap_or(zero),
                            )
                        }
                        ClassicalOp::Sub => {
                            let rhs = required_operand(&node.operands, 1)?;
                            emit_binary(
                                &mut lowered,
                                &mut block,
                                &mut next_node,
                                &mut next_value,
                                ClassicalOp::Sub,
                                lhs_derivative,
                                derivatives.get(&rhs).copied().unwrap_or(zero),
                            )
                        }
                        ClassicalOp::Mul => {
                            let rhs = required_operand(&node.operands, 1)?;
                            let da_b = emit_binary(
                                &mut lowered,
                                &mut block,
                                &mut next_node,
                                &mut next_value,
                                ClassicalOp::Mul,
                                lhs_derivative,
                                rhs,
                            );
                            let a_db = emit_binary(
                                &mut lowered,
                                &mut block,
                                &mut next_node,
                                &mut next_value,
                                ClassicalOp::Mul,
                                lhs,
                                derivatives.get(&rhs).copied().unwrap_or(zero),
                            );
                            emit_binary(
                                &mut lowered,
                                &mut block,
                                &mut next_node,
                                &mut next_value,
                                ClassicalOp::Add,
                                da_b,
                                a_db,
                            )
                        }
                        ClassicalOp::Div => {
                            let rhs = required_operand(&node.operands, 1)?;
                            let da_b = emit_binary(
                                &mut lowered,
                                &mut block,
                                &mut next_node,
                                &mut next_value,
                                ClassicalOp::Mul,
                                lhs_derivative,
                                rhs,
                            );
                            let a_db = emit_binary(
                                &mut lowered,
                                &mut block,
                                &mut next_node,
                                &mut next_value,
                                ClassicalOp::Mul,
                                lhs,
                                derivatives.get(&rhs).copied().unwrap_or(zero),
                            );
                            let numerator = emit_binary(
                                &mut lowered,
                                &mut block,
                                &mut next_node,
                                &mut next_value,
                                ClassicalOp::Sub,
                                da_b,
                                a_db,
                            );
                            let denominator = emit_binary(
                                &mut lowered,
                                &mut block,
                                &mut next_node,
                                &mut next_value,
                                ClassicalOp::Mul,
                                rhs,
                                rhs,
                            );
                            emit_binary(
                                &mut lowered,
                                &mut block,
                                &mut next_node,
                                &mut next_value,
                                ClassicalOp::Div,
                                numerator,
                                denominator,
                            )
                        }
                        ClassicalOp::Neg => emit_unary(
                            &mut lowered,
                            &mut block,
                            &mut next_node,
                            &mut next_value,
                            ClassicalOp::Neg,
                            lhs_derivative,
                        ),
                        _ => zero,
                    }
                }
                NodeKind::Alloc(_)
                | NodeKind::Free { .. }
                | NodeKind::Tuple { .. }
                | NodeKind::Extract { .. }
                | NodeKind::Gate(_)
                | NodeKind::Measure { .. }
                | NodeKind::Call { .. }
                | NodeKind::Phi { .. } => {
                    return Err(unsupported("autodiff encountered a non-classical GIR node"));
                }
            };
            derivatives.insert(result, derivative);
        }

        let Some(gradient_value) = derivatives.get(output_value).copied() else {
            return Err(unsupported("autodiff could not derive the returned value"));
        };
        block.terminator = Some(Terminator::Return(Some(gradient_value)));
        let gradient_id = FuncId(next_func);
        next_func = next_func.saturating_add(1);
        let mut gradient_func = source.clone();
        gradient_func.id = gradient_id;
        gradient_func.name = Ident::new(format!("{}_grad_{}", source.name, param_idx));
        gradient_func.blocks.clear();
        gradient_func.blocks.insert(block.id, block);
        gradient_func.entry_block = source.entry_block;
        lowered.funcs.insert(gradient_id, gradient_func);
        gradient_ids.push(gradient_id);
    }

    Ok((lowered, gradient_ids))
}

fn append_constant(
    gir: &mut Gir,
    block: &mut Block,
    next_node: &mut u32,
    next_value: &mut u32,
    constant: Constant,
) -> ValueId {
    let value = ValueId(*next_value);
    *next_value = next_value.saturating_add(1);
    let node_id = NodeId(*next_node);
    *next_node = next_node.saturating_add(1);
    block.nodes.push(node_id);
    gir.nodes.insert(
        node_id,
        GirNode {
            id: node_id,
            kind: NodeKind::Constant(constant),
            operands: Vec::new(),
            results: vec![value],
            span: Span::dummy(),
        },
    );
    value
}

fn emit_binary(
    gir: &mut Gir,
    block: &mut Block,
    next_node: &mut u32,
    next_value: &mut u32,
    op: ClassicalOp,
    lhs: ValueId,
    rhs: ValueId,
) -> ValueId {
    let value = ValueId(*next_value);
    *next_value = next_value.saturating_add(1);
    let node_id = NodeId(*next_node);
    *next_node = next_node.saturating_add(1);
    block.nodes.push(node_id);
    gir.nodes.insert(
        node_id,
        GirNode {
            id: node_id,
            kind: NodeKind::Classical(op),
            operands: vec![lhs, rhs],
            results: vec![value],
            span: Span::dummy(),
        },
    );
    value
}

fn emit_unary(
    gir: &mut Gir,
    block: &mut Block,
    next_node: &mut u32,
    next_value: &mut u32,
    op: ClassicalOp,
    operand: ValueId,
) -> ValueId {
    let value = ValueId(*next_value);
    *next_value = next_value.saturating_add(1);
    let node_id = NodeId(*next_node);
    *next_node = next_node.saturating_add(1);
    block.nodes.push(node_id);
    gir.nodes.insert(
        node_id,
        GirNode {
            id: node_id,
            kind: NodeKind::Classical(op),
            operands: vec![operand],
            results: vec![value],
            span: Span::dummy(),
        },
    );
    value
}

fn unsupported(message: &str) -> Diagnostics {
    let mut diagnostics = Diagnostics::new();
    diagnostics.push(Diagnostic::error(codes::BACKEND_CAPABILITY_MISMATCH, message));
    diagnostics
}

fn required_operand(operands: &[ValueId], index: usize) -> Result<ValueId, Diagnostics> {
    operands
        .get(index)
        .copied()
        .ok_or_else(|| unsupported("autodiff source node is missing an operand"))
}

/// Compatibility entry point for callers that only have a function view.
/// A semantic shift requires the owning GIR node arena; fail explicitly
/// instead of returning an unchanged function that looks transformed.
pub fn param_shift_gradient(
    _circuit: &GirFunc,
    _param_idx: usize,
    _shift: f64,
) -> Result<GirFunc, Diagnostics> {
    Err(unsupported("parameter shift requires the owning GIR module; use param_shift_gradient_gir"))
}

/// Compatibility entry point for callers that only have a function view.
/// Reverse mode needs node payloads and therefore cannot be performed on a
/// detached `GirFunc`.
pub fn reverse_mode_autodiff(_func: &GirFunc, _wrt: &[usize]) -> Result<GirFunc, Diagnostics> {
    Err(unsupported("reverse mode requires the owning GIR module; use reverse_mode_autodiff_gir"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gala_gir::{BlockId, GirNode};

    fn rotation_gir() -> (Gir, FuncId) {
        let mut gir = Gir::default();
        let mut func = GirFunc::new_test();
        func.params = vec![ValueId(10)];
        let block = func.blocks.get_mut(&BlockId(0)).expect("entry block exists");
        block.nodes = vec![NodeId(0)];
        block.terminator = Some(Terminator::Return(None));
        gir.funcs.insert(func.id, func);
        gir.nodes.insert(
            NodeId(0),
            GirNode {
                id: NodeId(0),
                kind: NodeKind::Gate(GateKind::Rz),
                operands: vec![ValueId(1), ValueId(10)],
                results: vec![ValueId(2)],
                span: Span::dummy(),
            },
        );
        (gir, FuncId(0))
    }

    #[test]
    fn test_lower_empty() {
        lower_gradients(&mut Gir::default());
    }

    #[test]
    fn test_param_shift_rewrites_rotation_angle() {
        let (gir, func_id) = rotation_gir();
        let shifted = param_shift_gradient_gir(&gir, func_id, 0, 0.5).expect("shift should lower");
        let block = &shifted.funcs[&func_id].blocks[&BlockId(0)];
        assert_eq!(block.nodes.len(), 3);
        let gate = shifted.nodes.get(&NodeId(0)).expect("gate exists");
        assert_ne!(gate.operands[1], ValueId(10));
    }

    #[test]
    fn test_param_shift_builds_plus_and_minus_circuits() {
        let (gir, func_id) = rotation_gir();
        let (plus, minus) = param_shift_circuits(&gir, func_id, 0, 0.5).expect("shift circuits");
        let (
            NodeKind::Constant(Constant::Float(plus_shift)),
            NodeKind::Constant(Constant::Float(minus_shift)),
        ) = (&plus.nodes[&NodeId(1)].kind, &minus.nodes[&NodeId(1)].kind)
        else {
            panic!("parameter shift should create float constants");
        };
        assert_eq!(*plus_shift, -*minus_shift);
    }

    #[test]
    fn test_reverse_mode_autodiff_integer_expression() {
        let mut gir = Gir::default();
        let mut func = GirFunc::new_test();
        func.params = vec![ValueId(0)];
        let block = func.blocks.get_mut(&BlockId(0)).expect("entry block exists");
        block.nodes = vec![NodeId(0), NodeId(1)];
        block.terminator = Some(Terminator::Return(Some(ValueId(2))));
        gir.funcs.insert(func.id, func);
        gir.nodes.insert(
            NodeId(0),
            GirNode {
                id: NodeId(0),
                kind: NodeKind::Constant(Constant::Int(3)),
                operands: Vec::new(),
                results: vec![ValueId(1)],
                span: Span::dummy(),
            },
        );
        gir.nodes.insert(
            NodeId(1),
            GirNode {
                id: NodeId(1),
                kind: NodeKind::Classical(ClassicalOp::Add),
                operands: vec![ValueId(0), ValueId(1)],
                results: vec![ValueId(2)],
                span: Span::dummy(),
            },
        );
        let (lowered, ids) = reverse_mode_autodiff_gir(&gir, FuncId(0), &[0]).expect("autodiff");
        assert_eq!(ids.len(), 1);
        let gradient = &lowered.funcs[&ids[0]];
        assert!(matches!(
            gradient.blocks[&BlockId(0)].terminator,
            Some(Terminator::Return(Some(_)))
        ));
        assert!(lowered
            .nodes
            .values()
            .any(|node| matches!(node.kind, NodeKind::Constant(Constant::Int(1)))));
    }

    #[test]
    fn test_reverse_mode_autodiff_float_parameter_without_float_literal() {
        let mut gir = Gir::default();
        gir.types.insert(
            gala_gir::TypeId(0),
            gala_gir::GirType {
                id: gala_gir::TypeId(0),
                kind: TypeKind::Float,
                span: Span::dummy(),
            },
        );
        let mut func = GirFunc::new_test();
        func.params = vec![ValueId(0)];
        func.param_tys = vec![gala_gir::TypeId(0)];
        func.ret_ty = Some(gala_gir::TypeId(0));
        let block = func.blocks.get_mut(&BlockId(0)).expect("entry block exists");
        block.terminator = Some(Terminator::Return(Some(ValueId(0))));
        gir.funcs.insert(func.id, func);

        let (lowered, ids) = reverse_mode_autodiff_gir(&gir, FuncId(0), &[0]).expect("autodiff");
        assert_eq!(ids.len(), 1);
        assert!(lowered.nodes.values().any(|node| {
            matches!(node.kind, NodeKind::Constant(Constant::Float(value)) if value == 1.0)
        }));
    }

    #[test]
    fn test_compatibility_wrappers_keep_function_shape() {
        let func = GirFunc::new_test();
        assert!(param_shift_gradient(&func, 0, 0.5).is_err());
        assert!(reverse_mode_autodiff(&func, &[0]).is_err());
    }
}
