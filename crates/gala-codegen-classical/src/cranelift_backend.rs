//! Native object generation for the classical subset of GIR using Cranelift.

use cranelift_codegen::ir::{
    self,
    condcodes::{FloatCC, IntCC},
    types, AbiParam, InstBuilder,
};
use cranelift_codegen::{settings, Context};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{default_libcall_names, FuncId as ClifFuncId, Linkage, Module};
use cranelift_native::builder as native_builder;
use cranelift_object::{ObjectBuilder, ObjectModule};
use gala_diagnostics::{codes, Diagnostic, Diagnostics};
use gala_gir::{
    ClassicalOp, Constant, Gir, GirFunc, GirNode, NodeKind, Terminator, TypeKind, ValueId,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClifType {
    Int,
    Bool,
    Float,
}

impl ClifType {
    fn ir_type(self) -> ir::Type {
        match self {
            ClifType::Int => types::I64,
            ClifType::Bool => types::I8,
            ClifType::Float => types::F64,
        }
    }
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    params: Vec<ClifType>,
    ret: Option<ClifType>,
    values: HashMap<ValueId, ClifType>,
}

pub fn emit_cranelift(gir: &Gir) -> Result<Vec<u8>, Diagnostics> {
    let infos = collect_function_info(gir)?;
    let flag_builder = settings::builder();
    let flags = settings::Flags::new(flag_builder);
    let isa = native_builder()
        .map_err(|error| unsupported(format!("Cranelift could not select the host ISA: {error}")))?
        .finish(flags)
        .map_err(|error| unsupported(format!("Cranelift could not build the host ISA: {error}")))?;
    let object_builder =
        ObjectBuilder::new(isa, "gala", default_libcall_names()).map_err(|error| {
            unsupported(format!("Cranelift could not create an object module: {error}"))
        })?;
    let mut module = ObjectModule::new(object_builder);

    let mut clif_ids = HashMap::<gala_gir::FuncId, ClifFuncId>::new();
    let mut funcs: Vec<&GirFunc> = gir.funcs.values().collect();
    funcs.sort_by_key(|func| func.id.0);
    for func in &funcs {
        let Some(info) = infos.get(&func.id) else {
            return Err(unsupported(format!("missing Cranelift signature for {}", func.name)));
        };
        let mut signature = module.make_signature();
        signature.params.extend(info.params.iter().map(|ty| AbiParam::new(ty.ir_type())));
        if let Some(ret) = info.ret {
            signature.returns.push(AbiParam::new(ret.ir_type()));
        }
        let id = module
            .declare_function(&func.name.0, Linkage::Export, &signature)
            .map_err(|error| unsupported(format!("cannot declare {}: {error}", func.name)))?;
        clif_ids.insert(func.id, id);
    }

    for func in funcs {
        let Some(info) = infos.get(&func.id) else {
            return Err(unsupported(format!("missing Cranelift signature for {}", func.name)));
        };
        let Some(&clif_id) = clif_ids.get(&func.id) else {
            return Err(unsupported(format!("missing Cranelift function ID for {}", func.name)));
        };
        define_function(&mut module, gir, func, info, &clif_ids, clif_id)?;
    }

    module
        .finish()
        .emit()
        .map_err(|error| unsupported(format!("Cranelift could not serialize the object: {error}")))
}

fn collect_function_info(
    gir: &Gir,
) -> Result<HashMap<gala_gir::FuncId, FunctionInfo>, Diagnostics> {
    let mut infos = HashMap::new();
    let mut funcs: Vec<&GirFunc> = gir.funcs.values().collect();
    funcs.sort_by_key(|func| func.id.0);
    for func in funcs {
        if func.blocks.len() != 1 {
            return Err(unsupported(format!(
                "Cranelift classical lowering currently requires one basic block in {}",
                func.name
            )));
        }
        let mut values = HashMap::new();
        let params = func
            .params
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let ty = func
                    .param_tys
                    .get(index)
                    .and_then(|type_id| gir.types.get(type_id))
                    .map(|ty| clif_type(&ty.kind))
                    .transpose()?
                    .unwrap_or(ClifType::Int);
                values.insert(*value, ty);
                Ok(ty)
            })
            .collect::<Result<Vec<_>, Diagnostics>>()?;

        let block = func
            .blocks
            .get(&func.entry_block)
            .ok_or_else(|| unsupported(format!("function {} has no entry block", func.name)))?;
        for node_id in &block.nodes {
            let node = gir
                .nodes
                .get(node_id)
                .ok_or_else(|| unsupported("Cranelift encountered a missing GIR node"))?;
            let result_ty = match &node.kind {
                NodeKind::Constant(Constant::Int(_)) => Some(ClifType::Int),
                NodeKind::Constant(Constant::Float(_)) => Some(ClifType::Float),
                NodeKind::Constant(Constant::Bool(_)) => Some(ClifType::Bool),
                NodeKind::Classical(op) => Some(classical_result_type(*op, &values, node)),
                NodeKind::Call { func: callee, .. } => infos
                    .get(callee)
                    .and_then(|callee_info: &FunctionInfo| callee_info.ret)
                    .or_else(|| {
                        gir.funcs.get(callee).and_then(|callee_func| {
                            callee_func
                                .ret_ty
                                .and_then(|type_id| gir.types.get(&type_id))
                                .and_then(|ty| clif_type(&ty.kind).ok())
                        })
                    }),
                NodeKind::Constant(Constant::Unit) => None,
                _ => {
                    return Err(unsupported(format!(
                        "Cranelift classical lowering does not support {:?}",
                        node.kind
                    )))
                }
            };
            if let Some(ty) = result_ty {
                for result in &node.results {
                    values.insert(*result, ty);
                }
            }
        }
        let ret = func
            .ret_ty
            .and_then(|type_id| gir.types.get(&type_id))
            .map(|ty| clif_type(&ty.kind))
            .transpose()?
            .or_else(|| match block.terminator {
                Some(Terminator::Return(Some(value))) => values.get(&value).copied(),
                _ => None,
            });
        infos.insert(func.id, FunctionInfo { params, ret, values });
    }
    Ok(infos)
}

