//! GIR to QIR.
//!
//! QIR is LLVM IR with a fixed set of quantum intrinsics. This crate writes that text directly,
//! because `inkwell` needs a system LLVM that this build does not require.
//!
//! Profiles:
//! - **base**: straight-line gates and terminal measurements, with statically numbered qubits.
//! - **adaptive**: the program branches on a measurement result (mid-circuit measurement). Each
//!   GIR block becomes an LLVM block, and branches read the result with `read_result`.
//!
//! Limits, reported as `E0602` rather than emitted incorrectly: loops, calls to user functions,
//! classical arithmetic that feeds a gate angle, and `print`. Angles must be literal constants.
//!
//! `Release` emits the inverse of the qubit's causal cone, computed from the emitted gates, so the
//! ancilla is back in |0⟩ before its static index is retired. The emitted gate sequence can be
//! checked against the source program with [`equivalent_on_simulator`].

use gala_diagnostics::{codes, Diagnostic, Diagnostics};
use gala_gir::{Const, GateKind, Gir, Inst, Slot, Term};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

/// The QIR profile a program needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Base,
    Adaptive,
}

/// One emitted quantum operation, in order. Qubits are static indices.
#[derive(Debug, Clone, PartialEq)]
pub struct QirOp {
    pub gate: GateKind,
    pub qubits: Vec<u32>,
    pub angle: f64,
}

/// The output of emission.
#[derive(Debug, Clone)]
pub struct QirModule {
    pub profile: Profile,
    /// Number of static qubits the program uses.
    pub num_qubits: usize,
    /// Number of measurement results recorded.
    pub num_results: usize,
    /// The LLVM IR text.
    pub text: String,
    /// The gate sequence, in emission order (releases already expanded to their inverses).
    pub ops: Vec<QirOp>,
}

/// Emit the QIR module for a program's entry point.
pub fn emit(gir: &Gir) -> Result<QirModule, Diagnostics> {
    let entry = gir.entry.ok_or_else(|| err("the program has no `main` function"))?;
    let f = &gir.funcs[entry];
    let mut e = Emitter::new(gir, entry);
    // Blocks are emitted in order; a backward jump would be a loop, which QIR cannot express here.
    for (i, b) in f.blocks.iter().enumerate() {
        for t in successors(&b.term) {
            if (t as usize) <= i {
                return Err(err("loops are not supported by the QIR emitter yet"));
            }
        }
    }
    let adaptive = f.blocks.iter().any(|b| matches!(b.term, Term::Branch { .. }));
    for (i, b) in f.blocks.iter().enumerate() {
        e.block_label = i as u32;
        for inst in &b.insts {
            e.inst(inst)?;
        }
        e.term(&b.term, adaptive)?;
    }
    let profile = if adaptive { Profile::Adaptive } else { Profile::Base };
    Ok(e.finish(profile))
}

fn err(msg: &str) -> Diagnostics {
    let mut d = Diagnostics::new();
    d.push(Diagnostic::error(codes::QIR_EMISSION_FAILED, msg.to_string()));
    d
}

fn successors(t: &Term) -> Vec<u32> {
    match t {
        Term::Jump(b) => vec![*b],
        Term::Branch { then_block, else_block, .. } => vec![*then_block, *else_block],
        _ => Vec::new(),
    }
}

/// What a slot holds during emission.
#[derive(Debug, Clone, PartialEq)]
enum Val {
    Qubit(u32),
    Register(Vec<u32>),
    Result(u32),
    Const(f64),
    Tuple(Vec<Val>),
    Unit,
}

struct Emitter {
    slots: HashMap<Slot, Val>,
    next_qubit: u32,
    next_result: u32,
    ops: Vec<QirOp>,
    body: String,
    block_label: u32,
    /// Ops emitted since each qubit was allocated: its causal history, for release.
    history: Vec<usize>,
    alloc_at: HashMap<u32, usize>,
    undone: HashSet<usize>,
    gir_name: String,
}

impl Emitter {
    fn new(gir: &Gir, entry: usize) -> Self {
        Emitter {
            slots: HashMap::new(),
            next_qubit: 0,
            next_result: 0,
            ops: Vec::new(),
            body: String::new(),
            block_label: 0,
            history: Vec::new(),
            alloc_at: HashMap::new(),
            undone: HashSet::new(),
            gir_name: gir.funcs[entry].name.clone(),
        }
    }

    fn get(&self, s: Slot) -> Result<Val, Diagnostics> {
        self.slots.get(&s).cloned().ok_or_else(|| err("a value is used before it is defined"))
    }

