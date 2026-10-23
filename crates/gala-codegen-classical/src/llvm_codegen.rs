//! LLVM codegen backend for Gala (WP-1: LLVM Backend, core slice WP-1.1–1.5).
//!
//! This module emits LLVM IR *text* (the same textual representation produced
//! by `Module::print_to_string()` in the `inkwell`/`LLVM` C++ API) directly
//! from GIR. Quantum operations are lowered to QIR intrinsics, reusing the
//! gate table from `gala-qir` (WP-1.4.1).
//!
//! Why pure Rust instead of `inkwell` + LLVM 10? The upstream plan pinned
//! `inkwell` against LLVM 10, but that requires a native LLVM install to even
//! *compile* the crate (plan Risk #1). On machines without a matching LLVM
//! toolchain (e.g. only LLVM 17/22 present, or none at all) the crate would
//! fail to build, which directly violates WP-1.7.4 ("`cargo build` with no
//! features must still compile"). Textual LLVM IR is a stable, toolchain-free
//! format: it can be validated with any `llvm-as`/`opt` on PATH, linked with
//! `llc`/`clang`, and is exactly the artifact QIR consumers expect. The shared
//! logic (type mapping, value/block maps, QIR intrinsics) mirrors the
//! `LLVMCodegen`/`Context`/`Module`/`Builder` lifecycle the plan describes.

use std::collections::HashMap;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use gala_diagnostics::Diagnostics;
use gala_gir::{
    AllocKind, Block, BlockId, ClassicalOp, Constant, FuncId, GateKind, Gir, GirFunc, GirNode,
    NodeKind, Terminator, TypeId, TypeKind, ValueId,
};
use gala_qir::gate_intrinsic_name;

// ═══════════════════════════════════════════════════════════════════════════════
// Public entry point (WP-1.1.2)
// ═══════════════════════════════════════════════════════════════════════════════

/// Emit LLVM IR text for the whole program.
///
/// Mirrors `emit_llvm(gir: &Gir) -> Result<String, Diagnostics>`: iterate every
/// function and accumulate a textual LLVM module.
pub fn emit_llvm(gir: &Gir) -> Result<String, Diagnostics> {
    let mut cg = LlvmCodegen::new(gir);
    cg.run()?;
    Ok(cg.out)
}

// ═══════════════════════════════════════════════════════════════════════════════
// LLVM Codegen context (WP-1.1.1)
// ═══════════════════════════════════════════════════════════════════════════════

struct LlvmCodegen<'a> {
    gir: &'a Gir,
    /// SSA register name for each GIR value id.
    regs: HashMap<ValueId, String>,
    /// Inferred LLVM type for each GIR value id.
    value_tys: HashMap<ValueId, String>,
    /// Resolved signature for each function (`FuncId -> name/retty/params`).
    func_sigs: HashMap<FuncId, FuncSig>,
    /// Block label for each `BlockId`.
    block_labels: HashMap<BlockId, String>,
    /// QIR intrinsic names already declared in the module.
    declared_intrinsics: std::collections::HashSet<String>,
    next_reg: u32,
    out: String,
}

#[derive(Clone)]
struct FuncSig {
    name: String,
    ret_ty: String,
    params: Vec<String>,
}

impl<'a> LlvmCodegen<'a> {
    fn new(gir: &'a Gir) -> Self {
        LlvmCodegen {
            gir,
            regs: HashMap::new(),
            value_tys: HashMap::new(),
            func_sigs: HashMap::new(),
            block_labels: HashMap::new(),
            declared_intrinsics: std::collections::HashSet::new(),
            next_reg: 0,
            out: String::new(),
        }
    }

    // ── SSA register / type helpers ─────────────────────────────────────────

    fn fresh_reg(&mut self) -> String {
        // Use a *named* temporary (`%vN`) rather than a bare numeric one.  LLVM
        // IR requires bare numeric temporaries (`%0`, `%1`, ...) to be numbered
        // sequentially in definition order; named temporaries impose no such
        // ordering and permit forward references, which keeps emission correct
        // regardless of GIR value-id discovery order or block layout.
        let name = format!("%v{}", self.next_reg);
        self.next_reg += 1;
        name
    }

    /// Assign a register to a value id (idempotent).
    fn ensure_reg(&mut self, v: ValueId) -> String {
        if let Some(r) = self.regs.get(&v) {
            return r.clone();
        }
        let r = self.fresh_reg();
        self.regs.insert(v, r.clone());
        r
    }