fn clif_type(kind: &TypeKind) -> Result<ClifType, Diagnostics> {
    match kind {
        TypeKind::Int => Ok(ClifType::Int),
        TypeKind::Float => Ok(ClifType::Float),
        TypeKind::Bool => Ok(ClifType::Bool),
        TypeKind::Unit => Err(unsupported("unit is not a machine value")),
        TypeKind::Qubit
        | TypeKind::Qubits(_)
        | TypeKind::Measured(_)
        | TypeKind::Tuple(_)
        | TypeKind::Array(_, _)
        | TypeKind::Fn { .. }
        | TypeKind::Named(_, _) => {
            Err(unsupported("Cranelift classical lowering received a non-classical type"))
        }
        TypeKind::Complex | TypeKind::String => {
            Err(unsupported("Cranelift classical lowering does not support this value type"))
        }
    }
}

fn classical_result_type(
    op: ClassicalOp,
    values: &HashMap<ValueId, ClifType>,
    node: &GirNode,
) -> ClifType {
    match op {
        ClassicalOp::Eq
        | ClassicalOp::Ne
        | ClassicalOp::Lt
        | ClassicalOp::Le
        | ClassicalOp::Gt
        | ClassicalOp::Ge => ClifType::Bool,
        ClassicalOp::And | ClassicalOp::Or | ClassicalOp::Not => ClifType::Bool,
        _ => node
            .operands
            .first()
            .and_then(|value| values.get(value))
            .copied()
            .unwrap_or(ClifType::Int),
    }
}

