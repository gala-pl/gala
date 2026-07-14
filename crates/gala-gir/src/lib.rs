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
    pub types: HashMap<TypeId, GirType>,
    pub consts: HashMap<ConstId, GirConst>,
}

#[derive(Debug, Clone)]
pub struct GirFunc {
    pub id: FuncId,
    pub name: Ident,
    pub params: Vec<ValueId>,
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
    Measure { target: ValueId },
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
    next_func_id: u32,
    next_block_id: u32,
    next_node_id: u32,
    next_value_id: u32,
    results: HashMap<FuncId, GirFunc>,
    diags: Diagnostics,
}

impl<'a> GirBuilder<'a> {
    fn new(hir_funcs: &'a HashMap<DefId, HirFnDef>) -> Self {
        GirBuilder {
            hir_funcs,
            next_func_id: 0,
            next_block_id: 1,
            next_node_id: 0,
            next_value_id: 0,
            results: HashMap::new(),
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
        // We'd store the node in a central graph; for now just return the id
        let _ = (kind, operands, results, span);
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
        let hir_funcs_copy = self.hir_funcs.clone();
        for hir_fn in hir_funcs_copy.values() {
            self.lower_func(hir_fn);
        }
    }

    fn lower_func(&mut self, hir_fn: &HirFnDef) {
        let func_id = self.fresh_func_id();
        let entry = BlockId(0);

        let mut blocks: HashMap<BlockId, Block> = HashMap::new();
        blocks.insert(
            entry,
            Block { id: entry, nodes: Vec::new(), terminator: None, span: hir_fn.span },
        );

        let mut params = Vec::new();
        let mut env: HashMap<LocalId, ValueId> = HashMap::new();
        for param in &hir_fn.params {
            let v = self.fresh_value_id();
            params.push(v);
            env.insert(param.local_id, v);
        }

        // Lower the body into the entry block
        self.lower_block(hir_fn, &hir_fn.body, &mut blocks, entry, &env, func_id);

        let gir_func = GirFunc {
            id: func_id,
            name: hir_fn.ident.clone(),
            params,
            ret_ty: None,
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
        env: &HashMap<LocalId, ValueId>,
        func_id: FuncId,
    ) -> Option<ValueId> {
        let mut last_val = None;

        for stmt in &block.stmts {
            match stmt {
                HirStmt::Let(l) => {
                    let val = self.lower_expr(hir_fn, &l.init, blocks, current_block, env, func_id);
                    if let Some(v) = val {
                        // In a full impl we'd track this binding
                        let _ = v;
                    }
                }
                HirStmt::Expr(e) => {
                    last_val = self.lower_expr(hir_fn, e, blocks, current_block, env, func_id);
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
        env: &HashMap<LocalId, ValueId>,
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
            HirExpr::Ident(_, _, _) => {
                // For now, just return a fresh value
                // In a full impl, we'd look up the value from env
                Some(self.fresh_value_id())
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
                let result = self.fresh_value_id();
                let block = blocks.get_mut(&current_block)?;
                let operands = vec![lhs.unwrap_or(ValueId(0)), rhs.unwrap_or(ValueId(0))];
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
                        self.emit_node(block, NodeKind::Gate(gate), arg_vals, vec![result], c.span);
                        return Some(result);
                    }
                }
                // For non-gate calls, emit a call node
                let result = self.fresh_value_id();
                let arg_vals: Vec<ValueId> = c
                    .args
                    .iter()
                    .filter_map(|a| self.lower_expr(hir_fn, a, blocks, current_block, env, func_id))
                    .collect();
                // TODO: map the callee's DefId to a FuncId; for now fall back to the
                // current function for all call targets.
                let block = blocks.get_mut(&current_block)?;
                self.emit_node(
                    block,
                    NodeKind::Call { func: func_id, args: arg_vals.clone() },
                    arg_vals,
                    vec![result],
                    c.span,
                );
                Some(result)
            }
            HirExpr::Block(b) => self.lower_block(hir_fn, b, blocks, current_block, env, func_id),
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
                self.lower_block(
                    hir_fn,
                    &i.then_branch,
                    &mut then_blocks,
                    then_block,
                    env,
                    func_id,
                );
                if let Some(then_block_data) = then_blocks.get_mut(&then_block) {
                    then_block_data.terminator = Some(Terminator::Branch(merge_block));
                }
                blocks.extend(then_blocks);

                // Lower else branch
                if let Some(else_) = &i.else_branch {
                    let mut else_blocks = HashMap::new();
                    else_blocks.insert(
                        else_block,
                        Block { id: else_block, nodes: Vec::new(), terminator: None, span: i.span },
                    );
                    self.lower_expr(hir_fn, else_, &mut else_blocks, else_block, env, func_id);
                    if let Some(else_block_data) = else_blocks.get_mut(&else_block) {
                        else_block_data.terminator = Some(Terminator::Branch(merge_block));
                    }
                    blocks.extend(else_blocks);
                }

                // Merge block
                blocks.insert(
                    merge_block,
                    Block { id: merge_block, nodes: Vec::new(), terminator: None, span: i.span },
                );

                Some(self.fresh_value_id())
            }
            _ => None,
        }
    }
}

/// Lower typed HIR to GIR.
pub fn lower_hir_to_gir(
    hir_funcs: &HashMap<DefId, HirFnDef>,
    _type_of: &HashMap<DefId, Ty>,
    _effect_of: &HashMap<DefId, Effect>,
) -> Result<Gir, Diagnostics> {
    let mut builder = GirBuilder::new(hir_funcs);
    builder.lower_all();

    let gir = Gir { funcs: builder.results, types: HashMap::new(), consts: HashMap::new() };

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
        let mut s = serializer.serialize_struct("Gir", 3)?;
        s.serialize_field("num_funcs", &self.funcs.len())?;
        s.serialize_field("num_types", &self.types.len())?;
        s.serialize_field("num_consts", &self.consts.len())?;
        s.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gala_ast::{self, Ident, Literal};
    use gala_span::{ByteSpan, FileId};

    #[test]
    fn test_gir_default() {
        let gir = Gir::default();
        assert!(gir.funcs.is_empty());
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
    }

    #[test]
    fn test_builder_fresh_ids() {
        let empty: HashMap<DefId, HirFnDef> = HashMap::new();
        let mut builder = GirBuilder::new(&empty);
        let f1 = builder.fresh_func_id();
        let f2 = builder.fresh_func_id();
        assert!(f2.0 > f1.0);
        let b1 = builder.fresh_block_id();
        let b2 = builder.fresh_block_id();
        assert!(b2.0 > b1.0);
    }
}