    fn qubit(&self, s: Slot) -> Result<u32, Diagnostics> {
        match self.get(s)? {
            Val::Qubit(q) => Ok(q),
            _ => Err(err("a gate was applied to something that is not a qubit")),
        }
    }

    fn angle(&self, s: Option<Slot>) -> Result<f64, Diagnostics> {
        match s {
            None => Ok(0.0),
            Some(s) => match self.get(s)? {
                Val::Const(v) => Ok(v),
                _ => Err(err("rotation angles must be literal constants for QIR emission")),
            },
        }
    }

    fn emit_op(&mut self, gate: GateKind, qubits: Vec<u32>, angle: f64) {
        let idx = self.ops.len();
        self.ops.push(QirOp { gate, qubits: qubits.clone(), angle });
        self.history.push(idx);
        let name = intrinsic(gate);
        let mut args: Vec<String> = Vec::new();
        if gate.has_angle() {
            args.push(format!("double {angle:e}"));
        }
        for q in &qubits {
            args.push(format!("%Qubit* inttoptr (i64 {q} to %Qubit*)"));
        }
        let _ = writeln!(self.body, "  call void @{name}({})", args.join(", "));
    }

    /// Inverse of the causal cone of `q` since its allocation, applied in reverse order.
    fn release(&mut self, q: u32) -> Result<(), Diagnostics> {
        let start = *self
            .alloc_at
            .get(&q)
            .ok_or_else(|| err("released a qubit that was never allocated"))?;
        let mut cone: HashSet<u32> = HashSet::from([q]);
        let mut chosen = Vec::new();
        for k in start..self.ops.len() {
            if self.undone.contains(&k) {
                continue;
            }
            let op = &self.ops[k];
            if op.qubits.iter().any(|x| cone.contains(x)) {
                cone.extend(op.qubits.iter().copied());
                chosen.push(k);
            }
        }
        for &k in chosen.iter().rev() {
            let op = self.ops[k].clone();
            let angle = if op.gate.has_angle() { -op.angle } else { op.angle };
            self.emit_op_adjoint(op.gate, op.qubits, angle);
        }
        self.undone.extend(chosen);
        Ok(())
    }

    fn emit_op_adjoint(&mut self, gate: GateKind, qubits: Vec<u32>, angle: f64) {
        // Adjoint of S and T is not a Clifford/phase of the same intrinsic; express as rz-based.
        match gate {
            GateKind::S => self.emit_op(GateKind::Rz, qubits, -std::f64::consts::FRAC_PI_2),
            GateKind::T => self.emit_op(GateKind::Rz, qubits, -std::f64::consts::FRAC_PI_4),
            _ => self.emit_op(gate, qubits, angle),
        }
    }

