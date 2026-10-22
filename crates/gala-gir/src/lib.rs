//! Gala IR (GIR): a typed, effect-annotated, block-structured program.
//!
//! GIR is the only thing backends see. Each function is a list of basic blocks. A block holds
//! straight-line instructions and ends in a terminator. Values live in numbered slots:
//!
//! - classical values (`Int`, `Float`, `Bool`, tuples, `Vec`) hold ordinary data;
//! - qubit values hold a runtime qubit ID. Gates act on the ID in place, so a qubit keeps its
//!   identity through gates and the simulator can record its history.
//!
//! The IR is inspectable: [`Gir::dump`] prints it in a stable text form (`gala build --emit gir`).

use gala_ast::{BinOp, Literal, UnOp};
use gala_diagnostics::{codes, Diagnostic, Diagnostics};
use gala_hir::{Builtin, DefId, HirBlock, HirExpr, HirExprKind, HirPackage, HirPat, HirStmt};
use gala_types::{Effect, TypeInfo};
use std::fmt::Write;

/// A value slot within one function.
pub type Slot = u32;
/// A basic block index within one function.
pub type BlockId = u32;

/// A whole program.
#[derive(Debug, Clone, Default)]
pub struct Gir {
    /// Functions, indexed by `DefId` from the HIR.
    pub funcs: Vec<GirFunc>,
    /// Index of the entry point (`main`), if the program has one.
    pub entry: Option<usize>,
}

/// One function.
#[derive(Debug, Clone)]
pub struct GirFunc {
    /// Source name.
    pub name: String,
    /// Effect on the lattice (declared, or inferred).
    pub effect: Effect,
    /// Slots that receive the arguments, in order.
    pub params: Vec<Slot>,
    /// Number of slots used.
    pub slot_count: u32,
    /// Basic blocks; block 0 is the entry.
    pub blocks: Vec<Block>,
}

/// A basic block.
#[derive(Debug, Clone, Default)]
pub struct Block {
    /// Instructions, executed in order.
    pub insts: Vec<Inst>,
    /// How control leaves the block.
    pub term: Term,
}

/// How a block ends.
#[derive(Debug, Clone, Default)]
pub enum Term {
    /// Not yet set during lowering; never present in a finished program.
    #[default]
    Unreachable,
    Jump(BlockId),
    Branch {
        cond: Slot,
        then_block: BlockId,
        else_block: BlockId,
    },
    Return(Option<Slot>),
}

/// The gates the simulator and optimizer know.
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
    Cx,
    Cz,
    Swap,
    Cp,
}

impl GateKind {
    /// Number of qubit operands.
    pub fn qubits(&self) -> usize {
        match self {
            GateKind::Cx | GateKind::Cz | GateKind::Swap | GateKind::Cp => 2,
            _ => 1,
        }
    }

    /// Whether the gate takes a rotation angle after its qubits.
    pub fn has_angle(&self) -> bool {
        matches!(self, GateKind::Rx | GateKind::Ry | GateKind::Rz | GateKind::Cp)
    }

    /// Name used in the text dump.
    pub fn name(&self) -> &'static str {
        match self {
            GateKind::H => "h",
            GateKind::X => "x",
            GateKind::Y => "y",
            GateKind::Z => "z",
            GateKind::S => "s",
            GateKind::T => "t",
            GateKind::Rx => "rx",
            GateKind::Ry => "ry",
            GateKind::Rz => "rz",
            GateKind::Cx => "cx",
            GateKind::Cz => "cz",
            GateKind::Swap => "swap",
            GateKind::Cp => "cp",
        }
    }
}

/// A compile-time constant.
#[derive(Debug, Clone, PartialEq)]
pub enum Const {
    Unit,
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Complex(f64, f64),
}