fn define_function(
    module: &mut ObjectModule,
    gir: &Gir,
    func: &GirFunc,
    info: &FunctionInfo,
    clif_ids: &HashMap<gala_gir::FuncId, ClifFuncId>,
    clif_id: ClifFuncId,
) -> Result<(), Diagnostics> {
    let mut signature = module.make_signature();
    signature.params.extend(info.params.iter().map(|ty| AbiParam::new(ty.ir_type())));
    if let Some(ret) = info.ret {
        signature.returns.push(AbiParam::new(ret.ir_type()));
    }
    let mut context = Context::new();
    context.func =
        ir::Function::with_name_signature(ir::UserFuncName::user(0, clif_id.as_u32()), signature);
    let mut function_refs = HashMap::new();
    for (func_id, target_id) in clif_ids {
        function_refs.insert(*func_id, module.declare_func_in_func(*target_id, &mut context.func));
    }

    let block = func
        .blocks
        .get(&func.entry_block)
        .ok_or_else(|| unsupported(format!("function {} has no entry block", func.name)))?;
    let mut builder_context = FunctionBuilderContext::new();
    {
        let mut builder = FunctionBuilder::new(&mut context.func, &mut builder_context);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        let mut values = HashMap::new();
        for (index, value) in func.params.iter().enumerate() {
            if let Some(clif_value) = builder.block_params(entry).get(index) {
                values.insert(*value, *clif_value);
            }
        }
        for node_id in &block.nodes {
            let node = gir
                .nodes
                .get(node_id)
                .ok_or_else(|| unsupported("Cranelift encountered a missing GIR node"))?;
            emit_node(&mut builder, node, &mut values, &function_refs, &info.values)?;
        }
        match block.terminator.as_ref() {
            Some(Terminator::Return(Some(value))) => {
                let value = values
                    .get(value)
                    .copied()
                    .ok_or_else(|| unsupported("Cranelift return references an unknown value"))?;
                builder.ins().return_(&[value]);
            }
            Some(Terminator::Return(None)) | None => {
                builder.ins().return_(&[]);
            }
            _ => {
                return Err(unsupported(
                    "Cranelift classical lowering supports return terminators only",
                ))
            }
        }
        builder.seal_all_blocks();
        builder.finalize(module.target_config());
    }
    module
        .define_function(clif_id, &mut context)
        .map_err(|error| unsupported(format!("Cranelift failed to define {}: {error}", func.name)))
}