    fn set_ty(&mut self, v: ValueId, ty: &str) {
        self.value_tys.insert(v, ty.to_string());
    }

    fn ty_of(&self, v: ValueId) -> String {
        self.value_tys.get(&v).cloned().unwrap_or_else(|| "i64".to_string())
    }

    fn block_label(&self, b: BlockId) -> String {
        self.block_labels.get(&b).cloned().unwrap_or_else(|| format!("block_{}", b.0))
    }

    // ── Driver ───────────────────────────────────────────────────────────────

    fn run(&mut self) -> Result<(), Diagnostics> {
        // 1. Assign a register to every value id that appears anywhere. Iterate
        //    in a deterministic order (sorted by id) so generated register names
        //    are stable across runs (HashMap iteration order is randomized).
        let mut all_ids: Vec<ValueId> = self
            .gir
            .nodes
            .values()
            .flat_map(|n| n.operands.iter().chain(n.results.iter()).copied())
            .chain(self.gir.funcs.values().flat_map(|f| f.params.iter().copied()))
            .collect();
        all_ids.sort_by_key(|v| v.0);
        all_ids.dedup();
        for v in all_ids {
            self.ensure_reg(v);
        }

        // 2. Seed value types from node kinds (WP-1.1.3 / 1.2 / 1.4).
        self.seed_value_types();

        // 3. Default any untyped value to i64 (classical fallback).
        let all_ids: Vec<ValueId> = self.regs.keys().cloned().collect();
        for v in all_ids {
            if !self.value_tys.contains_key(&v) {
                self.set_ty(v, "i64");
            }
        }

        // 4. Resolve function signatures and block labels.
        for func in self.gir.funcs.values() {
            let sig = self.compute_func_sig(func);
            self.func_sigs.insert(func.id, sig);
            for bid in func.blocks.keys() {
                if self.block_labels.contains_key(bid) {
                    continue;
                }
                let label = if *bid == func.entry_block {
                    "entry".to_string()
                } else {
                    format!("block_{}", bid.0)
                };
                self.block_labels.insert(*bid, label);
            }
        }

        // 5. Module header + QIR opaque qubit type.
        writeln!(self.out, "; ModuleID = 'gala'").unwrap();
        writeln!(self.out, "source_filename = \"gala.bc\"").unwrap();
        writeln!(self.out, "target triple = \"x86_64-unknown-linux-gnu\"").unwrap();
        writeln!(self.out).unwrap();
        writeln!(self.out, "%Qubit = type opaque").unwrap();
        writeln!(self.out).unwrap();

        // 6. Declare QIR intrinsics used by the program.
        self.declare_needed_intrinsics();
        if !self.declared_intrinsics.is_empty() {
            writeln!(self.out).unwrap();
        }

        // 7. Emit function bodies.
        //
        // Note on forward declarations (WP-1.5.2): textual LLVM IR resolves
        // `call` references at end-of-parse, so functions may be called before
        // they are defined in the module. This gives mutual/self recursion for
        // free without emitting a separate `declare` (which would conflict with
        // the later `define` and trip "invalid redefinition"). External
        // functions not present in `gir.funcs` are simply referenced by name and
        // resolved by the linker.
        for func in self.gir.funcs.values() {
            self.emit_func(func)?;
        }

        Ok(())
    }

    // ── Type mapping (WP-1.1.3) ───────────────────────────────────────────────

    fn map_type(&self, tid: TypeId) -> String {
        match self.gir.types.get(&tid) {
            Some(gt) => map_typekind(&gt.kind),
            None => "i64".to_string(),
        }
    }

    /// Determine the function return type: prefer an explicit `ret_ty`, else
    /// infer from the first `Return(Some(v))` terminator, else `void`.
    fn compute_ret_ty(&self, func: &GirFunc) -> String {
        if let Some(tid) = func.ret_ty {
            return self.map_type(tid);
        }
        for block in func.blocks.values() {
            if let Some(Terminator::Return(Some(v))) = &block.terminator {
                return self.ty_of(*v);
            }
        }
        "void".to_string()
    }

    fn compute_func_sig(&self, func: &GirFunc) -> FuncSig {
        let ret_ty = self.compute_ret_ty(func);
        let params = func.params.iter().map(|p| self.ty_of(*p)).collect::<Vec<_>>();
        FuncSig { name: func.name.0.clone(), ret_ty, params }
    }