/// One instruction.
#[derive(Debug, Clone)]
pub enum Inst {
    /// `dst = const`.
    Const { dst: Slot, value: Const },
    /// `dst = src`.
    Copy { dst: Slot, src: Slot },
    /// `dst = a op b` on classical scalars.
    Bin { dst: Slot, op: BinOp, a: Slot, b: Slot },
    /// `dst = op src`.
    Un { dst: Slot, op: UnOp, src: Slot },
    /// `dst = (items…)`.
    Tuple { dst: Slot, items: Vec<Slot> },
    /// `dst = [items…]`.
    VecLit { dst: Slot, items: Vec<Slot> },
    /// `dst = base[index]` for a classical `Vec`/tuple, or the qubit at a register position.
    Index { dst: Slot, base: Slot, index: Slot },
    /// `dst = len(src)`, for classical sequences.
    Len { dst: Slot, src: Slot },
    /// `dst = item i of tuple src`.
    TupleGet { dst: Slot, src: Slot, i: usize },
    /// Call a function; results go to `dsts` (one per tuple element, or one).
    Call { dsts: Vec<Slot>, func: usize, args: Vec<Slot> },
    /// `dst = a fresh qubit in |0⟩`.
    AllocQubit { dst: Slot },
    /// `dst = a register of `n` fresh qubits`.
    AllocRegister { dst: Slot, n: usize },
    /// Apply a gate in place to the qubits in `targets`. `dsts` receive the same qubit IDs,
    /// one per target, so a `let (a, b) = cx(a, b)` binds fresh slots.
    Gate { gate: GateKind, targets: Vec<Slot>, angle: Option<Slot>, dsts: Vec<Slot> },
    /// Measure a qubit (or every qubit in a register). Writes a `Bool` or `Vec<Bool>`.
    Measure { dst: Slot, target: Slot },
    /// Uncompute and release a qubit. The simulator applies the inverse of the qubit's recorded
    /// history, checks it returned to |0⟩, and frees it. Liftability is checked at compile time.
    Release { target: Slot },
    /// Write the text form of each value to standard output.
    Print { args: Vec<Slot> },
}

impl Gir {
    /// Look up a function by source name.
    pub fn func_named(&self, name: &str) -> Option<usize> {
        self.funcs.iter().position(|f| f.name == name)
    }

    /// Stable text form of the program, for `gala build --emit gir` and golden tests.
    pub fn dump(&self) -> String {
        let mut out = String::new();
        for (i, f) in self.funcs.iter().enumerate() {
            let params: Vec<String> = f.params.iter().map(|s| format!("%{s}")).collect();
            let _ = writeln!(
                out,
                "fn {} #{i} effect={} params=[{}] slots={} {{",
                f.name,
                f.effect,
                params.join(", "),
                f.slot_count
            );
            for (b, block) in f.blocks.iter().enumerate() {
                let _ = writeln!(out, "  bb{b}:");
                for inst in &block.insts {
                    let _ = writeln!(out, "    {}", dump_inst(inst));
                }
                let _ = writeln!(out, "    {}", dump_term(&block.term));
            }
            out.push_str("}\n");
        }
        out
    }
}

fn dump_inst(i: &Inst) -> String {
    match i {
        Inst::Const { dst, value } => format!("%{dst} = const {value:?}"),
        Inst::Copy { dst, src } => format!("%{dst} = copy %{src}"),
        Inst::Bin { dst, op, a, b } => format!("%{dst} = {op:?} %{a}, %{b}"),
        Inst::Un { dst, op, src } => format!("%{dst} = {op:?} %{src}"),
        Inst::Tuple { dst, items } => format!("%{dst} = tuple {}", slots(items)),
        Inst::VecLit { dst, items } => format!("%{dst} = vec {}", slots(items)),
        Inst::Index { dst, base, index } => format!("%{dst} = index %{base}[%{index}]"),
        Inst::Len { dst, src } => format!("%{dst} = len %{src}"),
        Inst::TupleGet { dst, src, i } => format!("%{dst} = %{src}.{i}"),
        Inst::Call { dsts, func, args } => {
            format!("{} = call #{func}({})", slots(dsts), slots(args))
        }
        Inst::AllocQubit { dst } => format!("%{dst} = alloc_qubit"),
        Inst::AllocRegister { dst, n } => format!("%{dst} = alloc_register {n}"),
        Inst::Gate { gate, targets, angle, dsts } => {
            let ang = angle.map(|a| format!(", %{a}")).unwrap_or_default();
            format!("{} = {}({}{})", slots(dsts), gate.name(), slots(targets), ang)
        }
        Inst::Measure { dst, target } => format!("%{dst} = measure %{target}"),
        Inst::Release { target } => format!("release %{target}"),
        Inst::Print { args } => format!("print {}", slots(args)),
    }
}

