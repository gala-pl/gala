//! Gala Intermediate Representation (GIR).

use gala_ast::{self, Ident, Literal};
use gala_diagnostics::Diagnostics;
use gala_hir::*;
use gala_span::Span;
use gala_types::{Effect, Ty};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════════
// GIR Data Model
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Default)]
pub struct Gir {
    pub funcs: HashMap<FuncId, GirFunc>,
    /// All nodes referenced by blocks.  Blocks store stable node IDs so that
    /// passes can rewrite the graph without copying node payloads around.
    pub nodes: HashMap<NodeId, GirNode>,
    pub types: HashMap<TypeId, GirType>,
    pub consts: HashMap<ConstId, GirConst>,
}

#[derive(Debug, Clone)]
pub struct GirFunc {
    pub id: FuncId,
    pub name: Ident,
    pub params: Vec<ValueId>,
    /// The type of each entry in `params`, in the same order.
    pub param_tys: Vec<TypeId>,
    pub ret_ty: Option<TypeId>,
    pub effect: Effect,
    pub blocks: HashMap<BlockId, Block>,
    pub entry_block: BlockId,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub id: BlockId,
    pub nodes: Vec<NodeId>,
    pub terminator: Option<Terminator>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct GirNode {
    pub id: NodeId,
    pub kind: NodeKind,
    pub operands: Vec<ValueId>,
    pub results: Vec<ValueId>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Gate(GateKind),
    Classical(ClassicalOp),
    Alloc(AllocKind),
    Free { target: ValueId },
    Measure { target: ValueId },
    Tuple { elements: Vec<ValueId> },
    Extract { tuple: ValueId, index: usize },
    Constant(Constant),
    Phi { incoming: Vec<(BlockId, ValueId)> },
    Call { func: FuncId, args: Vec<ValueId> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GateKind {
    H,
    X,
    Y,
    Z,
    S,
    T,
    Rx,
    Ry,
    Rz,
    CX,
    CZ,
    SWAP,
    Measure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClassicalOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    Neg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AllocKind {
    Qubit,
    Qubits(u64),
    Classical(TypeId),
}

#[derive(Debug, Clone)]
pub enum Constant {
    Int(i64),
    Float(f64),
    Bool(bool),
    Complex { re: f64, im: f64 },
    String(String),
    Unit,
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Return(Option<ValueId>),
    Branch(BlockId),
    CondBranch { cond: ValueId, then_bb: BlockId, else_bb: BlockId },
    Unreachable,
}

#[derive(Debug, Clone)]
pub struct GirType {
    pub id: TypeId,
    pub kind: TypeKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Int,
    Float,
    Bool,
    Complex,
    String,
    Unit,
    Qubit,
    Qubits(u64),
    Measured(TypeId),
    Tuple(Vec<TypeId>),
    Array(TypeId, u64),
    Fn { params: Vec<TypeId>, ret: TypeId, effect: Effect },
    Named(String, Vec<TypeId>),
}

#[derive(Debug, Clone)]
pub struct GirConst {
    pub id: ConstId,
    pub name: Ident,
    pub ty: TypeId,
    pub value: Constant,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FuncId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct NodeId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ValueId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TypeId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ConstId(pub u32);

// ═══════════════════════════════════════════════════════════════════════════════
// GIR Builder
// ═══════════════════════════════════════════════════════════════════════════════

struct GirBuilder<'a> {
    hir_funcs: &'a HashMap<DefId, HirFnDef>,
    type_of: &'a HashMap<DefId, Ty>,
    func_ids: HashMap<DefId, FuncId>,
    next_func_id: u32,
    next_block_id: u32,
    next_node_id: u32,
    next_value_id: u32,
    next_type_id: u32,
    results: HashMap<FuncId, GirFunc>,
    nodes: HashMap<NodeId, GirNode>,
    types: HashMap<TypeId, GirType>,
    type_ids: HashMap<Ty, TypeId>,
    diags: Diagnostics,
}

impl<'a> GirBuilder<'a> {
    fn new(hir_funcs: &'a HashMap<DefId, HirFnDef>, type_of: &'a HashMap<DefId, Ty>) -> Self {
        GirBuilder {
            hir_funcs,
            type_of,
            func_ids: HashMap::new(),
            next_func_id: 0,
            next_block_id: 1,
            next_node_id: 0,
            next_value_id: 0,
            next_type_id: 0,
            results: HashMap::new(),
            nodes: HashMap::new(),
            types: HashMap::new(),
            type_ids: HashMap::new(),
            diags: Diagnostics::new(),
        }
    }

    fn fresh_func_id(&mut self) -> FuncId {
        let id = FuncId(self.next_func_id);
        self.next_func_id += 1;
        id
    }

    fn fresh_block_id(&mut self) -> BlockId {
        let id = BlockId(self.next_block_id);
        self.next_block_id += 1;
        id
    }

    fn fresh_node_id(&mut self) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        id
    }

    fn fresh_value_id(&mut self) -> ValueId {
        let id = ValueId(self.next_value_id);
        self.next_value_id += 1;
        id
    }

    fn fresh_type_id(&mut self) -> TypeId {
        let id = TypeId(self.next_type_id);
        self.next_type_id += 1;
        id
    }

    fn type_id_for_ty(&mut self, ty: &Ty, span: Span) -> TypeId {
        if let Some(id) = self.type_ids.get(ty) {
            return *id;
        }

        let id = self.fresh_type_id();
        let kind = match ty {
            Ty::Int => TypeKind::Int,
            Ty::Float => TypeKind::Float,
            Ty::Bool => TypeKind::Bool,
            Ty::Complex => TypeKind::Complex,
            Ty::String => TypeKind::String,
            Ty::Unit => TypeKind::Unit,
            Ty::Qubit => TypeKind::Qubit,
            Ty::Qubits(size) => TypeKind::Qubits(*size),
            Ty::Measured(inner) => TypeKind::Measured(self.type_id_for_ty(inner, span)),
            Ty::Tuple(items) => {
                TypeKind::Tuple(items.iter().map(|item| self.type_id_for_ty(item, span)).collect())
            }
            Ty::Array(inner, size) => TypeKind::Array(self.type_id_for_ty(inner, span), *size),
            Ty::Fn { params, ret, effect } => TypeKind::Fn {
                params: params.iter().map(|param| self.type_id_for_ty(param, span)).collect(),
                ret: self.type_id_for_ty(ret, span),
                effect: *effect,
            },
            Ty::Named(def_id, args) => TypeKind::Named(
                format!("def:{}", def_id.index),
                args.iter().map(|arg| self.type_id_for_ty(arg, span)).collect(),
            ),
            Ty::Params => TypeKind::Named("Params".to_string(), Vec::new()),
            Ty::Var(_) | Ty::TyError => TypeKind::Named("unknown".to_string(), Vec::new()),
        };
        self.type_ids.insert(ty.clone(), id);
        self.types.insert(id, GirType { id, kind, span });
        id
    }

    fn emit_node(
        &mut self,
        block: &mut Block,
        kind: NodeKind,
        operands: Vec<ValueId>,
        results: Vec<ValueId>,
        span: Span,
    ) -> NodeId {
        let id = self.fresh_node_id();
        block.nodes.push(id);
        self.nodes.insert(id, GirNode { id, kind, operands, results, span });
        id
    }

    fn gate_kind_from_name(name: &str) -> Option<GateKind> {
        match name {
            "h" => Some(GateKind::H),
            "x" => Some(GateKind::X),
            "y" => Some(GateKind::Y),
            "z" => Some(GateKind::Z),
            "s" => Some(GateKind::S),
            "t" => Some(GateKind::T),
            "rx" => Some(GateKind::Rx),
            "ry" => Some(GateKind::Ry),
            "rz" => Some(GateKind::Rz),
            "cx" => Some(GateKind::CX),
            "cz" => Some(GateKind::CZ),
            "swap" => Some(GateKind::SWAP),
            _ => None,
        }
    }

    fn lower_all(&mut self) {
        // Assign function IDs before lowering bodies so calls can refer to any
        // function, including functions declared later in the source file.
        let mut defs: Vec<DefId> = self.hir_funcs.keys().cloned().collect();
        defs.sort_by_key(|def| (def.crate_id.0 .0, def.index));
        for def_id in defs {
            let func_id = self.fresh_func_id();
            self.func_ids.insert(def_id, func_id);
        }

        let mut hir_funcs_copy: Vec<HirFnDef> = self.hir_funcs.values().cloned().collect();
        hir_funcs_copy.sort_by_key(|f| (f.def_id.crate_id.0 .0, f.def_id.index));
        for hir_fn in &hir_funcs_copy {
            self.lower_func(hir_fn);
        }
    }

    fn lower_func(&mut self, hir_fn: &HirFnDef) {
        let Some(&func_id) = self.func_ids.get(&hir_fn.def_id) else {
            self.diags.push(gala_diagnostics::Diagnostic::error(
                gala_diagnostics::codes::UNKNOWN_TYPE,
                format!("missing GIR function ID for {:?}", hir_fn.def_id),
            ));
            return;
        };
        let entry = BlockId(0);

        let mut blocks: HashMap<BlockId, Block> = HashMap::new();
        blocks.insert(
            entry,
            Block { id: entry, nodes: Vec::new(), terminator: None, span: hir_fn.span },
        );

        let mut params = Vec::new();
        let param_tys: Vec<TypeId> = hir_fn
            .params
            .iter()
            .map(|param| self.type_id_for_ty(&gala_types::ast_type_to_ty(&param.ty), param.span))
            .collect();
        let mut env: HashMap<LocalId, ValueId> = HashMap::new();
        for param in &hir_fn.params {
            let v = self.fresh_value_id();
            params.push(v);
            env.insert(param.local_id, v);
        }

        // Lower the body into the entry block
        let last_val =
            self.lower_block(hir_fn, &hir_fn.body, &mut blocks, entry, &mut env, func_id);
        if let Some(entry_block) = blocks.get_mut(&entry) {
            if entry_block.terminator.is_none() {
                entry_block.terminator = Some(Terminator::Return(last_val));
            }
        }

        let ret_ty = hir_fn
            .ret_ty
            .as_ref()
            .map(|ret| self.type_id_for_ty(&gala_types::ast_type_to_ty(ret), hir_fn.span))
            .or_else(|| {
                self.type_of
                    .get(&hir_fn.def_id)
                    .filter(|ty| **ty != Ty::Unit)
                    .map(|ty| self.type_id_for_ty(ty, hir_fn.span))
            });
        let gir_func = GirFunc {
            id: func_id,
            name: hir_fn.ident.clone(),
            params,
            param_tys,
            ret_ty,
            effect: match &hir_fn.effect {
                gala_ast::Effect::Pure => gala_types::Effect::Pure,
                gala_ast::Effect::Quantum => gala_types::Effect::Quantum,
                gala_ast::Effect::Prob => gala_types::Effect::Prob,
            },
            blocks,
            entry_block: entry,
            span: hir_fn.span,
        };

        self.results.insert(func_id, gir_func);
    }

    fn lower_block(
        &mut self,
        hir_fn: &HirFnDef,
        block: &HirBlock,
        blocks: &mut HashMap<BlockId, Block>,
        current_block: BlockId,
        env: &mut HashMap<LocalId, ValueId>,
        func_id: FuncId,
    ) -> Option<ValueId> {
        let mut last_val = None;

        for stmt in &block.stmts {
            match stmt {
                HirStmt::Let(l) => {
                    let val = self.lower_expr(hir_fn, &l.init, blocks, current_block, env, func_id);
                    if let Some(v) = val {
                        let local_ids = if l.local_ids.is_empty() {
                            vec![l.local_id]
                        } else {
                            l.local_ids.clone()
                        };
                        let bindings = pattern_binding_count(&l.pattern);
                        if bindings <= 1 {
                            env.insert(local_ids[0], v);
                        } else {
                            for (index, local_id) in local_ids.into_iter().enumerate() {
                                if let Some(value) =
                                    self.extract_tuple(blocks, current_block, v, index, l.span)
                                {
                                    env.insert(local_id, value);
                                }
                            }
                        }
                    }
                }
                HirStmt::Expr(e) => {
                    last_val = self.lower_expr(hir_fn, e, blocks, current_block, env, func_id);
                    if let HirExpr::Call(call) = e {
                        self.thread_in_place_call(
                            call,
                            last_val,
                            blocks,
                            current_block,
                            env,
                            e_span(e),
                        );
                    }
                }
                HirStmt::Return(e) => {
                    last_val = e.as_ref().and_then(|value| {
                        self.lower_expr(hir_fn, value, blocks, current_block, env, func_id)
                    });
                    if let Some(block_data) = blocks.get_mut(&current_block) {
                        block_data.terminator = Some(Terminator::Return(last_val));
                    }
                    return last_val;
                }
                _ => {}
            }
        }

        if let Some(tail) = &block.tail {
            last_val = self.lower_expr(hir_fn, tail, blocks, current_block, env, func_id);
        }

        last_val
    }

    fn lower_expr(
        &mut self,
        hir_fn: &HirFnDef,
        expr: &HirExpr,
        blocks: &mut HashMap<BlockId, Block>,
        current_block: BlockId,
        env: &mut HashMap<LocalId, ValueId>,
        func_id: FuncId,
    ) -> Option<ValueId> {
        match expr {
            HirExpr::Literal(lit) => {
                let val = self.fresh_value_id();
                let kind = match lit {
                    Literal::Int(i) => NodeKind::Constant(Constant::Int(*i)),
                    Literal::Float(f) => NodeKind::Constant(Constant::Float(*f)),
                    Literal::Bool(b) => NodeKind::Constant(Constant::Bool(*b)),
                    Literal::String(s) => NodeKind::Constant(Constant::String(s.clone())),
                    Literal::Complex { re, im } => {
                        NodeKind::Constant(Constant::Complex { re: *re, im: *im })
                    }
                    Literal::Unit => NodeKind::Constant(Constant::Unit),
                };
                let block = blocks.get_mut(&current_block)?;
                self.emit_node(block, kind, vec![], vec![val], Span::dummy());
                Some(val)
            }
            HirExpr::Ident(_, def_id, _) => {
                let local_id = local_id_from_def_id(def_id).unwrap_or(LocalId(def_id.index));
                env.get(&local_id).copied()
            }
            HirExpr::Binary(b) => {
                let lhs = self.lower_expr(hir_fn, &b.lhs, blocks, current_block, env, func_id);
                let rhs = self.lower_expr(hir_fn, &b.rhs, blocks, current_block, env, func_id);
                let op = match b.op {
                    gala_ast::BinOp::Add => ClassicalOp::Add,
                    gala_ast::BinOp::Sub => ClassicalOp::Sub,
                    gala_ast::BinOp::Mul => ClassicalOp::Mul,
                    gala_ast::BinOp::Div => ClassicalOp::Div,
                    gala_ast::BinOp::Eq => ClassicalOp::Eq,
                    gala_ast::BinOp::Ne => ClassicalOp::Ne,
                    gala_ast::BinOp::Lt => ClassicalOp::Lt,
                    gala_ast::BinOp::Le => ClassicalOp::Le,
                    gala_ast::BinOp::Gt => ClassicalOp::Gt,
                    gala_ast::BinOp::Ge => ClassicalOp::Ge,
                    gala_ast::BinOp::And => ClassicalOp::And,
                    gala_ast::BinOp::Or => ClassicalOp::Or,
                    _ => return None,
                };
                let (Some(lhs), Some(rhs)) = (lhs, rhs) else { return None };
                let result = self.fresh_value_id();
                let block = blocks.get_mut(&current_block)?;
                let operands = vec![lhs, rhs];
                self.emit_node(block, NodeKind::Classical(op), operands, vec![result], b.span);
                Some(result)
            }
            HirExpr::Call(c) => {
                // Check if it's a gate call
                if let HirExpr::Ident(gate_name, _, _) = c.callee.as_ref() {
                    if let Some(gate) = Self::gate_kind_from_name(&gate_name.0) {
                        let arg_vals: Vec<ValueId> = c
                            .args
                            .iter()
                            .filter_map(|a| {
                                self.lower_expr(hir_fn, a, blocks, current_block, env, func_id)
                            })
                            .collect();
                        let result = self.fresh_value_id();
                        let block = blocks.get_mut(&current_block)?;
                        if matches!(gate, GateKind::CX | GateKind::CZ | GateKind::SWAP)
                            && arg_vals.len() == 2
                        {
                            let first = result;
                            let second = self.fresh_value_id();
                            self.emit_node(
                                block,
                                NodeKind::Gate(gate),
                                arg_vals.clone(),
                                vec![first, second],
                                c.span,
                            );
                            let tuple = self.fresh_value_id();
                            self.emit_node(
                                block,
                                NodeKind::Tuple { elements: vec![first, second] },
                                vec![first, second],
                                vec![tuple],
                                c.span,
                            );
                            return Some(tuple);
                        }
                        self.emit_node(block, NodeKind::Gate(gate), arg_vals, vec![result], c.span);
                        return Some(result);
                    }
                }
                // Allocation and measurement are builtins, not ordinary calls.
                if let HirExpr::Ident(name, _, _) = c.callee.as_ref() {
                    if name.0 == "qubit" {
                        let result = self.fresh_value_id();
                        let block = blocks.get_mut(&current_block)?;
                        self.emit_node(
                            block,
                            NodeKind::Alloc(AllocKind::Qubit),
                            Vec::new(),
                            vec![result],
                            c.span,
                        );
                        return Some(result);
                    }
                    if name.0 == "measure" {
                        let target = c.args.first().and_then(|arg| {
                            self.lower_expr(hir_fn, arg, blocks, current_block, env, func_id)
                        })?;
                        let result = self.fresh_value_id();
                        let block = blocks.get_mut(&current_block)?;
                        self.emit_node(
                            block,
                            NodeKind::Measure { target },
                            vec![target],
                            vec![result],
                            c.span,
                        );
                        return Some(result);
                    }
                    if name.0 == "drop" {
                        let target = c.args.first().and_then(|arg| {
                            self.lower_expr(hir_fn, arg, blocks, current_block, env, func_id)
                        })?;
                        let block = blocks.get_mut(&current_block)?;
                        self.emit_node(
                            block,
                            NodeKind::Free { target },
                            vec![target],
                            Vec::new(),
                            c.span,
                        );
                        return None;
                    }
                }

                // For non-builtin calls, resolve the callee to a GIR function.
                let callee_def = match c.callee.as_ref() {
                    HirExpr::Ident(_, def_id, _) => def_id,
                    _ => return None,
                };
                let Some(&callee_func) = self.func_ids.get(callee_def) else {
                    self.diags.push(gala_diagnostics::Diagnostic::error(
                        gala_diagnostics::codes::UNKNOWN_TYPE,
                        format!("cannot lower unresolved function call {:?}", callee_def),
                    ));
                    return None;
                };

                let result = self.fresh_value_id();
                let arg_vals: Vec<ValueId> = c
                    .args
                    .iter()
                    .filter_map(|a| self.lower_expr(hir_fn, a, blocks, current_block, env, func_id))
                    .collect();
                let block = blocks.get_mut(&current_block)?;
                self.emit_node(
                    block,
                    NodeKind::Call { func: callee_func, args: arg_vals.clone() },
                    arg_vals,
                    vec![result],
                    c.span,
                );
                Some(result)
            }
            HirExpr::Block(b) => self.lower_block(hir_fn, b, blocks, current_block, env, func_id),
            HirExpr::Tuple(items) => {
                let elements: Vec<_> = items
                    .iter()
                    .filter_map(|item| {
                        self.lower_expr(hir_fn, item, blocks, current_block, env, func_id)
                    })
                    .collect();
                let result = self.fresh_value_id();
                let block = blocks.get_mut(&current_block)?;
                self.emit_node(
                    block,
                    NodeKind::Tuple { elements: elements.clone() },
                    elements,
                    vec![result],
                    Span::dummy(),
                );
                Some(result)
            }
            HirExpr::If(i) => {
                let cond_val =
                    self.lower_expr(hir_fn, &i.cond, blocks, current_block, env, func_id);
                let then_block = self.fresh_block_id();
                let else_block = self.fresh_block_id();
                let merge_block = self.fresh_block_id();

                let cond = cond_val.unwrap_or(ValueId(0));
                let block = blocks.get_mut(&current_block)?;
                block.terminator =
                    Some(Terminator::CondBranch { cond, then_bb: then_block, else_bb: else_block });

                // Lower then branch
                let mut then_blocks = HashMap::new();
                then_blocks.insert(
                    then_block,
                    Block { id: then_block, nodes: Vec::new(), terminator: None, span: i.span },
                );
                let mut then_env = env.clone();
                let then_val = self.lower_block(
                    hir_fn,
                    &i.then_branch,
                    &mut then_blocks,
                    then_block,
                    &mut then_env,
                    func_id,
                );
                if let Some(then_block_data) = then_blocks.get_mut(&then_block) {
                    if then_block_data.terminator.is_none() {
                        then_block_data.terminator = Some(Terminator::Branch(merge_block));
                    }
                }
                blocks.extend(then_blocks);

                // Lower else branch
                if let Some(else_) = &i.else_branch {
                    let mut else_blocks = HashMap::new();
                    else_blocks.insert(
                        else_block,
                        Block { id: else_block, nodes: Vec::new(), terminator: None, span: i.span },
                    );
                    let mut else_env = env.clone();
                    let else_val = self.lower_expr(
                        hir_fn,
                        else_,
                        &mut else_blocks,
                        else_block,
                        &mut else_env,
                        func_id,
                    );
                    if let Some(else_block_data) = else_blocks.get_mut(&else_block) {
                        if else_block_data.terminator.is_none() {
                            else_block_data.terminator = Some(Terminator::Branch(merge_block));
                        }
                    }
                    blocks.extend(else_blocks);
                    blocks.insert(
                        merge_block,
                        Block {
                            id: merge_block,
                            nodes: Vec::new(),
                            terminator: None,
                            span: i.span,
                        },
                    );
                    if let (Some(then_val), Some(else_val)) = (then_val, else_val) {
                        let result = self.fresh_value_id();
                        let merge = blocks.get_mut(&merge_block)?;
                        self.emit_node(
                            merge,
                            NodeKind::Phi {
                                incoming: vec![(then_block, then_val), (else_block, else_val)],
                            },
                            vec![then_val, else_val],
                            vec![result],
                            i.span,
                        );
                        return Some(result);
                    }
                    return None;
                }

                // Merge block
                blocks.insert(
                    else_block,
                    Block {
                        id: else_block,
                        nodes: Vec::new(),
                        terminator: Some(Terminator::Branch(merge_block)),
                        span: i.span,
                    },
                );
                blocks.insert(
                    merge_block,
                    Block { id: merge_block, nodes: Vec::new(), terminator: None, span: i.span },
                );

                None
            }
            _ => None,
        }
    }

    fn extract_tuple(
        &mut self,
        blocks: &mut HashMap<BlockId, Block>,
        current_block: BlockId,
        tuple: ValueId,
        index: usize,
        span: Span,
    ) -> Option<ValueId> {
        let result = self.fresh_value_id();
        let block = blocks.get_mut(&current_block)?;
        self.emit_node(block, NodeKind::Extract { tuple, index }, vec![tuple], vec![result], span);
        Some(result)
    }

    fn thread_in_place_call(
        &mut self,
        call: &HirCallExpr,
        result: Option<ValueId>,
        blocks: &mut HashMap<BlockId, Block>,
        current_block: BlockId,
        env: &mut HashMap<LocalId, ValueId>,
        span: Span,
    ) {
        let Some(result) = result else { return };
        let HirExpr::Ident(name, _, _) = call.callee.as_ref() else { return };
        if !matches!(
            name.0.as_str(),
            "h" | "x" | "y" | "z" | "s" | "t" | "rx" | "ry" | "rz" | "cx" | "cz" | "swap"
        ) {
            return;
        }
        for (index, arg) in call.args.iter().enumerate() {
            let HirExpr::Ident(_, def_id, _) = arg else { continue };
            let local_id = local_id_from_def_id(def_id).unwrap_or(LocalId(def_id.index));
            let value = if call.args.len() > 1 {
                self.extract_tuple(blocks, current_block, result, index, span)
            } else {
                Some(result)
            };
            if let Some(value) = value {
                env.insert(local_id, value);
            }
        }
    }
}

fn pattern_binding_count(pattern: &gala_ast::Pattern) -> usize {
    match pattern {
        gala_ast::Pattern::Ident(_) => 1,
        gala_ast::Pattern::Tuple(patterns) => patterns.iter().map(pattern_binding_count).sum(),
        gala_ast::Pattern::Struct { fields, .. } => {
            fields.iter().map(|(_, pattern)| pattern_binding_count(pattern)).sum()
        }
        gala_ast::Pattern::Wildcard | gala_ast::Pattern::Literal(_) => 0,
    }
}

fn e_span(expr: &HirExpr) -> Span {
    match expr {
        HirExpr::Call(call) => call.span,
        HirExpr::Binary(binary) => binary.span,
        HirExpr::Unary(unary) => unary.span,
        _ => Span::dummy(),
    }
}

/// Lower typed HIR to GIR.
pub fn lower_hir_to_gir(
    hir_funcs: &HashMap<DefId, HirFnDef>,
    _type_of: &HashMap<DefId, Ty>,
    _effect_of: &HashMap<DefId, Effect>,
) -> Result<Gir, Diagnostics> {
    let mut builder = GirBuilder::new(hir_funcs, _type_of);
    builder.lower_all();

    let gir = Gir {
        funcs: builder.results,
        nodes: builder.nodes,
        types: builder.types,
        consts: HashMap::new(),
    };

    if builder.diags.has_errors() {
        Err(builder.diags)
    } else {
        Ok(gir)
    }
}

impl GirFunc {
    pub fn new_test() -> Self {
        let mut blocks = HashMap::new();
        let entry = BlockId(0);
        blocks.insert(
            entry,
            Block { id: entry, nodes: Vec::new(), terminator: None, span: Span::dummy() },
        );
        GirFunc {
            id: FuncId(0),
            name: gala_ast::Ident::new("test"),
            params: Vec::new(),
            param_tys: Vec::new(),
            ret_ty: None,
            effect: gala_types::Effect::Pure,
            blocks,
            entry_block: entry,
            span: Span::dummy(),
        }
    }
}

impl serde::Serialize for Gir {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Gir", 4)?;
        s.serialize_field("num_funcs", &self.funcs.len())?;
        s.serialize_field("num_nodes", &self.nodes.len())?;
        s.serialize_field("num_types", &self.types.len())?;
        s.serialize_field("num_consts", &self.consts.len())?;
        s.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gala_ast::{self, Ident, Literal};
    use gala_span::FileId;

    #[test]
    fn test_gir_default() {
        let gir = Gir::default();
        assert!(gir.funcs.is_empty());
        assert!(gir.nodes.is_empty());
        assert!(gir.types.is_empty());
    }

    #[test]
    fn test_gir_func_new_test() {
        let func = GirFunc::new_test();
        assert_eq!(func.name.0, "test");
        assert_eq!(func.blocks.len(), 1);
        assert!(func.params.is_empty());
    }

    #[test]
    fn test_gir_func_block_insert() {
        let mut blocks = HashMap::new();
        let b0 = BlockId(0);
        blocks
            .insert(b0, Block { id: b0, nodes: Vec::new(), terminator: None, span: Span::dummy() });
        let b1 = BlockId(1);
        blocks.insert(
            b1,
            Block {
                id: b1,
                nodes: Vec::new(),
                terminator: Some(Terminator::Branch(b0)),
                span: Span::dummy(),
            },
        );
        assert_eq!(blocks.len(), 2);
        assert!(matches!(blocks[&b1].terminator, Some(Terminator::Branch(_))));
    }

    #[test]
    fn test_gate_kind_from_name() {
        assert_eq!(GirBuilder::gate_kind_from_name("h"), Some(GateKind::H));
        assert_eq!(GirBuilder::gate_kind_from_name("cx"), Some(GateKind::CX));
        assert_eq!(GirBuilder::gate_kind_from_name("swap"), Some(GateKind::SWAP));
        assert_eq!(GirBuilder::gate_kind_from_name("rz"), Some(GateKind::Rz));
        assert_eq!(GirBuilder::gate_kind_from_name("nonexistent"), None);
    }

    #[test]
    fn test_lower_simple_function() {
        let mut hir_funcs = HashMap::new();
        let hir_fn = HirFnDef {
            def_id: DefId { crate_id: CrateId(FileId(0)), index: 0 },
            ident: Ident::new("main"),
            generics: Vec::new(),
            params: Vec::new(),
            ret_ty: None,
            effect: gala_ast::Effect::Pure,
            is_extern: false,
            body: HirBlock {
                stmts: vec![HirStmt::Return(Some(Box::new(HirExpr::Literal(Literal::Int(42)))))],
                tail: None,
                span: Span::dummy(),
            },
            span: Span::dummy(),
        };
        hir_funcs.insert(DefId { crate_id: CrateId(FileId(0)), index: 0 }, hir_fn);
        let result = lower_hir_to_gir(&hir_funcs, &HashMap::new(), &HashMap::new());
        assert!(result.is_ok());
        let gir = result.unwrap();
        assert_eq!(gir.funcs.len(), 1);
        assert_eq!(gir.nodes.len(), 1);
        let entry = gir.funcs[&FuncId(0)].blocks[&BlockId(0)].clone();
        assert_eq!(entry.nodes.len(), 1);
        assert!(matches!(entry.terminator, Some(Terminator::Return(Some(_)))));
    }

    #[test]
    fn test_lower_binary_expr() {
        let mut hir_funcs = HashMap::new();
        let hir_fn = HirFnDef {
            def_id: DefId { crate_id: CrateId(FileId(0)), index: 0 },
            ident: Ident::new("add"),
            generics: Vec::new(),
            params: Vec::new(),
            ret_ty: None,
            effect: gala_ast::Effect::Pure,
            is_extern: false,
            body: HirBlock {
                stmts: Vec::new(),
                tail: Some(Box::new(HirExpr::Binary(HirBinaryExpr {
                    lhs: Box::new(HirExpr::Literal(Literal::Int(2))),
                    op: gala_ast::BinOp::Add,
                    rhs: Box::new(HirExpr::Literal(Literal::Int(3))),
                    span: Span::dummy(),
                }))),
                span: Span::dummy(),
            },
            span: Span::dummy(),
        };
        hir_funcs.insert(DefId { crate_id: CrateId(FileId(0)), index: 0 }, hir_fn);
        let result = lower_hir_to_gir(&hir_funcs, &HashMap::new(), &HashMap::new());
        assert!(result.is_ok());
        let gir = result.unwrap();
        assert_eq!(gir.nodes.len(), 3);
    }

    #[test]
    fn test_builder_fresh_ids() {
        let empty: HashMap<DefId, HirFnDef> = HashMap::new();
        let empty_types: HashMap<DefId, Ty> = HashMap::new();
        let mut builder = GirBuilder::new(&empty, &empty_types);
        let f1 = builder.fresh_func_id();
        let f2 = builder.fresh_func_id();
        assert!(f2.0 > f1.0);
        let b1 = builder.fresh_block_id();
        let b2 = builder.fresh_block_id();
        assert!(b2.0 > b1.0);
    }
}