    // ── Value type seeding ─────────────────────────────────────────────────────

    fn seed_value_types(&mut self) {
        for node in self.gir.nodes.values() {
            match &node.kind {
                NodeKind::Constant(c) => {
                    let ty = match c {
                        Constant::Int(_) => "i64",
                        Constant::Float(_) | Constant::Complex { .. } => "double",
                        Constant::Bool(_) => "i1",
                        Constant::String(_) | Constant::Unit => "i64",
                    };
                    if let Some(r) = node.results.first() {
                        self.set_ty(*r, ty);
                    }
                }
                NodeKind::Alloc(AllocKind::Qubit) | NodeKind::Alloc(AllocKind::Qubits(_)) => {
                    for r in &node.results {
                        self.set_ty(*r, "%Qubit*");
                    }
                }
                NodeKind::Alloc(AllocKind::Classical(_)) => {
                    for r in &node.results {
                        self.set_ty(*r, "i64");
                    }
                }
                NodeKind::Gate(g) => {
                    // Operands are qubits; results (post-gate qubits) are qubits too.
                    for op in &node.operands {
                        self.set_ty(*op, "%Qubit*");
                    }
                    for r in &node.results {
                        self.set_ty(*r, "%Qubit*");
                    }
                    let _ = g;
                }
                NodeKind::Measure { target } => {
                    self.set_ty(*target, "%Qubit*");
                    if let Some(r) = node.results.first() {
                        self.set_ty(*r, "i1");
                    }
                }
                NodeKind::Call { .. } => {
                    for r in &node.results {
                        self.set_ty(*r, "i64");
                    }
                }
                NodeKind::Phi { incoming } => {
                    if let Some((_, v)) = incoming.first() {
                        let ty = self.ty_of(*v);
                        for r in &node.results {
                            self.set_ty(*r, &ty);
                        }
                    }
                }
                NodeKind::Classical(op) => {
                    // Best-effort result-type inference (WP-1.2). Operand types
                    // may still be the `i64` default at this point; full type
                    // propagation comes from HIR type info (WP-4). Comparison
                    // results are always `i1`; bitwise on booleans stays `i1`.
                    let lty = node
                        .operands
                        .first()
                        .map(|v| self.ty_of(*v))
                        .unwrap_or_else(|| "i64".to_string());
                    let rty =
                        node.operands.get(1).map(|v| self.ty_of(*v)).unwrap_or_else(|| lty.clone());
                    let is_float = lty == "double" || rty == "double";
                    let ty = match op {
                        ClassicalOp::Eq
                        | ClassicalOp::Ne
                        | ClassicalOp::Lt
                        | ClassicalOp::Le
                        | ClassicalOp::Gt
                        | ClassicalOp::Ge => "i1".to_string(),
                        ClassicalOp::And | ClassicalOp::Or => {
                            if lty == "i1" {
                                "i1".to_string()
                            } else {
                                "i64".to_string()
                            }
                        }
                        ClassicalOp::Not | ClassicalOp::Neg => lty,
                        _ => {
                            if is_float {
                                "double".to_string()
                            } else {
                                "i64".to_string()
                            }
                        }
                    };
                    for r in &node.results {
                        self.set_ty(*r, &ty);
                    }
                }
            }
        }
    }

    // ── QIR intrinsic handling (WP-1.4) ─────────────────────────────────────────