fn dump_term(t: &Term) -> String {
    match t {
        Term::Unreachable => "unreachable".into(),
        Term::Jump(b) => format!("jump bb{b}"),
        Term::Branch { cond, then_block, else_block } => {
            format!("br %{cond}, bb{then_block}, bb{else_block}")
        }
        Term::Return(Some(s)) => format!("return %{s}"),
        Term::Return(None) => "return".into(),
    }
}

fn slots(xs: &[Slot]) -> String {
    xs.iter().map(|s| format!("%{s}")).collect::<Vec<_>>().join(", ")
}

/// Lower a checked package to GIR.
pub fn lower_package(pkg: &HirPackage, info: &TypeInfo) -> Result<Gir, Diagnostics> {
    let mut gir = Gir::default();
    let mut diags = Diagnostics::new();
    for f in &pkg.fns {
        let effect = info.effects[f.def.0 as usize];
        let name = f.name.as_str().to_string();
        match &f.body {
            Some(body) => {
                let mut fb = FnBuilder::new(pkg, &mut diags);
                let params: Vec<Slot> = f
                    .params
                    .iter()
                    .map(|p| {
                        let s = fb.new_slot();
                        fb.locals.insert(p.local, s);
                        s
                    })
                    .collect();
                let result = fb.block_value(body);
                fb.terminate(Term::Return(result));
                let (blocks, slot_count) = fb.finish();
                gir.funcs.push(GirFunc { name, effect, params, slot_count, blocks });
            }
            None => {
                // Intrinsic declarations have no body; calls to them are lowered as builtins.
                gir.funcs.push(GirFunc {
                    name,
                    effect,
                    params: Vec::new(),
                    slot_count: 0,
                    blocks: Vec::new(),
                });
            }
        }
    }
    gir.entry = pkg.main.map(|DefId(i)| i as usize);
    if diags.has_errors() {
        Err(diags)
    } else {
        Ok(gir)
    }
}

struct FnBuilder<'a> {
    pkg: &'a HirPackage,
    diags: &'a mut Diagnostics,
    blocks: Vec<Block>,
    cur: BlockId,
    next_slot: u32,
    locals: std::collections::HashMap<gala_hir::LocalId, Slot>,
}

impl<'a> FnBuilder<'a> {
    fn new(pkg: &'a HirPackage, diags: &'a mut Diagnostics) -> Self {
        FnBuilder {
            pkg,
            diags,
            blocks: vec![Block::default()],
            cur: 0,
            next_slot: 0,
            locals: Default::default(),
        }
    }

    fn finish(mut self) -> (Vec<Block>, u32) {
        for b in &mut self.blocks {
            if matches!(b.term, Term::Unreachable) {
                b.term = Term::Return(None);
            }
        }
        (self.blocks, self.next_slot)
    }

    fn new_slot(&mut self) -> Slot {
        let s = self.next_slot;
        self.next_slot += 1;
        s
    }

    fn new_block(&mut self) -> BlockId {
        self.blocks.push(Block::default());
        (self.blocks.len() - 1) as BlockId
    }

    fn emit(&mut self, inst: Inst) {
        self.blocks[self.cur as usize].insts.push(inst);
    }

    fn terminate(&mut self, term: Term) {
        let b = &mut self.blocks[self.cur as usize];
        if matches!(b.term, Term::Unreachable) {
            b.term = term;
        }
    }

    fn switch_to(&mut self, b: BlockId) {
        self.cur = b;
    }

