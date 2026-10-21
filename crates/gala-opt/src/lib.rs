//! GIR optimization passes for Gala.

use gala_gir::{
    ClassicalOp, Constant, FuncId, GateKind, Gir, GirNode, NodeId, NodeKind, Terminator, ValueId,
};
use std::collections::{HashMap, HashSet};

/// Run the optimization passes that are currently semantics-preserving for the
/// GIR data model. Passes operate on the shared node arena as well as block
/// node lists, so they cannot silently optimize a discarded copy of a node.
pub fn optimize_gir(gir: &mut Gir) {
    let mut function_ids: Vec<FuncId> = gir.funcs.keys().copied().collect();
    function_ids.sort_by_key(|id| id.0);
    for func_id in function_ids {
        cancel_adjacent_gates(gir, func_id);
        constant_folding(gir, func_id);
        dead_code_elimination(gir, func_id);
    }
}

fn cancel_adjacent_gates(gir: &mut Gir, func_id: FuncId) {
    let block_ids: Vec<_> = gir
        .funcs
        .get(&func_id)
        .map(|func| func.blocks.keys().copied().collect())
        .unwrap_or_default();
    for block_id in block_ids {
        let node_ids = gir
            .funcs
            .get(&func_id)
            .and_then(|func| func.blocks.get(&block_id))
            .map(|block| block.nodes.clone())
            .unwrap_or_default();

        let mut remove = HashSet::new();
        let mut index = 0;
        while index + 1 < node_ids.len() {
            let first_id = node_ids[index];
            let second_id = node_ids[index + 1];
            let Some(first) = gir.nodes.get(&first_id).cloned() else {
                index += 1;
                continue;
            };
            let Some(second) = gir.nodes.get(&second_id).cloned() else {
                index += 1;
                continue;
            };
            if !can_cancel(&first, &second) {
                index += 1;
                continue;
            }

            if let Some(target) = first.operands.first().copied() {
                for result in first.results.iter().chain(second.results.iter()) {
                    replace_value(gir, func_id, *result, target);
                }
            }
            remove.insert(first_id);
            remove.insert(second_id);
            index += 2;
        }

        if !remove.is_empty() {
            if let Some(func) = gir.funcs.get_mut(&func_id) {
                if let Some(block) = func.blocks.get_mut(&block_id) {
                    block.nodes.retain(|node_id| !remove.contains(node_id));
                }
            }
            for node_id in remove {
                gir.nodes.remove(&node_id);
            }
        }
    }
}

fn can_cancel(first: &GirNode, second: &GirNode) -> bool {
    let (NodeKind::Gate(first_gate), NodeKind::Gate(second_gate)) = (&first.kind, &second.kind)
    else {
        return false;
    };
    if !matches!(
        first_gate,
        GateKind::H
            | GateKind::X
            | GateKind::Y
            | GateKind::Z
            | GateKind::CX
            | GateKind::CZ
            | GateKind::SWAP
    ) || first_gate != second_gate
    {
        return false;
    }
    let same_target =
        first.results.first().zip(second.operands.first()).is_some_and(|(a, b)| a == b);
    if first.operands.len() == 1 && second.operands.len() == 1 {
        same_target
    } else {
        same_target
            && first.operands.len() == second.operands.len()
            && first.operands.get(1..) == second.operands.get(1..)
    }
}

fn replace_value(gir: &mut Gir, func_id: FuncId, old: ValueId, new: ValueId) {
    if old == new {
        return;
    }
    for node in gir.nodes.values_mut() {
        for operand in &mut node.operands {
            if *operand == old {
                *operand = new;
            }
        }
        if let NodeKind::Measure { target } = &mut node.kind {
            if *target == old {
                *target = new;
            }
        }
        if let NodeKind::Free { target } = &mut node.kind {
            if *target == old {
                *target = new;
            }
        }
        if let NodeKind::Phi { incoming } = &mut node.kind {
            for (_, value) in incoming {
                if *value == old {
                    *value = new;
                }
            }
        }
    }
    if let Some(func) = gir.funcs.get_mut(&func_id) {
        for block in func.blocks.values_mut() {
            if let Some(terminator) = &mut block.terminator {
                match terminator {
                    Terminator::Return(value) if *value == Some(old) => *value = Some(new),
                    Terminator::CondBranch { cond, .. } if *cond == old => *cond = new,
                    _ => {}
                }
            }
        }
    }
}

fn constant_folding(gir: &mut Gir, func_id: FuncId) {
    let node_ids: Vec<NodeId> = gir
        .funcs
        .get(&func_id)
        .map(|func| func.blocks.values().flat_map(|block| block.nodes.clone()).collect())
        .unwrap_or_default();
    let mut constants = HashMap::<ValueId, Constant>::new();

    for node_id in node_ids {
        let Some(node) = gir.nodes.get(&node_id).cloned() else {
            continue;
        };
        match node.kind {
            NodeKind::Constant(constant) => {
                if let Some(result) = node.results.first() {
                    constants.insert(*result, constant);
                }
            }
            NodeKind::Classical(op) => {
                let folded = fold_classical(op, &node.operands, &constants);
                if let Some(constant) = folded {
                    if let Some(updated) = gir.nodes.get_mut(&node_id) {
                        updated.kind = NodeKind::Constant(constant.clone());
                        updated.operands.clear();
                    }
                    if let Some(result) = node.results.first() {
                        constants.insert(*result, constant);
                    }
                }
            }
            _ => {}
        }
    }
}