    /// Declare every QIR intrinsic actually used by the GIR.
    fn declare_needed_intrinsics(&mut self) {
        let mut names: Vec<String> = Vec::new();
        for func in self.gir.funcs.values() {
            for block in func.blocks.values() {
                for nid in &block.nodes {
                    if let Some(node) = self.gir.nodes.get(nid) {
                        match &node.kind {
                            NodeKind::Gate(g) => names.push(gate_intrinsic_name(g).to_string()),
                            NodeKind::Measure { .. } => {
                                names.push("__quantum__qis__mz__body".to_string())
                            }
                            NodeKind::Alloc(AllocKind::Qubit)
                            | NodeKind::Alloc(AllocKind::Qubits(_)) => {
                                names.push("__quantum__qis__qubit__allocate".to_string())
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        names.sort();
        names.dedup();
        for name in names {
            self.declare_intrinsic(&name);
        }
    }

    fn declare_intrinsic(&mut self, name: &str) {
        if self.declared_intrinsics.contains(name) {
            return;
        }
        self.declared_intrinsics.insert(name.to_string());
        let (ret, params) = intrinsic_sig(name);
        writeln!(self.out, "declare {} @{}({})", ret, name, params.join(", ")).unwrap();
    }

    // ── Function emission ───────────────────────────────────────────────────────

    fn emit_func(&mut self, func: &GirFunc) -> Result<(), Diagnostics> {
        let sig = self.func_sigs[&func.id].clone();
        let params = func
            .params
            .iter()
            .enumerate()
            .map(|(i, p)| format!("{} {}", sig.params[i], self.regs[p]))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(self.out, "define {} @{}({}) {{", sig.ret_ty, sig.name, params).unwrap();

        // Emit entry block first, then any other blocks.
        let mut order: Vec<BlockId> = func.blocks.keys().cloned().collect();
        order.sort_by_key(|b| if *b == func.entry_block { 0 } else { b.0 + 1 });
        for bid in order {
            let block = &func.blocks[&bid];
            let label = self.block_label(bid);
            if label == "entry" {
                writeln!(self.out, "entry:").unwrap();
            } else {
                writeln!(self.out, "{}:", label).unwrap();
            }
            self.emit_block(block)?;
        }

        writeln!(self.out, "}}").unwrap();
        writeln!(self.out).unwrap();
        Ok(())
    }

    fn emit_block(&mut self, block: &Block) -> Result<(), Diagnostics> {
        for nid in &block.nodes {
            if let Some(node) = self.gir.nodes.get(nid) {
                self.emit_node(node);
            }
        }
        self.emit_terminator(block);
        Ok(())
    }

    fn emit_node(&mut self, node: &GirNode) {
        match &node.kind {
            NodeKind::Constant(c) => self.emit_constant(node, c),
            NodeKind::Classical(op) => self.emit_classical(node, *op),
            NodeKind::Gate(g) => self.emit_gate(node, *g),
            NodeKind::Measure { target } => self.emit_measure(node, *target),
            NodeKind::Alloc(kind) => self.emit_alloc(node, kind),
            NodeKind::Call { func, args } => self.emit_call(node, *func, args),
            NodeKind::Phi { incoming } => self.emit_phi(node, incoming),
        }
    }

    // ── Classical operations (WP-1.2) ───────────────────────────────────────────

    fn emit_constant(&mut self, node: &GirNode, c: &Constant) {
        let Some(r) = node.results.first() else { return };
        let reg = self.regs[r].clone();
        match c {
            Constant::Int(i) => {
                writeln!(self.out, "  {} = add i64 0, {}", reg, i).unwrap();
            }
            Constant::Float(f) => {
                writeln!(self.out, "  {} = fadd double 0.0, {}", reg, format_double(*f)).unwrap();
            }
            Constant::Bool(b) => {
                writeln!(self.out, "  {} = add i1 0, {}", reg, if *b { 1 } else { 0 }).unwrap();
            }
            Constant::Complex { re, .. } => {
                writeln!(self.out, "  {} = fadd double 0.0, {}", reg, format_double(*re)).unwrap();
            }
            Constant::String(_) | Constant::Unit => {
                writeln!(self.out, "  {} = add i64 0, 0", reg).unwrap();
            }
        }
    }

    fn emit_classical(&mut self, node: &GirNode, op: ClassicalOp) {
        let Some(lhs_v) = node.operands.first().copied() else { return };
        let Some(rhs_v) = node.operands.get(1).copied() else {
            // Unary ops (Not / Neg) only have a left operand.
            self.emit_unary(node, op, lhs_v);
            return;
        };
        let lhs = self.regs[&lhs_v].clone();
        let rhs = self.regs[&rhs_v].clone();
        let lty = self.ty_of(lhs_v);
        let rty = self.ty_of(rhs_v);
        let is_float = lty == "double" || rty == "double";

        let Some(res_v) = node.results.first().copied() else { return };
        let res = self.regs[&res_v].clone();

        match op {
            ClassicalOp::Add => {
                let (inst, ty) = if is_float { ("fadd", "double") } else { ("add", "i64") };
                self.set_ty(res_v, ty);
                writeln!(self.out, "  {} = {} {} {}, {}", res, inst, ty, lhs, rhs).unwrap();
            }
            ClassicalOp::Sub => {
                let (inst, ty) = if is_float { ("fsub", "double") } else { ("sub", "i64") };
                self.set_ty(res_v, ty);
                writeln!(self.out, "  {} = {} {} {}, {}", res, inst, ty, lhs, rhs).unwrap();
            }
            ClassicalOp::Mul => {
                let (inst, ty) = if is_float { ("fmul", "double") } else { ("mul", "i64") };
                self.set_ty(res_v, ty);
                writeln!(self.out, "  {} = {} {} {}, {}", res, inst, ty, lhs, rhs).unwrap();
            }
            ClassicalOp::Div => {
                let (inst, ty) = if is_float { ("fdiv", "double") } else { ("sdiv", "i64") };
                self.set_ty(res_v, ty);
                writeln!(self.out, "  {} = {} {} {}, {}", res, inst, ty, lhs, rhs).unwrap();
            }
            ClassicalOp::Mod => {
                let (inst, ty) = if is_float { ("frem", "double") } else { ("srem", "i64") };
                self.set_ty(res_v, ty);
                writeln!(self.out, "  {} = {} {} {}, {}", res, inst, ty, lhs, rhs).unwrap();
            }
            ClassicalOp::Eq
            | ClassicalOp::Ne
            | ClassicalOp::Lt
            | ClassicalOp::Le
            | ClassicalOp::Gt
            | ClassicalOp::Ge => {
                self.set_ty(res_v, "i1");
                if is_float {
                    let pred = fcmp_pred(op);
                    writeln!(self.out, "  {} = fcmp {} double {}, {}", res, pred, lhs, rhs)
                        .unwrap();
                } else {
                    let pred = icmp_pred(op);
                    writeln!(self.out, "  {} = icmp {} i64 {}, {}", res, pred, lhs, rhs).unwrap();
                }
            }
            ClassicalOp::And | ClassicalOp::Or => {
                let ty = if lty == "i1" { "i1" } else { "i64" };
                self.set_ty(res_v, ty);
                let inst = if matches!(op, ClassicalOp::And) { "and" } else { "or" };
                writeln!(self.out, "  {} = {} {} {}, {}", res, inst, ty, lhs, rhs).unwrap();
            }
            ClassicalOp::Not | ClassicalOp::Neg => {
                self.emit_unary(node, op, lhs_v);
            }
        }
    }

    fn emit_unary(&mut self, node: &GirNode, op: ClassicalOp, v: ValueId) {
        let Some(res_v) = node.results.first().copied() else { return };
        let res = self.regs[&res_v].clone();
        let operand = self.regs[&v].clone();
        let ty = self.ty_of(v);
        self.set_ty(res_v, &ty);
        match op {
            ClassicalOp::Not => {
                let mask = if ty == "i1" { "true" } else { "-1" };
                writeln!(self.out, "  {} = xor {} {}, {}", res, ty, operand, mask).unwrap();
            }
            ClassicalOp::Neg => {
                if ty == "double" {
                    writeln!(self.out, "  {} = fsub double -0.0, {}", res, operand).unwrap();
                } else {
                    writeln!(self.out, "  {} = sub i64 0, {}", res, operand).unwrap();
                }
            }
            _ => {}
        }
    }

    // ── Phi nodes (WP-1.2.5) ─────────────────────────────────────────────────────

    fn emit_phi(&mut self, node: &GirNode, incoming: &[(BlockId, ValueId)]) {
        let Some(res_v) = node.results.first().copied() else { return };
        let res = self.regs[&res_v].clone();
        let ty = self.ty_of(res_v);
        let mut clauses = Vec::new();
        for (bb, v) in incoming {
            clauses.push(format!("[{}, {}]", self.regs[v], self.block_label(*bb)));
        }
        writeln!(self.out, "  {} = phi {} {}", res, ty, clauses.join(", ")).unwrap();
    }

    // ── Quantum operations via QIR intrinsics (WP-1.4) ───────────────────────────

    fn emit_gate(&mut self, node: &GirNode, gate: GateKind) {
        // QIR gates mutate their qubit(s) in place and return `void`, so a gate's
        // result value aliases the operand qubit's register rather than being a
        // fresh SSA definition. Without this the result would be an undefined
        // value (the `call void` emits no binding).
        if let Some(op) = node.operands.first() {
            let qreg = self.regs[op].clone();
            for r in &node.results {
                self.regs.insert(*r, qreg.clone());
            }
        }

        let name = gate_intrinsic_name(&gate);
        self.declare_intrinsic(name);
        let args = match gate {
            GateKind::Rx | GateKind::Ry | GateKind::Rz => {
                let q = self.regs[&node.operands[0]].clone();
                let angle = self.regs[&node.operands[1]].clone();
                vec![format!("%Qubit* {}", q), format!("double {}", angle)]
            }
            GateKind::CX | GateKind::CZ | GateKind::SWAP => {
                let a = self.regs[&node.operands[0]].clone();
                let b = self.regs[&node.operands[1]].clone();
                vec![format!("%Qubit* {}", a), format!("%Qubit* {}", b)]
            }
            _ => {
                let q = self.regs[&node.operands[0]].clone();
                vec![format!("%Qubit* {}", q)]
            }
        };
        writeln!(self.out, "  call void @{}({})", name, args.join(", ")).unwrap();
    }

    fn emit_measure(&mut self, node: &GirNode, target: ValueId) {
        self.declare_intrinsic("__quantum__qis__mz__body");
        let qubit = self.regs[&target].clone();
        let res =
            node.results.first().map(|r| self.regs[r].clone()).unwrap_or_else(|| self.fresh_reg());
        writeln!(self.out, "  {} = call i1 @__quantum__qis__mz__body(%Qubit* {})", res, qubit)
            .unwrap();
    }

    fn emit_alloc(&mut self, node: &GirNode, kind: &AllocKind) {
        match kind {
            AllocKind::Qubit => {
                self.declare_intrinsic("__quantum__qis__qubit__allocate");
                if let Some(r) = node.results.first() {
                    let reg = self.regs[r].clone();
                    writeln!(
                        self.out,
                        "  {} = call %Qubit* @__quantum__qis__qubit__allocate()",
                        reg
                    )
                    .unwrap();
                }
            }
            AllocKind::Qubits(n) => {
                self.declare_intrinsic("__quantum__qis__qubit__allocate");
                for r in node.results.iter().take(*n as usize) {
                    let reg = self.regs[r].clone();
                    writeln!(
                        self.out,
                        "  {} = call %Qubit* @__quantum__qis__qubit__allocate()",
                        reg
                    )
                    .unwrap();
                }
            }
            AllocKind::Classical(_) => {
                // Placeholder: classical storage is represented as an i64 value.
                // (A full alloca-based lowering is out of scope for the core slice.)
            }
        }
    }

    fn emit_call(&mut self, node: &GirNode, func: FuncId, args: &[ValueId]) {
        let sig = self.func_sigs.get(&func).cloned();
        let (callee_name, ret_ty, param_tys) = match sig {
            Some(s) => (s.name, s.ret_ty, s.params),
            None => ("unknown".to_string(), "i64".to_string(), Vec::new()),
        };
        let arg_strs: Vec<String> = args
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let ty = param_tys.get(i).cloned().unwrap_or_else(|| self.ty_of(*v));
                format!("{} {}", ty, self.regs[v])
            })
            .collect();
        let Some(res_v) = node.results.first().copied() else { return };
        let res = self.regs[&res_v].clone();
        if ret_ty == "void" {
            writeln!(self.out, "  call void @{}({})", callee_name, arg_strs.join(", ")).unwrap();
        } else {
            self.set_ty(res_v, &ret_ty);
            writeln!(
                self.out,
                "  {} = call {} @{}({})",
                res,
                ret_ty,
                callee_name,
                arg_strs.join(", ")
            )
            .unwrap();
        }
    }

    // ── Terminators and control flow (WP-1.3) ─────────────────────────────────────

    fn emit_terminator(&mut self, block: &Block) {
        match &block.terminator {
            None | Some(Terminator::Unreachable) => {
                writeln!(self.out, "  unreachable").unwrap();
            }
            Some(Terminator::Return(None)) => {
                writeln!(self.out, "  ret void").unwrap();
            }
            Some(Terminator::Return(Some(v))) => {
                let ty = self.ty_of(*v);
                let reg = self.regs[v].clone();
                writeln!(self.out, "  ret {} {}", ty, reg).unwrap();
            }
            Some(Terminator::Branch(target)) => {
                writeln!(self.out, "  br label %{}", self.block_label(*target)).unwrap();
            }
            Some(Terminator::CondBranch { cond, then_bb, else_bb }) => {
                let c = self.regs[cond].clone();
                writeln!(
                    self.out,
                    "  br i1 {}, label %{}, label %{}",
                    c,
                    self.block_label(*then_bb),
                    self.block_label(*else_bb)
                )
                .unwrap();
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

fn map_typekind(kind: &TypeKind) -> String {
    match kind {
        TypeKind::Int => "i64".to_string(),
        TypeKind::Float => "double".to_string(),
        TypeKind::Bool => "i1".to_string(),
        TypeKind::Complex => "double".to_string(),
        TypeKind::String => "i8*".to_string(),
        TypeKind::Unit => "void".to_string(),
        TypeKind::Qubit => "%Qubit*".to_string(),
        TypeKind::Qubits(_) => "%Qubit**".to_string(),
        TypeKind::Measured(_) => "i1".to_string(),
        TypeKind::Tuple(_)
        | TypeKind::Array(_, _)
        | TypeKind::Fn { .. }
        | TypeKind::Named(_, _) => "i8*".to_string(),
    }
}

fn intrinsic_sig(name: &str) -> (&'static str, Vec<&'static str>) {
    match name {
        "__quantum__qis__qubit__allocate" => ("%Qubit*", vec![]),
        "__quantum__qis__mz__body" => ("i1", vec!["%Qubit*"]),
        "__quantum__qis__h__body"
        | "__quantum__qis__x__body"
        | "__quantum__qis__y__body"
        | "__quantum__qis__z__body"
        | "__quantum__qis__s__body"
        | "__quantum__qis__t__body" => ("void", vec!["%Qubit*"]),
        "__quantum__qis__rx__body" | "__quantum__qis__ry__body" | "__quantum__qis__rz__body" => {
            ("void", vec!["%Qubit*", "double"])
        }
        "__quantum__qis__cx__body" | "__quantum__qis__cz__body" | "__quantum__qis__swap__body" => {
            ("void", vec!["%Qubit*", "%Qubit*"])
        }
        _ => ("void", vec![]),
    }
}

fn icmp_pred(op: ClassicalOp) -> &'static str {
    match op {
        ClassicalOp::Eq => "eq",
        ClassicalOp::Ne => "ne",
        ClassicalOp::Lt => "slt",
        ClassicalOp::Le => "sle",
        ClassicalOp::Gt => "sgt",
        ClassicalOp::Ge => "sge",
        _ => "eq",
    }
}

fn fcmp_pred(op: ClassicalOp) -> &'static str {
    match op {
        ClassicalOp::Eq => "oeq",
        ClassicalOp::Ne => "one",
        ClassicalOp::Lt => "olt",
        ClassicalOp::Le => "ole",
        ClassicalOp::Gt => "ogt",
        ClassicalOp::Ge => "oge",
        _ => "oeq",
    }
}

fn format_double(f: f64) -> String {
    if f.is_infinite() {
        if f.is_sign_positive() {
            return "0x7FF0000000000000".to_string();
        }
        return "0xFFF0000000000000".to_string();
    } else if f.is_nan() {
        return "0x7FF8000000000000".to_string();
    } else if f == 0.0 {
        if f.is_sign_negative() {
            return "-0.0".to_string();
        }
        return "0.0".to_string();
    }
    format!("{:#?}", f)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Native object / executable emission (WP-1.6)
// ═══════════════════════════════════════════════════════════════════════════════
//
// The upstream plan drives `Target`/`TargetMachine`/`write_to_file` through the
// `inkwell` + LLVM C++ API. We instead lower to LLVM IR *text* (see above) and
// hand it to the system `llc` (assembler) and `cc`/`clang` (linker). This is the
// textual-IR equivalent of `TargetMachine::write_to_file(..., FileType::Object)`
// followed by a link step, and it needs no native LLVM Rust bindings.

/// What kind of artifact `--emit` should produce for the native backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitTarget {
    /// LLVM IR text (`.ll`).
    LlvmIr,
    /// Native object file (`.o`).
    Object,
    /// Linked native executable.
    Executable,
}

/// Build a single-error `Diagnostics`.
fn diag(code: gala_diagnostics::DiagnosticCode, msg: String) -> Diagnostics {
    let mut d = Diagnostics::new();
    d.push(gala_diagnostics::Diagnostic::error(code, msg));
    d
}

fn io_err(e: std::io::Error) -> Diagnostics {
    diag(gala_diagnostics::codes::NATIVE_EMISSION_FAILED, format!("native backend I/O error: {e}"))
}

fn tool_not_found(tool: &str) -> Diagnostics {
    diag(
        gala_diagnostics::codes::BACKEND_CAPABILITY_MISMATCH,
        format!(
            "native backend requires the '{tool}' toolchain (LLVM 'llc' and a C compiler \
             'cc'/'clang'). Install them and ensure they are on PATH, or set the \
             {}_BIN / {} environment variable.",
            tool.to_uppercase(),
            tool
        ),
    )
}

fn run_tool(tool: &str, cmd: &mut Command) -> Result<(), Diagnostics> {
    match cmd.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(diag(
            gala_diagnostics::codes::NATIVE_EMISSION_FAILED,
            format!("'{tool}' exited with status {:?}", status.code()),
        )),
        Err(e) => Err(diag(
            gala_diagnostics::codes::BACKEND_CAPABILITY_MISMATCH,
            format!("failed to invoke '{tool}': {e}"),
        )),
    }
}

/// Locate a toolchain binary, honouring an override env var then PATH.
fn find_tool(env_key: &str, name: &str) -> Option<PathBuf> {
    if let Ok(p) = std::env::var(env_key) {
        if !p.is_empty() {
            return Some(PathBuf::from(p));
        }
    }
    if let Ok(paths) = std::env::var("PATH") {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Emit a native object file (`.o`) by lowering GIR to LLVM IR text and
/// assembling it with the system `llc` (WP-1.6.1 / WP-1.6.2).
pub fn emit_native_object(gir: &Gir, out_path: &Path) -> Result<(), Diagnostics> {
    let ir = emit_llvm(gir)?;
    let ir_path = out_path.with_extension("ll");
    std::fs::write(&ir_path, ir).map_err(io_err)?;
    let result = assemble_object(&ir_path, out_path);
    let _ = std::fs::remove_file(&ir_path);
    result
}

/// Emit a linked native executable (WP-1.6.4). Requires a `main` function in the
/// GIR; quantum programs additionally need a QIR runtime to satisfy the
/// `__quantum__qis__*` symbols at link time.
pub fn emit_native_executable(gir: &Gir, out_path: &Path) -> Result<(), Diagnostics> {
    let obj_path = out_path.with_extension("o");
    emit_native_object(gir, &obj_path)?;
    let cc = match find_tool("CC", "cc").or_else(|| find_tool("CC", "clang")) {
        Some(p) => p,
        None => return Err(tool_not_found("cc")),
    };
    let mut cmd = Command::new(&cc);
    cmd.arg(&obj_path).arg("-o").arg(out_path);
    run_tool("cc", &mut cmd)
}

/// Assemble LLVM IR text into a native object file (WP-1.6.2). Prefers `llc`
/// (the direct `TargetMachine::write_to_file` analogue) and falls back to
/// `clang -c`; both emit for the host triple so the object links natively.
fn assemble_object(ir_path: &Path, out_path: &Path) -> Result<(), Diagnostics> {
    let triple = host_triple();
    if let Some(llc) = find_tool("LLVM_LLC", "llc") {
        let mut cmd = Command::new(&llc);
        if let Some(t) = &triple {
            cmd.arg("-mtriple").arg(t);
        }
        cmd.arg("-filetype=obj").arg("-o").arg(out_path).arg(ir_path);
        return run_tool("llc", &mut cmd);
    }
    if let Some(clang) = find_tool("CC", "clang") {
        let mut cmd = Command::new(&clang);
        cmd.arg("-c").arg("-o").arg(out_path).arg(ir_path);
        return run_tool("clang", &mut cmd);
    }
    Err(tool_not_found("llc"))
}

/// Canonical host target triple, so `llc`/`clang` emit a native object for the
/// current platform (mirrors `Target::initialize_native()` host detection).
fn host_triple() -> Option<String> {
    let cc = find_tool("CC", "clang").or_else(|| find_tool("CC", "cc"))?;
    let out = Command::new(&cc).arg("--print-target-triple").output().ok()?;
    if out.status.success() {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !s.is_empty() {
            return Some(s);
        }
    }
    None
}

/// High-level entry point selecting the emission target (WP-1.6.3).
pub fn emit_native(gir: &Gir, target: EmitTarget, out_path: &Path) -> Result<(), Diagnostics> {
    match target {
        EmitTarget::LlvmIr => {
            let ir = emit_llvm(gir)?;
            std::fs::write(out_path, ir).map_err(io_err)
        }
        EmitTarget::Object => emit_native_object(gir, out_path),
        EmitTarget::Executable => emit_native_executable(gir, out_path),
    }
}

#[cfg(test)]
mod tests;