    fn constant(&mut self, value: Const) -> Slot {
        let dst = self.new_slot();
        self.emit(Inst::Const { dst, value });
        dst
    }

    fn unsupported(&mut self, what: &str, span: gala_span::Span) {
        self.diags.push(
            Diagnostic::error(
                codes::UNSUPPORTED_FEATURE,
                format!("{what} is not supported by the code generator yet"),
            )
            .with_primary_label(span, "here"),
        );
    }

    /// Lower a block's statements and return the slot of its tail value. Does not terminate the
    /// current block: the caller decides where control goes next.
    fn block_value(&mut self, b: &HirBlock) -> Option<Slot> {
        for s in &b.stmts {
            self.stmt(s);
        }
        b.tail.as_ref().and_then(|t| self.expr(t))
    }

    fn stmt(&mut self, s: &HirStmt) {
        match s {
            HirStmt::Let { pat, init, .. } => {
                if let Some(v) = self.expr(init) {
                    self.bind(pat, v);
                }
            }
            HirStmt::Expr(e) => {
                self.expr(e);
            }
            HirStmt::Return { value, .. } => {
                let v = value.as_ref().and_then(|e| self.expr(e));
                self.terminate(Term::Return(v));
                // Code after a return lands in a fresh, unreachable block.
                let dead = self.new_block();
                self.switch_to(dead);
            }
        }
    }

    fn bind(&mut self, pat: &HirPat, v: Slot) {
        match pat {
            HirPat::Bind { local, .. } => {
                let dst = self.new_slot();
                self.emit(Inst::Copy { dst, src: v });
                self.locals.insert(*local, dst);
            }
            HirPat::Tuple(items, _) => {
                for (i, p) in items.iter().enumerate() {
                    let dst = self.new_slot();
                    self.emit(Inst::TupleGet { dst, src: v, i });
                    self.bind(p, dst);
                }
            }
            HirPat::Wild | HirPat::Lit(_, _) => {}
        }
    }

