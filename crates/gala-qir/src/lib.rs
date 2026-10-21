//! QIR emission (LLVM IR) for Gala.

use gala_diagnostics::{codes, Diagnostic, Diagnostics};
use gala_gir::{AllocKind, Constant, GateKind, Gir, NodeKind, Terminator, TypeKind, ValueId};
use std::collections::{BTreeSet, HashMap};

/// Map a GIR gate to the corresponding QIR base-profile intrinsic.
pub fn gate_intrinsic_name(gate: &GateKind) -> &'static str {
    match gate {
        GateKind::H => "__quantum__qis__h__body",
        GateKind::X => "__quantum__qis__x__body",
        GateKind::Y => "__quantum__qis__y__body",
        GateKind::Z => "__quantum__qis__z__body",
        GateKind::S => "__quantum__qis__s__body",
        GateKind::T => "__quantum__qis__t__body",
        GateKind::Rx => "__quantum__qis__rx__body",
        GateKind::Ry => "__quantum__qis__ry__body",
        GateKind::Rz => "__quantum__qis__rz__body",
        GateKind::CX => "__quantum__qis__cx__body",
        GateKind::CZ => "__quantum__qis__cz__body",
        GateKind::SWAP => "__quantum__qis__swap__body",
        GateKind::Measure => "__quantum__qis__mz__body",
    }
}

/// QIR profile to emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QirProfile {
    Base,
    Adaptive,
}

/// Emit QIR from GIR.
pub fn emit_qir(gir: &Gir, profile: QirProfile) -> Result<String, Diagnostics> {
    let mut output = String::new();

    output.push_str("; QIR Module\n");
    output.push_str("target triple = \"x86_64-unknown-linux-gnu\"\n\n");
    output.push_str("%Qubit = type opaque\n");
    output.push_str("%Result = type opaque\n\n");

    let mut declarations = BTreeSet::new();
    let mut funcs: Vec<_> = gir.funcs.values().collect();
    funcs.sort_by_key(|func| func.id.0);
    for func in &funcs {
        for block in func.blocks.values() {
            for node_id in &block.nodes {
                let Some(node) = gir.nodes.get(node_id) else {
                    return Err(unsupported("QIR emission encountered a missing GIR node"));
                };
                match &node.kind {
                    NodeKind::Gate(gate) => {
                        declarations.insert(gate_intrinsic_name(gate).to_string());
                    }
                    NodeKind::Measure { .. } => {
                        declarations.insert("__quantum__qis__mz__body".to_string());
                        declarations.insert("__quantum__rt__result_get_zero".to_string());
                    }
                    NodeKind::Alloc(AllocKind::Qubit) | NodeKind::Alloc(AllocKind::Qubits(_)) => {
                        declarations.insert("__quantum__rt__qubit_allocate".to_string());
                    }
                    NodeKind::Free { .. } => {
                        declarations.insert("__quantum__rt__qubit_release".to_string());
                    }
                    NodeKind::Alloc(AllocKind::Classical(_)) => {
                        return Err(unsupported(
                            "QIR emission does not support classical allocation",
                        ));
                    }
                    NodeKind::Constant(Constant::Float(_)) => {}
                    NodeKind::Constant(Constant::Unit) => {}
                    NodeKind::Constant(_) => {
                        return Err(unsupported(
                            "QIR emission only supports unit and rotation-angle constants",
                        ));
                    }
                    NodeKind::Classical(_) | NodeKind::Phi { .. } | NodeKind::Call { .. } => {
                        return Err(unsupported(
                            "QIR emission does not yet support classical nodes, Phi nodes, or calls",
                        ));
                    }
                    NodeKind::Tuple { .. } | NodeKind::Extract { .. } => {}
                }
            }
        }
    }
    for declaration in declarations {
        output.push_str(&intrinsic_declaration(&declaration));
    }
    if output.ends_with("\n") {
        output.push('\n');
    }

    for func in funcs {
        if func.blocks.len() != 1 {
            return Err(unsupported("QIR emission currently requires a single basic block"));
        }
        let Some(block) = func.blocks.values().next() else {
            return Err(unsupported("QIR function has no basic block"));
        };
        if !matches!(&block.terminator, Some(Terminator::Return(_)) | None) {
            return Err(unsupported("QIR emission currently supports return terminators only"));
        }
        let mut parameter_declarations = Vec::with_capacity(func.params.len());
        for (index, _) in func.params.iter().enumerate() {
            let ty = if let Some(type_id) = func.param_tys.get(index) {
                let Some(gir_type) = gir.types.get(type_id) else {
                    return Err(unsupported("QIR parameter references a missing GIR type"));
                };
                qir_type(&gir_type.kind)?
            } else {
                // Preserve compatibility with hand-built GIR fixtures created
                // before parameter metadata was introduced.
                "%Qubit*".to_string()
            };
            parameter_declarations.push(format!("{ty} %q{index}"));
        }
        let params = parameter_declarations.join(", ");
        let return_ty = func
            .ret_ty
            .and_then(|type_id| gir.types.get(&type_id))
            .map(|ty| qir_type(&ty.kind))
            .transpose()?
            .unwrap_or_else(|| "void".to_string());
        output.push_str(&format!("; Function: {}\n", func.name));
        output.push_str(&format!("define {return_ty} @{}({params}) {{\n", func.name));
        let mut values: HashMap<ValueId, String> = func
            .params
            .iter()
            .enumerate()
            .map(|(index, value)| (*value, format!("%q{index}")))
            .collect();
        let mut next_temp = 0usize;
        let mut tuples = HashMap::<ValueId, Vec<String>>::new();
        for node_id in &block.nodes {
            let Some(node) = gir.nodes.get(node_id) else {
                return Err(unsupported("QIR emission encountered a missing GIR node"));
            };
            emit_node(&mut output, node, &mut values, &mut tuples, &mut next_temp)?;
        }
        match block.terminator.as_ref() {
            Some(Terminator::Return(Some(value))) => {
                let Some(mapped) = values.get(value) else {
                    return Err(unsupported("QIR return references an unmapped value"));
                };
                if return_ty == "void" {
                    return Err(unsupported(
                        "QIR function returns a value without return type metadata",
                    ));
                }
                output.push_str(&format!("  ret {return_ty} {mapped}\n"));
            }
            Some(Terminator::Return(None)) | None => {
                if return_ty != "void" {
                    return Err(unsupported("QIR typed function has no return value"));
                }
                output.push_str("  ret void\n");
            }
            _ => {
                return Err(unsupported("QIR emission currently supports return terminators only"))
            }
        }
        output.push_str("}\n\n");
    }

    if profile == QirProfile::Adaptive {
        output.insert_str(0, "; profile: adaptive\n");
    } else {
        output.insert_str(0, "; profile: base\n");
    }
    Ok(output)
}