fn emit_node(
    builder: &mut FunctionBuilder<'_>,
    node: &GirNode,
    values: &mut HashMap<ValueId, ir::Value>,
    function_refs: &HashMap<gala_gir::FuncId, ir::FuncRef>,
    value_types: &HashMap<ValueId, ClifType>,
) -> Result<(), Diagnostics> {
    let operands = || {
        node.operands
            .iter()
            .map(|value| {
                values
                    .get(value)
                    .copied()
                    .ok_or_else(|| unsupported("Cranelift node references an unknown operand"))
            })
            .collect::<Result<Vec<_>, Diagnostics>>()
    };
    let result = node.results.first().copied();
    let emitted = match &node.kind {
        NodeKind::Constant(Constant::Int(value)) => builder.ins().iconst(types::I64, *value),
        NodeKind::Constant(Constant::Bool(value)) => {
            builder.ins().iconst(types::I8, i64::from(*value))
        }
        NodeKind::Constant(Constant::Float(value)) => builder.ins().f64const(*value),
        NodeKind::Classical(op) => {
            let args = operands()?;
            let lhs = args
                .first()
                .copied()
                .ok_or_else(|| unsupported("classical operation is missing its left operand"))?;
            let rhs = args.get(1).copied();
            let lhs_type = node
                .operands
                .first()
                .and_then(|value| value_types.get(value))
                .copied()
                .unwrap_or(ClifType::Int);
            match op {
                ClassicalOp::Add => binary(
                    builder,
                    lhs,
                    rhs,
                    |b, a, c| b.ins().iadd(a, c),
                    |b, a, c| b.ins().fadd(a, c),
                    lhs_type,
                )?,
                ClassicalOp::Sub => binary(
                    builder,
                    lhs,
                    rhs,
                    |b, a, c| b.ins().isub(a, c),
                    |b, a, c| b.ins().fsub(a, c),
                    lhs_type,
                )?,
                ClassicalOp::Mul => binary(
                    builder,
                    lhs,
                    rhs,
                    |b, a, c| b.ins().imul(a, c),
                    |b, a, c| b.ins().fmul(a, c),
                    lhs_type,
                )?,
                ClassicalOp::Div => binary(
                    builder,
                    lhs,
                    rhs,
                    |b, a, c| b.ins().sdiv(a, c),
                    |b, a, c| b.ins().fdiv(a, c),
                    lhs_type,
                )?,
                ClassicalOp::Mod => {
                    if lhs_type == ClifType::Float {
                        return Err(unsupported(
                            "Cranelift does not support floating-point modulo",
                        ));
                    }
                    let rhs =
                        rhs.ok_or_else(|| unsupported("modulo is missing its right operand"))?;
                    builder.ins().srem(lhs, rhs)
                }
                ClassicalOp::And => builder
                    .ins()
                    .band(lhs, rhs.ok_or_else(|| unsupported("and is missing its right operand"))?),
                ClassicalOp::Or => builder
                    .ins()
                    .bor(lhs, rhs.ok_or_else(|| unsupported("or is missing its right operand"))?),
                ClassicalOp::Not => {
                    let one = builder.ins().iconst(types::I8, 1);
                    builder.ins().bxor(lhs, one)
                }
                ClassicalOp::Neg => {
                    if lhs_type == ClifType::Float {
                        builder.ins().fneg(lhs)
                    } else {
                        builder.ins().ineg(lhs)
                    }
                }
                ClassicalOp::Eq
                | ClassicalOp::Ne
                | ClassicalOp::Lt
                | ClassicalOp::Le
                | ClassicalOp::Gt
                | ClassicalOp::Ge => {
                    let rhs =
                        rhs.ok_or_else(|| unsupported("comparison is missing its right operand"))?;
                    let condition = match op {
                        ClassicalOp::Eq => (IntCC::Equal, FloatCC::Equal),
                        ClassicalOp::Ne => (IntCC::NotEqual, FloatCC::NotEqual),
                        ClassicalOp::Lt => (IntCC::SignedLessThan, FloatCC::LessThan),
                        ClassicalOp::Le => (IntCC::SignedLessThanOrEqual, FloatCC::LessThanOrEqual),
                        ClassicalOp::Gt => (IntCC::SignedGreaterThan, FloatCC::GreaterThan),
                        ClassicalOp::Ge => {
                            (IntCC::SignedGreaterThanOrEqual, FloatCC::GreaterThanOrEqual)
                        }
                        _ => return Err(unsupported("invalid classical comparison operator")),
                    };
                    if lhs_type == ClifType::Float {
                        builder.ins().fcmp(condition.1, lhs, rhs)
                    } else {
                        builder.ins().icmp(condition.0, lhs, rhs)
                    }
                }
            }
        }
        NodeKind::Call { func, .. } => {
            let args = operands()?;
            let func_ref = function_refs
                .get(func)
                .copied()
                .ok_or_else(|| unsupported("Cranelift call references an unknown function"))?;
            let call = builder.ins().call(func_ref, &args);
            let results = builder.inst_results(call);
            let Some(value) = results.first() else {
                return Ok(());
            };
            *value
        }
        NodeKind::Constant(Constant::Unit) => return Ok(()),
        _ => {
            return Err(unsupported(format!(
                "Cranelift classical lowering does not support {:?}",
                node.kind
            )))
        }
    };
    if let Some(result) = result {
        let value = if matches!(
            node.kind,
            NodeKind::Classical(
                ClassicalOp::Eq
                    | ClassicalOp::Ne
                    | ClassicalOp::Lt
                    | ClassicalOp::Le
                    | ClassicalOp::Gt
                    | ClassicalOp::Ge
            )
        ) {
            let one = builder.ins().iconst(types::I8, 1);
            let zero = builder.ins().iconst(types::I8, 0);
            builder.ins().select(emitted, one, zero)
        } else {
            emitted
        };
        values.insert(result, value);
    }
    Ok(())
}

fn binary(
    builder: &mut FunctionBuilder<'_>,
    lhs: ir::Value,
    rhs: Option<ir::Value>,
    int: impl FnOnce(&mut FunctionBuilder<'_>, ir::Value, ir::Value) -> ir::Value,
    float: impl FnOnce(&mut FunctionBuilder<'_>, ir::Value, ir::Value) -> ir::Value,
    lhs_type: ClifType,
) -> Result<ir::Value, Diagnostics> {
    let rhs = rhs.ok_or_else(|| unsupported("binary operation is missing its right operand"))?;
    Ok(if lhs_type == ClifType::Float { float(builder, lhs, rhs) } else { int(builder, lhs, rhs) })
}

fn unsupported(message: impl Into<String>) -> Diagnostics {
    let mut diagnostics = Diagnostics::new();
    diagnostics.push(Diagnostic::error(codes::BACKEND_CAPABILITY_MISMATCH, message));
    diagnostics
}