    /// Lower an expression to the slot holding its value. `None` means it produced no value
    /// (for example after an unsupported construct).
    fn expr(&mut self, e: &HirExpr) -> Option<Slot> {
        match &e.kind {
            HirExprKind::Lit(l) => Some(self.constant(match l {
                Literal::Int(v) => Const::Int(*v),
                Literal::Float(v) => Const::Float(*v),
                Literal::Complex { re, im } => Const::Complex(*re, *im),
                Literal::Bool(v) => Const::Bool(*v),
                Literal::String(s) => Const::Str(s.clone()),
                Literal::Unit => Const::Unit,
            })),
            HirExprKind::Local(l, _) => self.locals.get(l).copied(),
            HirExprKind::Const(idx) => {
                let c = self.pkg.consts.get(*idx)?;
                let value = c.value.clone();
                self.expr(&value)
            }
            HirExprKind::Fn(_) | HirExprKind::Builtin(_) => None,
            HirExprKind::Call { callee, args, generics, .. } => {
                self.call(callee, args, generics, e.span)
            }
            HirExprKind::Unary { op, expr } => {
                let src = self.expr(expr)?;
                let dst = self.new_slot();
                self.emit(Inst::Un { dst, op: *op, src });
                Some(dst)
            }
            HirExprKind::Binary { op, lhs, rhs } => {
                let a = self.expr(lhs)?;
                let b = self.expr(rhs)?;
                let dst = self.new_slot();
                self.emit(Inst::Bin { dst, op: *op, a, b });
                Some(dst)
            }
            HirExprKind::If { cond, then_block, else_expr } => {
                let c = self.expr(cond)?;
                let result = self.new_slot();
                let then_b = self.new_block();
                let else_b = self.new_block();
                let join = self.new_block();
                self.terminate(Term::Branch { cond: c, then_block: then_b, else_block: else_b });
                self.switch_to(then_b);
                if let Some(v) = self.block_value(then_block) {
                    self.emit(Inst::Copy { dst: result, src: v });
                }
                self.terminate_to(join);
                self.switch_to(else_b);
                if let Some(e2) = else_expr {
                    if let Some(v) = self.expr(e2) {
                        self.emit(Inst::Copy { dst: result, src: v });
                    }
                }
                self.terminate_to(join);
                self.switch_to(join);
                Some(result)
            }
            HirExprKind::While { cond, body } => {
                let header = self.new_block();
                let body_b = self.new_block();
                let exit = self.new_block();
                self.terminate_to(header);
                self.switch_to(header);
                let c = self.expr(cond)?;
                self.terminate(Term::Branch { cond: c, then_block: body_b, else_block: exit });
                self.switch_to(body_b);
                self.block_value(body);
                self.terminate_to(header);
                self.switch_to(exit);
                Some(self.constant(Const::Unit))
            }
            HirExprKind::For { pat, iterable, body } => self.for_loop(pat, iterable, body),
            HirExprKind::Block(b) => self.block_value(b),
            HirExprKind::Tuple(items) => {
                let slots: Vec<Slot> = items.iter().filter_map(|i| self.expr(i)).collect();
                let dst = self.new_slot();
                self.emit(Inst::Tuple { dst, items: slots });
                Some(dst)
            }
            HirExprKind::Array(items) => {
                let slots: Vec<Slot> = items.iter().filter_map(|i| self.expr(i)).collect();
                let dst = self.new_slot();
                self.emit(Inst::VecLit { dst, items: slots });
                Some(dst)
            }
            HirExprKind::Index { base, index } => {
                let b = self.expr(base)?;
                let i = self.expr(index)?;
                let dst = self.new_slot();
                self.emit(Inst::Index { dst, base: b, index: i });
                Some(dst)
            }
            HirExprKind::Assign { target, op, value } => {
                let v = self.expr(value)?;
                if let HirExprKind::Local(l, _) = &target.kind {
                    if let Some(slot) = self.locals.get(l).copied() {
                        let new = match op {
                            Some(bop) => {
                                let dst = self.new_slot();
                                self.emit(Inst::Bin { dst, op: *bop, a: slot, b: v });
                                dst
                            }
                            None => v,
                        };
                        self.emit(Inst::Copy { dst: slot, src: new });
                    }
                } else {
                    self.unsupported("assignment to a non-local place", e.span);
                }
                Some(self.constant(Const::Unit))
            }
            HirExprKind::Match { scrutinee, arms } => {
                let s = self.expr(scrutinee)?;
                let result = self.new_slot();
                let join = self.new_block();
                for arm in arms {
                    let body_b = self.new_block();
                    let next = self.new_block();
                    // Test the pattern: a literal compares, a binding or `_` always matches.
                    let matched = match &arm.pat {
                        HirPat::Lit(l, _) => {
                            let lit = self.constant(match l {
                                Literal::Int(v) => Const::Int(*v),
                                Literal::Bool(v) => Const::Bool(*v),
                                _ => Const::Unit,
                            });
                            let dst = self.new_slot();
                            self.emit(Inst::Bin { dst, op: BinOp::Eq, a: s, b: lit });
                            Some(dst)
                        }
                        _ => None,
                    };
                    match matched {
                        Some(c) => self.terminate(Term::Branch {
                            cond: c,
                            then_block: body_b,
                            else_block: next,
                        }),
                        None => self.terminate_to(body_b),
                    }
                    self.switch_to(body_b);
                    self.bind(&arm.pat, s);
                    if let Some(v) = self.expr(&arm.body) {
                        self.emit(Inst::Copy { dst: result, src: v });
                    }
                    self.terminate_to(join);
                    self.switch_to(next);
                }
                // No arm matched: the type checker requires exhaustive literal matches, so
                // this is unreachable in practice; fall through with the unit value.
                self.terminate_to(join);
                self.switch_to(join);
                Some(result)
            }
            HirExprKind::Error => None,
        }
    }

    fn terminate_to(&mut self, b: BlockId) {
        self.terminate(Term::Jump(b));
    }