fn intrinsic_declaration(name: &str) -> String {
    let signature = match name {
        "__quantum__rt__qubit_allocate" => "%Qubit* @__quantum__rt__qubit_allocate()",
        "__quantum__rt__result_get_zero" => "%Result* @__quantum__rt__result_get_zero()",
        "__quantum__qis__rx__body" | "__quantum__qis__ry__body" | "__quantum__qis__rz__body" => {
            return format!("declare void @{name}(double, %Qubit*)\n")
        }
        "__quantum__qis__cx__body" | "__quantum__qis__cz__body" | "__quantum__qis__swap__body" => {
            return format!("declare void @{name}(%Qubit*, %Qubit*)\n")
        }
        "__quantum__qis__mz__body" => return format!("declare void @{name}(%Qubit*, %Result*)\n"),
        _ => return format!("declare void @{name}(%Qubit*)\n"),
    };
    format!("declare {signature}\n")
}

fn qir_type(kind: &TypeKind) -> Result<String, Diagnostics> {
    match kind {
        TypeKind::Int => Ok("i64".to_string()),
        TypeKind::Float => Ok("double".to_string()),
        TypeKind::Bool => Ok("i1".to_string()),
        TypeKind::Qubit => Ok("%Qubit*".to_string()),
        TypeKind::Measured(_) => Ok("%Result*".to_string()),
        TypeKind::Unit => Err(unsupported("QIR function parameters cannot have unit type")),
        TypeKind::Qubits(_)
        | TypeKind::Tuple(_)
        | TypeKind::Array(_, _)
        | TypeKind::Fn { .. }
        | TypeKind::Named(_, _)
        | TypeKind::Complex
        | TypeKind::String => Err(unsupported("QIR cannot represent this GIR parameter type")),
    }
}