fn fold_classical(
    op: ClassicalOp,
    operands: &[ValueId],
    constants: &HashMap<ValueId, Constant>,
) -> Option<Constant> {
    let lhs = constants.get(operands.first()?)?;
    match op {
        ClassicalOp::Not => match lhs {
            Constant::Bool(value) => Some(Constant::Bool(!value)),
            _ => None,
        },
        ClassicalOp::Neg => match lhs {
            Constant::Int(value) => value.checked_neg().map(Constant::Int),
            _ => None,
        },
        ClassicalOp::And | ClassicalOp::Or => {
            let rhs = constants.get(operands.get(1)?)?;
            match (op, lhs, rhs) {
                (ClassicalOp::And, Constant::Bool(a), Constant::Bool(b)) => {
                    Some(Constant::Bool(*a && *b))
                }
                (ClassicalOp::Or, Constant::Bool(a), Constant::Bool(b)) => {
                    Some(Constant::Bool(*a || *b))
                }
                _ => None,
            }
        }
        ClassicalOp::Add
        | ClassicalOp::Sub
        | ClassicalOp::Mul
        | ClassicalOp::Div
        | ClassicalOp::Mod => {
            let rhs = constants.get(operands.get(1)?)?;
            let (Constant::Int(a), Constant::Int(b)) = (lhs, rhs) else {
                return None;
            };
            let value = match op {
                ClassicalOp::Add => a.checked_add(*b),
                ClassicalOp::Sub => a.checked_sub(*b),
                ClassicalOp::Mul => a.checked_mul(*b),
                ClassicalOp::Div if *b != 0 => a.checked_div(*b),
                ClassicalOp::Mod if *b != 0 => a.checked_rem(*b),
                _ => None,
            }?;
            Some(Constant::Int(value))
        }
        ClassicalOp::Eq
        | ClassicalOp::Ne
        | ClassicalOp::Lt
        | ClassicalOp::Le
        | ClassicalOp::Gt
        | ClassicalOp::Ge => {
            let rhs = constants.get(operands.get(1)?)?;
            let (Constant::Int(a), Constant::Int(b)) = (lhs, rhs) else {
                return None;
            };
            let value = match op {
                ClassicalOp::Eq => a == b,
                ClassicalOp::Ne => a != b,
                ClassicalOp::Lt => a < b,
                ClassicalOp::Le => a <= b,
                ClassicalOp::Gt => a > b,
                ClassicalOp::Ge => a >= b,
                _ => false,
            };
            Some(Constant::Bool(value))
        }
    }
}

fn dead_code_elimination(gir: &mut Gir, func_id: FuncId) {
    let Some(func) = gir.funcs.get(&func_id) else {
        return;
    };
    let mut used = HashSet::new();
    for block in func.blocks.values() {
        for node_id in &block.nodes {
            if let Some(node) = gir.nodes.get(node_id) {
                used.extend(node.operands.iter().copied());
                if let NodeKind::Phi { incoming } = &node.kind {
                    used.extend(incoming.iter().map(|(_, value)| *value));
                }
            }
        }
        if let Some(terminator) = &block.terminator {
            match terminator {
                Terminator::Return(value) => {
                    used.extend(value.iter().copied());
                }
                Terminator::CondBranch { cond, .. } => {
                    used.insert(*cond);
                }
                _ => {}
            }
        }
    }

    let removable: HashSet<_> = func
        .blocks
        .values()
        .flat_map(|block| block.nodes.iter())
        .filter_map(|node_id| gir.nodes.get(node_id).map(|node| (*node_id, node)))
        .filter(|(_, node)| {
            matches!(
                node.kind,
                NodeKind::Constant(_) | NodeKind::Classical(_) | NodeKind::Phi { .. }
            ) && node.results.iter().all(|result| !used.contains(result))
        })
        .map(|(node_id, _)| node_id)
        .collect();
    if removable.is_empty() {
        return;
    }
    if let Some(func) = gir.funcs.get_mut(&func_id) {
        for block in func.blocks.values_mut() {
            block.nodes.retain(|node_id| !removable.contains(node_id));
        }
    }
    for node_id in removable {
        gir.nodes.remove(&node_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimize_empty() {
        optimize_gir(&mut Gir::default());
    }

    #[test]
    fn test_optimize_with_func() {
        let mut gir = Gir::default();
        let func = gala_gir::GirFunc::new_test();
        gir.funcs.insert(func.id, func);
        optimize_gir(&mut gir);
        assert_eq!(gir.funcs.len(), 1);
    }

    #[test]
    fn folds_integer_constants_and_removes_dead_operands() {
        let mut gir = Gir::default();
        let mut func = gala_gir::GirFunc::new_test();
        let block = func.blocks.get_mut(&gala_gir::BlockId(0)).expect("entry block exists");
        block.nodes = vec![gala_gir::NodeId(0), gala_gir::NodeId(1), gala_gir::NodeId(2)];
        block.terminator = Some(Terminator::Return(Some(ValueId(2))));
        gir.funcs.insert(func.id, func);
        for (id, value, node_id) in [(0, 2, 0), (1, 3, 1)] {
            gir.nodes.insert(
                NodeId(node_id),
                GirNode {
                    id: NodeId(node_id),
                    kind: NodeKind::Constant(Constant::Int(value)),
                    operands: Vec::new(),
                    results: vec![ValueId(id)],
                    span: gala_span::Span::dummy(),
                },
            );
        }
        gir.nodes.insert(
            NodeId(2),
            GirNode {
                id: NodeId(2),
                kind: NodeKind::Classical(ClassicalOp::Add),
                operands: vec![ValueId(0), ValueId(1)],
                results: vec![ValueId(2)],
                span: gala_span::Span::dummy(),
            },
        );

        optimize_gir(&mut gir);
        assert!(matches!(gir.nodes[&NodeId(2)].kind, NodeKind::Constant(Constant::Int(5))));
        assert!(!gir.nodes.contains_key(&NodeId(0)));
        assert!(!gir.nodes.contains_key(&NodeId(1)));
    }
}