    fn for_loop(&mut self, pat: &HirPat, iterable: &HirExpr, body: &HirBlock) -> Option<Slot> {
        // `for i in a..b` counts from a to b-1; any other iterable is walked by index.
        let (start, end, seq) = match &iterable.kind {
            HirExprKind::Binary { op: BinOp::Range, lhs, rhs } => {
                (self.expr(lhs)?, self.expr(rhs)?, None)
            }
            _ => {
                let v = self.expr(iterable)?;
                let n = self.new_slot();
                self.emit(Inst::Len { dst: n, src: v });
                let zero = self.constant(Const::Int(0));
                (zero, n, Some(v))
            }
        };
        let idx = self.new_slot();
        self.emit(Inst::Copy { dst: idx, src: start });
        let header = self.new_block();
        let body_b = self.new_block();
        let exit = self.new_block();
        self.terminate_to(header);
        self.switch_to(header);
        let cond = self.new_slot();
        self.emit(Inst::Bin { dst: cond, op: BinOp::Lt, a: idx, b: end });
        self.terminate(Term::Branch { cond, then_block: body_b, else_block: exit });
        self.switch_to(body_b);
        let item = match seq {
            Some(s) => {
                let dst = self.new_slot();
                self.emit(Inst::Index { dst, base: s, index: idx });
                dst
            }
            None => idx,
        };
        self.bind(pat, item);
        self.block_value(body);
        let one = self.constant(Const::Int(1));
        self.emit(Inst::Bin { dst: idx, op: BinOp::Add, a: idx, b: one });
        self.terminate_to(header);
        self.switch_to(exit);
        Some(self.constant(Const::Unit))
    }

    fn call(
        &mut self,
        callee: &HirExpr,
        args: &[HirExpr],
        generics: &[gala_ast::TypeArg],
        span: gala_span::Span,
    ) -> Option<Slot> {
        match &callee.kind {
            HirExprKind::Builtin(b) => self.builtin(*b, args, generics, span),
            HirExprKind::Fn(def) => {
                let slots: Vec<Slot> = args.iter().filter_map(|a| self.expr(a)).collect();
                let dst = self.new_slot();
                self.emit(Inst::Call { dsts: vec![dst], func: def.0 as usize, args: slots });
                Some(dst)
            }
            _ => {
                self.unsupported("this call", span);
                None
            }
        }
    }

    fn builtin(
        &mut self,
        b: Builtin,
        args: &[HirExpr],
        generics: &[gala_ast::TypeArg],
        span: gala_span::Span,
    ) -> Option<Slot> {
        if b.is_gate() {
            let mut targets = Vec::new();
            let mut angle = None;
            for (i, a) in args.iter().enumerate() {
                let s = self.expr(a)?;
                if i < b.qubit_arity() {
                    targets.push(s);
                } else {
                    angle = Some(s);
                }
            }
            let gate = gate_of(b);
            let dsts: Vec<Slot> = targets.iter().map(|_| self.new_slot()).collect();
            self.emit(Inst::Gate { gate, targets, angle, dsts: dsts.clone() });
            return Some(if dsts.len() == 1 {
                dsts[0]
            } else {
                let dst = self.new_slot();
                self.emit(Inst::Tuple { dst, items: dsts });
                dst
            });
        }
        match b {
            Builtin::Qubit => {
                let dst = self.new_slot();
                self.emit(Inst::AllocQubit { dst });
                Some(dst)
            }
            Builtin::Qubits => {
                let n = match generics.first() {
                    Some(gala_ast::TypeArg::Const(c)) => {
                        gala_hir::const_int(c).unwrap_or(0).max(0) as usize
                    }
                    _ => 0,
                };
                let dst = self.new_slot();
                self.emit(Inst::AllocRegister { dst, n });
                Some(dst)
            }
            Builtin::Measure => {
                let t = self.expr(&args[0])?;
                let dst = self.new_slot();
                self.emit(Inst::Measure { dst, target: t });
                Some(dst)
            }
            Builtin::Drop => {
                let t = self.expr(&args[0])?;
                self.emit(Inst::Release { target: t });
                Some(self.constant(Const::Unit))
            }
            Builtin::Print => {
                let slots: Vec<Slot> = args.iter().filter_map(|a| self.expr(a)).collect();
                self.emit(Inst::Print { args: slots });
                Some(self.constant(Const::Unit))
            }
            Builtin::Clone | Builtin::Grad => {
                self.unsupported("this builtin", span);
                None
            }
            _ => None,
        }
    }
}