fn emit_node(
    output: &mut String,
    node: &gala_gir::GirNode,
    values: &mut HashMap<ValueId, String>,
    tuples: &mut HashMap<ValueId, Vec<String>>,
    next_temp: &mut usize,
) -> Result<(), Diagnostics> {
    match &node.kind {
        NodeKind::Alloc(AllocKind::Qubit) => {
            let Some(result) = node.results.first() else {
                return Err(unsupported("QIR allocation is missing its result"));
            };
            let register = format!("%qir{next_temp}");
            *next_temp += 1;
            output.push_str(&format!(
                "  {register} = call %Qubit* @__quantum__rt__qubit_allocate()\n"
            ));
            values.insert(*result, register);
        }
        NodeKind::Gate(gate) => {
            let Some(target) = node.operands.first().and_then(|value| values.get(value)) else {
                return Err(unsupported("QIR gate is missing a mapped target qubit"));
            };
            let intrinsic = gate_intrinsic_name(gate);
            match gate {
                GateKind::Rx | GateKind::Ry | GateKind::Rz => {
                    let Some(angle) = node.operands.get(1).and_then(|value| values.get(value))
                    else {
                        return Err(unsupported("QIR rotation is missing its mapped angle"));
                    };
                    output.push_str(&format!(
                        "  call void @{intrinsic}(double {angle}, %Qubit* {target})\n"
                    ));
                }
                GateKind::CX | GateKind::CZ | GateKind::SWAP => {
                    let Some(second) = node.operands.get(1).and_then(|value| values.get(value))
                    else {
                        return Err(unsupported("QIR two-qubit gate is missing its second target"));
                    };
                    output.push_str(&format!(
                        "  call void @{intrinsic}(%Qubit* {target}, %Qubit* {second})\n"
                    ));
                }
                GateKind::Measure => {
                    return Err(unsupported("QIR measurement must use a Measure node"));
                }
                _ => output.push_str(&format!("  call void @{intrinsic}(%Qubit* {target})\n")),
            }
            if let Some(result) = node.results.first() {
                values.insert(*result, target.clone());
            }
            if matches!(gate, GateKind::CX | GateKind::CZ | GateKind::SWAP) {
                if let (Some(result), Some(second)) =
                    (node.results.get(1), node.operands.get(1).and_then(|value| values.get(value)))
                {
                    values.insert(*result, second.clone());
                }
            }
        }
        NodeKind::Free { target } => {
            let Some(mapped_target) = values.get(target).cloned() else {
                return Err(unsupported("QIR free is missing its mapped target"));
            };
            output.push_str(&format!(
                "  call void @__quantum__rt__qubit_release(%Qubit* {mapped_target})\n"
            ));
            values.remove(target);
        }
        NodeKind::Measure { target } => {
            let Some(target) = values.get(target) else {
                return Err(unsupported("QIR measurement is missing its mapped target"));
            };
            let result = format!("%result{next_temp}");
            *next_temp += 1;
            output.push_str(&format!(
                "  {result} = call %Result* @__quantum__rt__result_get_zero()\n"
            ));
            output.push_str(&format!(
                "  call void @__quantum__qis__mz__body(%Qubit* {target}, %Result* {result})\n"
            ));
            if let Some(value) = node.results.first() {
                values.insert(*value, result);
            }
        }
        NodeKind::Constant(Constant::Float(value)) => {
            let Some(result) = node.results.first() else {
                return Err(unsupported("QIR constant is missing its result"));
            };
            values.insert(*result, format_double(*value));
        }
        NodeKind::Constant(Constant::Unit) => {}
        NodeKind::Tuple { elements } => {
            let mapped = elements
                .iter()
                .map(|element| {
                    values
                        .get(element)
                        .cloned()
                        .ok_or_else(|| unsupported("QIR tuple references an unmapped value"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            if let Some(result) = node.results.first() {
                tuples.insert(*result, mapped);
            }
        }
        NodeKind::Extract { tuple, index } => {
            let Some(element) = tuples.get(tuple).and_then(|elements| elements.get(*index)) else {
                return Err(unsupported("QIR tuple extraction is out of bounds"));
            };
            if let Some(result) = node.results.first() {
                values.insert(*result, element.clone());
            }
        }
        _ => return Err(unsupported("unsupported GIR node in QIR emitter")),
    }
    Ok(())
}

fn format_double(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.17e}")
    } else {
        "0.0".to_string()
    }
}

fn unsupported(message: &str) -> Diagnostics {
    let mut diagnostics = Diagnostics::new();
    diagnostics.push(Diagnostic::error(codes::BACKEND_CAPABILITY_MISMATCH, message));
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use gala_gir::GirFunc;

    #[test]
    fn test_qir_emission_basic() {
        let gir = Gir::default();
        let result = emit_qir(&gir, QirProfile::Base);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("QIR Module"));
    }

    #[test]
    fn test_qir_profile_debug() {
        assert_eq!(format!("{:?}", QirProfile::Base), "Base");
        assert_eq!(format!("{:?}", QirProfile::Adaptive), "Adaptive");
    }

    #[test]
    fn test_qir_emission_both_profiles() {
        let gir = Gir::default();
        for profile in &[QirProfile::Base, QirProfile::Adaptive] {
            let result = emit_qir(&gir, *profile);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_qir_emission_with_func() {
        let mut gir = Gir::default();
        let func = GirFunc::new_test();
        gir.funcs.insert(func.id, func);
        let result = emit_qir(&gir, QirProfile::Base);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("test"));
    }

    #[test]
    fn test_qir_target_triple() {
        let result = emit_qir(&Gir::default(), QirProfile::Base);
        let output = result.unwrap();
        assert!(output.contains("x86_64"));
    }
}