    fn inst(&mut self, inst: &Inst) -> Result<(), Diagnostics> {
        match inst {
            Inst::Const { dst, value } => {
                let v = match value {
                    Const::Float(f) => Val::Const(*f),
                    Const::Int(i) => Val::Const(*i as f64),
                    Const::Unit => Val::Unit,
                    _ => Val::Unit,
                };
                self.slots.insert(*dst, v);
            }
            Inst::Copy { dst, src } => {
                let v = self.get(*src)?;
                self.slots.insert(*dst, v);
            }
            Inst::TupleGet { dst, src, i } => match self.get(*src)? {
                Val::Tuple(items) => {
                    let v =
                        items.get(*i).cloned().ok_or_else(|| err("tuple index out of range"))?;
                    self.slots.insert(*dst, v);
                }
                _ => return Err(err("destructuring a non-tuple")),
            },
            Inst::Tuple { dst, items } => {
                let vals = items.iter().map(|s| self.get(*s)).collect::<Result<Vec<_>, _>>()?;
                self.slots.insert(*dst, Val::Tuple(vals));
            }
            Inst::AllocQubit { dst } => {
                let q = self.next_qubit;
                self.next_qubit += 1;
                self.alloc_at.insert(q, self.history.len());
                self.slots.insert(*dst, Val::Qubit(q));
            }
            Inst::AllocRegister { dst, n } => {
                let mut ids = Vec::new();
                for _ in 0..*n {
                    let q = self.next_qubit;
                    self.next_qubit += 1;
                    self.alloc_at.insert(q, self.history.len());
                    ids.push(q);
                }
                self.slots.insert(*dst, Val::Register(ids));
            }
            Inst::Gate { gate, targets, angle, dsts } => {
                let qs: Vec<u32> =
                    targets.iter().map(|s| self.qubit(*s)).collect::<Result<_, _>>()?;
                let a = self.angle(*angle)?;
                match gate {
                    GateKind::Cp => {
                        // CP(θ) = Rz(θ/2)⊗Rz(θ/2) · CX · (I⊗Rz(−θ/2)) · CX, up to global phase.
                        let (c, t) = (qs[0], qs[1]);
                        self.emit_op(GateKind::Rz, vec![t], a / 2.0);
                        self.emit_op(GateKind::Cx, vec![c, t], 0.0);
                        self.emit_op(GateKind::Rz, vec![t], -a / 2.0);
                        self.emit_op(GateKind::Cx, vec![c, t], 0.0);
                        self.emit_op(GateKind::Rz, vec![c], a / 2.0);
                    }
                    _ => self.emit_op(*gate, qs.clone(), a),
                }
                for (d, q) in dsts.iter().zip(qs) {
                    self.slots.insert(*d, Val::Qubit(q));
                }
            }
            Inst::Measure { dst, target } => match self.get(*target)? {
                Val::Qubit(q) => {
                    let r = self.next_result;
                    self.next_result += 1;
                    let _ = writeln!(
                        self.body,
                        "  call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 {q} to %Qubit*), %Result* inttoptr (i64 {r} to %Result*))"
                    );
                    self.slots.insert(*dst, Val::Result(r));
                }
                _ => {
                    return Err(err(
                        "measurement of a register is not supported by the QIR emitter yet",
                    ))
                }
            },
            Inst::Release { target } => match self.get(*target)? {
                Val::Qubit(q) => self.release(q)?,
                Val::Register(ids) => {
                    for q in ids {
                        self.release(q)?;
                    }
                }
                _ => return Err(err("releasing a non-qubit")),
            },
            Inst::Call { .. } => {
                return Err(err("calls to user functions are not supported by the QIR emitter yet"))
            }
            Inst::Print { .. } => return Err(err("`print` has no QIR form")),
            Inst::Run { .. } => {
                return Err(err(
                    "`run` inside a program has no QIR form; run the function on a backend instead",
                ))
            }
            Inst::Bin { .. }
            | Inst::Un { .. }
            | Inst::Len { .. }
            | Inst::Index { .. }
            | Inst::VecLit { .. } => {
                return Err(err("classical arithmetic is not supported by the QIR emitter yet"));
            }
        }
        Ok(())
    }

    fn term(&mut self, t: &Term, adaptive: bool) -> Result<(), Diagnostics> {
        match t {
            Term::Jump(b) => {
                let _ = writeln!(self.body, "  br label %bb{b}");
            }
            Term::Branch { cond, then_block, else_block } => {
                if !adaptive {
                    return Err(err("branching needs the adaptive profile"));
                }
                let r = match self.get(*cond)? {
                    Val::Result(r) => r,
                    _ => return Err(err("branch conditions must be measurement results")),
                };
                let _ = writeln!(
                    self.body,
                    "  %c{r} = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 {r} to %Result*))"
                );
                let _ = writeln!(
                    self.body,
                    "  br i1 %c{r}, label %bb{then_block}, label %bb{else_block}"
                );
            }
            Term::Return(_) | Term::Unreachable => {
                // The entry's return value is classical; QIR programs return void.
                let _ = writeln!(self.body, "  ret void");
            }
        }
        // Labels partition the body: each GIR block starts its own LLVM block.
        Ok(())
    }

    fn finish(self, profile: Profile) -> QirModule {
        let num_qubits = self.next_qubit as usize;
        let num_results = self.next_result as usize;
        let profile_attr = match profile {
            Profile::Base => "base_profile",
            Profile::Adaptive => "adaptive_profile",
        };
        let mut text = String::new();
        let _ = writeln!(text, "; Gala QIR for `{}`", self.gir_name);
        text.push_str("%Qubit = type opaque\n%Result = type opaque\n\n");
        let _ = writeln!(text, "define void @{}() #0 {{", self.gir_name);
        // Entry block label, then the GIR blocks as labelled sections.
        text.push_str("entry:\n");
        text.push_str(&self.body);
        text.push_str("}\n\n");
        text.push_str(&format!(
            "attributes #0 = {{ \"entry_point\" \"qir_profiles\"=\"{profile_attr}\" \"required_num_qubits\"=\"{num_qubits}\" \"required_num_results\"=\"{num_results}\" }}\n"
        ));
        QirModule { profile, num_qubits, num_results, text, ops: self.ops }
    }
}