fn gate_of(b: Builtin) -> GateKind {
    match b {
        Builtin::H => GateKind::H,
        Builtin::X => GateKind::X,
        Builtin::Y => GateKind::Y,
        Builtin::Z => GateKind::Z,
        Builtin::S => GateKind::S,
        Builtin::T => GateKind::T,
        Builtin::Rx => GateKind::Rx,
        Builtin::Ry => GateKind::Ry,
        Builtin::Rz => GateKind::Rz,
        Builtin::Cx => GateKind::Cx,
        Builtin::Cz => GateKind::Cz,
        Builtin::Swap => GateKind::Swap,
        _ => GateKind::Cp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gala_hir::lower_file;
    use gala_parser::parse_source;
    use gala_span::FileId;

    fn lower(src: &str) -> Result<Gir, Diagnostics> {
        let out = parse_source(FileId(0), src);
        assert!(!out.diagnostics.has_errors(), "parse: {:?}", out.diagnostics);
        let (pkg, _) = lower_file(FileId(0), out.items);
        let (info, _) = gala_types::check_package(&pkg);
        lower_package(&pkg, &info)
    }

    #[test]
    fn classical_function_lowers_to_return() {
        let gir = lower("fn main() -> Int { 1 + 2 }").unwrap();
        assert_eq!(gir.entry, Some(0));
        let f = &gir.funcs[0];
        assert!(matches!(f.blocks[0].term, Term::Return(Some(_))));
    }

    #[test]
    fn bell_lowers_to_gates_and_measure() {
        let gir = lower("fn main() -> Measured<Bool> prob {\n let a = qubit()\n let b = qubit()\n h(a)\n let (a, b) = cx(a, b)\n measure(a)\n}").unwrap();
        let dump = gir.dump();
        assert!(dump.contains("alloc_qubit"), "{dump}");
        assert!(dump.contains("h("), "{dump}");
        assert!(dump.contains("cx("), "{dump}");
        assert!(dump.contains("measure"), "{dump}");
    }

    #[test]
    fn if_creates_branch_and_join() {
        let gir = lower("fn f(c: Bool) -> Int { if c { 1 } else { 2 } }").unwrap();
        assert!(gir.funcs[0].blocks.len() >= 4);
        assert!(gir.funcs[0].blocks.iter().any(|b| matches!(b.term, Term::Branch { .. })));
    }

    #[test]
    fn for_range_creates_loop_blocks() {
        let gir = lower("fn f() -> Int { let mut s = 0\n for i in 0..3 { s += i }\n s }").unwrap();
        // A loop has a header that branches back to itself through the body.
        let has_back_edge = gir.funcs[0]
            .blocks
            .iter()
            .enumerate()
            .any(|(i, b)| matches!(b.term, Term::Jump(t) if (t as usize) < i));
        assert!(has_back_edge);
    }

    #[test]
    fn dump_is_deterministic() {
        let src = "fn main() -> Int { let a = 1\n a + 2 }";
        assert_eq!(lower(src).unwrap().dump(), lower(src).unwrap().dump());
    }
}

/// Structural invariants of a lowered program. Returns one message per violation.
///
/// Checked: every jump and branch targets an existing block; every slot is below `slot_count`;
/// every block is terminated; the entry point names a function; and a gate's outputs are fresh
/// slots (each defined once), so the in-place rewriting in later passes stays sound.
pub fn verify(gir: &Gir) -> Vec<String> {
    let mut problems = Vec::new();
    if let Some(e) = gir.entry {
        if e >= gir.funcs.len() {
            problems.push(format!("entry {e} is not a function"));
        }
    }
    for f in &gir.funcs {
        if f.blocks.is_empty() {
            continue;
        }
        let nblocks = f.blocks.len() as u32;
        let slot_ok = |s: Slot, what: &str, problems: &mut Vec<String>| {
            if s >= f.slot_count {
                problems.push(format!(
                    "{}: {what} uses slot %{s} beyond slot_count {}",
                    f.name, f.slot_count
                ));
            }
        };
        for p in &f.params {
            slot_ok(*p, "parameter", &mut problems);
        }
        let mut defined: std::collections::HashMap<Slot, usize> = std::collections::HashMap::new();
        for (bi, b) in f.blocks.iter().enumerate() {
            for inst in &b.insts {
                let dsts: Vec<Slot> = match inst {
                    Inst::Const { dst, .. }
                    | Inst::Copy { dst, .. }
                    | Inst::Bin { dst, .. }
                    | Inst::Un { dst, .. }
                    | Inst::Tuple { dst, .. }
                    | Inst::VecLit { dst, .. }
                    | Inst::Index { dst, .. }
                    | Inst::Len { dst, .. }
                    | Inst::TupleGet { dst, .. }
                    | Inst::AllocQubit { dst }
                    | Inst::AllocRegister { dst, .. }
                    | Inst::Measure { dst, .. } => vec![*dst],
                    Inst::Call { dsts, .. } | Inst::Gate { dsts, .. } => dsts.clone(),
                    Inst::Release { .. } | Inst::Print { .. } => Vec::new(),
                };
                for d in dsts {
                    slot_ok(d, "definition", &mut problems);
                    *defined.entry(d).or_insert(0) += 1;
                }
            }
            match &b.term {
                Term::Unreachable => {
                    problems.push(format!("{}: block bb{bi} has no terminator", f.name))
                }
                Term::Jump(t) if *t >= nblocks => {
                    problems.push(format!("{}: bb{bi} jumps to missing block bb{t}", f.name))
                }
                Term::Branch { cond, then_block, else_block } => {
                    slot_ok(*cond, "branch condition", &mut problems);
                    for t in [then_block, else_block] {
                        if *t >= nblocks {
                            problems.push(format!(
                                "{}: bb{bi} branches to missing block bb{t}",
                                f.name
                            ));
                        }
                    }
                }
                Term::Return(Some(s)) => slot_ok(*s, "return", &mut problems),
                _ => {}
            }
        }
        // Gate outputs are fresh; a slot defined by a gate must not be defined anywhere else.
        for b in &f.blocks {
            for inst in &b.insts {
                if let Inst::Gate { dsts, .. } = inst {
                    for d in dsts {
                        if defined.get(d).copied().unwrap_or(0) > 1 {
                            problems.push(format!(
                                "{}: gate output %{d} is defined more than once",
                                f.name
                            ));
                        }
                    }
                }
            }
        }
    }
    problems
}

#[cfg(test)]
mod verify_tests {
    use super::*;

    #[test]
    fn lowered_programs_satisfy_the_invariants() {
        let src = "fn main() -> Measured<Bool> prob {\n let a = qubit()\n let b = qubit()\n h(a)\n let (a, b) = cx(a, b)\n drop(b)\n measure(a)\n}\n";
        let out = gala_parser::parse_source(gala_span::FileId(0), src);
        let (pkg, _) = gala_hir::lower_file(gala_span::FileId(0), out.items);
        let (info, _) = gala_types::check_package(&pkg);
        let gir = lower_package(&pkg, &info).unwrap();
        assert!(verify(&gir).is_empty(), "{:?}", verify(&gir));
    }

    #[test]
    fn a_broken_jump_is_reported() {
        let mut gir = Gir::default();
        gir.funcs.push(GirFunc {
            name: "f".into(),
            effect: Effect::Pure,
            params: vec![],
            slot_count: 0,
            blocks: vec![Block { insts: vec![], term: Term::Jump(9) }],
        });
        assert!(verify(&gir).iter().any(|m| m.contains("missing block bb9")));
    }
}