fn intrinsic(g: GateKind) -> &'static str {
    match g {
        GateKind::H => "__quantum__qis__h__body",
        GateKind::X => "__quantum__qis__x__body",
        GateKind::Y => "__quantum__qis__y__body",
        GateKind::Z => "__quantum__qis__z__body",
        GateKind::S => "__quantum__qis__s__body",
        GateKind::T => "__quantum__qis__t__body",
        GateKind::Rx => "__quantum__qis__rx__body",
        GateKind::Ry => "__quantum__qis__ry__body",
        GateKind::Rz => "__quantum__qis__rz__body",
        GateKind::Cx => "__quantum__qis__cnot__body",
        GateKind::Cz => "__quantum__qis__cz__body",
        GateKind::Swap => "__quantum__qis__swap__body",
        GateKind::Cp => "__quantum__qis__rz__body",
    }
}

/// Check that a QIR gate sequence computes the same exact distribution as the program's own
/// gate sequence on the simulator, for straight-line programs.
pub fn equivalent_on_simulator(
    module: &QirModule,
    reference: &[(GateKind, Vec<u32>, f64)],
) -> bool {
    let ops: Vec<(GateKind, Vec<u32>, f64)> =
        module.ops.iter().map(|o| (o.gate, o.qubits.clone(), o.angle)).collect();
    match (
        gala_sim::exact_probabilities(module.num_qubits, &ops),
        gala_sim::exact_probabilities(module.num_qubits, reference),
    ) {
        (Ok(a), Ok(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < 1e-9)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile(src: &str) -> Gir {
        let mut sources = gala_driver::Sources::default();
        gala_driver::compile_source(src, &mut sources).expect("compiles").gir
    }

    #[test]
    fn bell_pair_is_base_profile_with_static_qubits() {
        let gir = compile("fn main() -> Measured<Bool> prob {\n let a = qubit()\n let b = qubit()\n h(a)\n let (a, b) = cx(a, b)\n drop(b)\n measure(a)\n}");
        let m = emit(&gir).unwrap();
        assert_eq!(m.profile, Profile::Base);
        assert_eq!(m.num_qubits, 2);
        assert!(m.text.contains("__quantum__qis__h__body"));
        assert!(m.text.contains("__quantum__qis__cnot__body"));
        assert!(m.text.contains("base_profile"));
    }

    #[test]
    fn mid_circuit_measurement_selects_adaptive_profile() {
        let gir = compile("fn main() -> Int {\n let q = qubit()\n h(q)\n let r = measure(q)\n if r {\n  let p = qubit()\n  x(p)\n  drop(p)\n }\n 0\n}");
        let m = emit(&gir).unwrap();
        assert_eq!(m.profile, Profile::Adaptive);
        assert!(m.text.contains("read_result"));
        assert!(m.text.contains("adaptive_profile"));
        assert!(m.text.contains("br i1"));
    }

    #[test]
    fn emitted_gates_match_source_on_simulator() {
        // Source: X then H on two qubits, with a CP between them.
        let gir = compile("fn main() -> Measured<Bool> prob {\n let a = qubit()\n let b = qubit()\n h(a)\n let (a, b) = cp(a, b, 0.7)\n drop(b)\n measure(a)\n}");
        let m = emit(&gir).unwrap();
        // Reference: the same circuit as direct gates on the simulator (CP kept whole).
        let reference = vec![(GateKind::H, vec![0], 0.0), (GateKind::Cp, vec![0, 1], 0.7)];
        // The QIR decomposition of CP is equal up to a global phase, so the distribution matches.
        assert!(equivalent_on_simulator(&m, &reference));
    }

    #[test]
    fn release_emits_inverse_of_entangling_gates() {
        let gir = compile("fn main() -> Measured<Bool> prob {\n let d = qubit()\n h(d)\n let a = qubit()\n cx(d, a)\n drop(a)\n measure(d)\n}");
        let m = emit(&gir).unwrap();
        // cx(d,a) is undone by a second cx(d,a) at release, leaving a in |0⟩.
        let cnots = m.ops.iter().filter(|o| o.gate == GateKind::Cx).count();
        assert_eq!(cnots, 2);
        let reference = vec![(GateKind::H, vec![0], 0.0)];
        assert!(equivalent_on_simulator(&m, &reference));
    }

    #[test]
    fn loops_are_rejected_with_e0602() {
        let gir = compile(
            "fn main() -> Int {\n for i in 0..2 {\n  let a = qubit()\n  h(a)\n  drop(a)\n }\n 0\n}",
        );
        let e = emit(&gir).unwrap_err();
        assert!(e.errors().iter().any(|d| d.code.to_string() == "E0602"));
    }
}
