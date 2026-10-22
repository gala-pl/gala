//! Simulator backend: runs GIR on an exact state-vector simulator.
//!
//! The state holds only the live qubits. A qubit is allocated by extending the state with a |0⟩
//! factor, and removed by measurement or release, which leaves the remaining qubits in a
//! normalized state. Releasing a qubit first applies the inverse of its recorded gate history
//! (its causal cone), then checks that it is back in |0⟩. Any shot with a different outcome is a
//! runtime error (`E0701`), so an incorrect uncomputation cannot pass silently.
//!
//! The default backend is this pure-Rust engine. It is deterministic under a fixed seed.

use gala_ast::{BinOp, UnOp};
use gala_diagnostics::{codes, Diagnostic, DiagnosticCode};
use gala_gir::{Const, GateKind, Gir, Inst, Term};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;

/// Largest number of simultaneously live qubits the simulator will allocate.
pub const MAX_QUBITS: usize = 24;
/// Instruction budget per shot; guards against non-terminating programs.
const STEP_LIMIT: u64 = 50_000_000;
/// Tolerance for "is |0⟩" checks at release.
const ZERO_TOL: f64 = 1e-9;

/// Simulation configuration.
#[derive(Debug, Clone)]
pub struct SimConfig {
    /// Number of shots when the entry point is probabilistic.
    pub shots: u64,
    /// Seed for measurement sampling and noise.
    pub seed: u64,
    /// Depolarizing error probability applied to every target qubit after each gate. Zero is the
    /// noiseless simulator. The error is X, Y, or Z with probability `p/3` each.
    pub depolarizing: f64,
}

impl Default for SimConfig {
    fn default() -> Self {
        SimConfig { shots: 1024, seed: 0, depolarizing: 0.0 }
    }
}

/// A runtime error with a code.
#[derive(Debug, Clone, PartialEq)]
pub struct SimError {
    /// Diagnostic code, for example `E0701`.
    pub code: DiagnosticCode,
    /// Explanation.
    pub message: String,
}

impl SimError {
    fn new(code: DiagnosticCode, message: impl Into<String>) -> Self {
        SimError { code, message: message.into() }
    }

    /// Convert to a diagnostic (without a source span).
    pub fn to_diagnostic(&self) -> Diagnostic {
        Diagnostic::error(self.code, self.message.clone())
    }
}

impl fmt::Display for SimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

/// A runtime value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Complex(f64, f64),
    /// A live qubit, by its runtime ID.
    Qubit(u32),
    /// A register: an ordered list of live qubit IDs.
    Register(Vec<u32>),
    Tuple(Vec<Value>),
    Vec(Vec<Value>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Unit => write!(f, "()"),
            Value::Int(v) => write!(f, "{v}"),
            Value::Float(v) => write!(f, "{v}"),
            Value::Bool(v) => write!(f, "{v}"),
            Value::Str(s) => write!(f, "{s}"),
            Value::Complex(re, im) => write!(f, "{re}+{im}i"),
            Value::Qubit(id) => write!(f, "<qubit {id}>"),
            Value::Register(ids) => write!(f, "<register of {}>", ids.len()),
            Value::Tuple(items) => {
                let parts: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                write!(f, "({})", parts.join(", "))
            }
            Value::Vec(items) => {
                let parts: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", parts.join(", "))
            }
        }
    }
}

/// Result of running the entry point.
#[derive(Debug, Clone, Default)]
pub struct RunReport {
    /// Lines printed by `print` during the first shot.
    pub output: Vec<String>,
    /// Number of shots executed.
    pub shots: u64,
    /// Histogram of the entry point's return value across shots (keyed by its text form).
    pub histogram: BTreeMap<String, u64>,
    /// Entry point's return value in the first shot.
    pub first_result: Option<Value>,
    /// Largest number of qubits live at once in any shot.
    pub peak_qubits: usize,
}

/// Run the program's entry point.
///
/// Probabilistic entry points (effect `prob`) run `config.shots` times. Others run once.
pub fn run_entry(gir: &Gir, config: &SimConfig) -> Result<RunReport, SimError> {
    let entry = gir
        .entry
        .ok_or_else(|| SimError::new(codes::RUNTIME_ERROR, "the program has no `main` function"))?;
    run_function(gir, entry, Vec::new(), config, &ShiftPlan::new())
}

/// Angle offsets for individual gate instructions, keyed by `(function, block, instruction)`.
/// Used by parameter-shift differentiation: each entry adds its offset to that one gate's angle.
pub type ShiftPlan = HashMap<(usize, u32, usize), f64>;

/// Run any function with arguments, applying an optional per-gate angle shift plan.
///
/// A probabilistic function (effect `prob`) runs `config.shots` times; others run once.
pub fn run_function(
    gir: &Gir,
    func: usize,
    args: Vec<Value>,
    config: &SimConfig,
    shifts: &ShiftPlan,
) -> Result<RunReport, SimError> {
    let effect = gir.funcs[func].effect;
    let shots = if effect == gala_types::Effect::Prob { config.shots.max(1) } else { 1 };
    let mut report = RunReport::default();
    let mut rng = StdRng::seed_from_u64(config.seed);
    for shot in 0..shots {
        let mut machine = Machine::new(gir, &mut rng, shifts, config.depolarizing);
        let value = machine.call(func, args.clone())?;
        if shot == 0 {
            report.output = machine.output.clone();
            report.first_result = Some(value.clone());
            report.peak_qubits = machine.peak;
        }
        *report.histogram.entry(value.to_string()).or_insert(0) += 1;
        report.shots += 1;
    }
    Ok(report)
}

/// Complex number.
#[derive(Debug, Clone, Copy, PartialEq)]
struct C {
    re: f64,
    im: f64,
}

impl C {
    const ZERO: C = C { re: 0.0, im: 0.0 };
    const ONE: C = C { re: 1.0, im: 0.0 };

    fn new(re: f64, im: f64) -> C {
        C { re, im }
    }
    fn mul(self, o: C) -> C {
        C::new(self.re * o.re - self.im * o.im, self.re * o.im + self.im * o.re)
    }
    fn add(self, o: C) -> C {
        C::new(self.re + o.re, self.im + o.im)
    }
    fn scale(self, k: f64) -> C {
        C::new(self.re * k, self.im * k)
    }
    fn norm2(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
    fn exp_i(theta: f64) -> C {
        C::new(theta.cos(), theta.sin())
    }
}

/// A 2×2 unitary.
type M2 = [[C; 2]; 2];

/// The single-qubit matrix of a gate, or its adjoint when `adjoint` is set.
fn matrix1(gate: GateKind, angle: f64, adjoint: bool) -> M2 {
    let r = std::f64::consts::FRAC_1_SQRT_2;
    let sign = if adjoint { -1.0 } else { 1.0 };
    match gate {
        GateKind::H => [[C::new(r, 0.0), C::new(r, 0.0)], [C::new(r, 0.0), C::new(-r, 0.0)]],
        GateKind::X => [[C::ZERO, C::ONE], [C::ONE, C::ZERO]],
        GateKind::Y => [[C::ZERO, C::new(0.0, -1.0)], [C::new(0.0, 1.0), C::ZERO]],
        GateKind::Z => [[C::ONE, C::ZERO], [C::ZERO, C::new(-1.0, 0.0)]],
        GateKind::S => [[C::ONE, C::ZERO], [C::ZERO, C::new(0.0, sign)]],
        GateKind::T => [[C::ONE, C::ZERO], [C::ZERO, C::exp_i(sign * std::f64::consts::FRAC_PI_4)]],
        GateKind::Rx => {
            let t = sign * angle / 2.0;
            [
                [C::new(t.cos(), 0.0), C::new(0.0, -t.sin())],
                [C::new(0.0, -t.sin()), C::new(t.cos(), 0.0)],
            ]
        }
        GateKind::Ry => {
            let t = sign * angle / 2.0;
            [
                [C::new(t.cos(), 0.0), C::new(-t.sin(), 0.0)],
                [C::new(t.sin(), 0.0), C::new(t.cos(), 0.0)],
            ]
        }
        GateKind::Rz => {
            let t = sign * angle / 2.0;
            [[C::exp_i(-t), C::ZERO], [C::ZERO, C::exp_i(t)]]
        }
        _ => [[C::ONE, C::ZERO], [C::ZERO, C::ONE]],
    }
}

/// Quantum state over the live qubits. Bit `p` of an amplitude index is the qubit at position
/// `p` in `ids`.
#[derive(Debug, Default)]
struct Quantum {
    ids: Vec<u32>,
    amps: Vec<C>,
    /// Every gate applied, with its qubit IDs and angle, in execution order.
    history: Vec<Record>,
    /// Index into `history` where each qubit was allocated.
    alloc_at: HashMap<u32, usize>,
    next_id: u32,
    peak: usize,
}

#[derive(Debug, Clone)]
struct Record {
    gate: GateKind,
    targets: Vec<u32>,
    angle: f64,
    /// Set once a release has applied this gate's inverse; it no longer entangles anything.
    undone: bool,
}

impl Quantum {
    fn pos(&self, id: u32) -> Result<usize, SimError> {
        self.ids
            .iter()
            .position(|&i| i == id)
            .ok_or_else(|| SimError::new(codes::RUNTIME_ERROR, format!("qubit {id} is not live")))
    }

    fn alloc(&mut self) -> Result<u32, SimError> {
        if self.ids.len() >= MAX_QUBITS {
            return Err(SimError::new(
                codes::SIMULATOR_CAPACITY,
                format!("the program needs more than {MAX_QUBITS} live qubits"),
            ));
        }
        let id = self.next_id;
        self.next_id += 1;
        if self.amps.is_empty() {
            // The empty product state is the scalar 1.
            self.amps.push(C::ONE);
        }
        let n = self.amps.len();
        self.amps.resize(2 * n, C::ZERO);
        // The new qubit is the highest bit; the old amplitudes stay where they were.
        self.ids.push(id);
        self.alloc_at.insert(id, self.history.len());
        self.peak = self.peak.max(self.ids.len());
        Ok(id)
    }

    fn apply(
        &mut self,
        gate: GateKind,
        targets: &[u32],
        angle: f64,
        adjoint: bool,
    ) -> Result<(), SimError> {
        let pos: Vec<usize> = targets.iter().map(|&t| self.pos(t)).collect::<Result<_, _>>()?;
        match gate {
            GateKind::Cx | GateKind::Cz | GateKind::Swap | GateKind::Cp => {
                self.apply_two(gate, pos[0], pos[1], angle, adjoint)
            }
            _ => {
                let m = matrix1(gate, angle, adjoint);
                self.apply_one(pos[0], m);
            }
        }
        Ok(())
    }

    fn apply_one(&mut self, p: usize, m: M2) {
        let bit = 1usize << p;
        for i in 0..self.amps.len() {
            if i & bit == 0 {
                let j = i | bit;
                let (a, b) = (self.amps[i], self.amps[j]);
                self.amps[i] = m[0][0].mul(a).add(m[0][1].mul(b));
                self.amps[j] = m[1][0].mul(a).add(m[1][1].mul(b));
            }
        }
    }

    fn apply_two(&mut self, gate: GateKind, c: usize, t: usize, angle: f64, adjoint: bool) {
        let cb = 1usize << c;
        let tb = 1usize << t;
        match gate {
            GateKind::Cx => {
                for i in 0..self.amps.len() {
                    if i & cb != 0 && i & tb == 0 {
                        self.amps.swap(i, i | tb);
                    }
                }
            }
            GateKind::Cz => {
                for i in 0..self.amps.len() {
                    if i & cb != 0 && i & tb != 0 {
                        self.amps[i] = self.amps[i].scale(-1.0);
                    }
                }
            }
            GateKind::Cp => {
                let theta = if adjoint { -angle } else { angle };
                let phase = C::exp_i(theta);
                for i in 0..self.amps.len() {
                    if i & cb != 0 && i & tb != 0 {
                        self.amps[i] = self.amps[i].mul(phase);
                    }
                }
            }
            GateKind::Swap => {
                for i in 0..self.amps.len() {
                    // Swap each pair once: bit c = 0, bit t = 1 with its partner.
                    if i & cb == 0 && i & tb != 0 {
                        self.amps.swap(i, (i | cb) & !tb);
                    }
                }
            }
            _ => {}
        }
    }

    /// Probability that qubit at position `p` reads 1.
    fn prob_one(&self, p: usize) -> f64 {
        let bit = 1usize << p;
        self.amps.iter().enumerate().filter(|(i, _)| i & bit != 0).map(|(_, a)| a.norm2()).sum()
    }

    /// Project qubit `id` onto `outcome` and remove it from the state.
    fn collapse_and_remove(&mut self, id: u32, outcome: bool) -> Result<(), SimError> {
        let p = self.pos(id)?;
        let bit = 1usize << p;
        let keep: Vec<usize> =
            (0..self.amps.len()).filter(|i| ((i & bit) != 0) == outcome).collect();
        let norm: f64 = keep.iter().map(|&i| self.amps[i].norm2()).sum::<f64>().sqrt();
        if norm < 1e-12 {
            return Err(SimError::new(
                codes::RUNTIME_ERROR,
                "projected onto a zero-probability outcome",
            ));
        }
        // Kept indices in increasing order are exactly the compressed indices of the remaining
        // qubits, so collecting them in order removes bit `p`.
        self.amps = keep.iter().map(|&i| self.amps[i].scale(1.0 / norm)).collect();
        self.ids.remove(p);
        Ok(())
    }

    /// Measure one qubit, sample from its distribution, collapse, and remove it.
    fn measure(&mut self, id: u32, rng: &mut StdRng) -> Result<bool, SimError> {
        let p = self.pos(id)?;
        let p1 = self.prob_one(p).clamp(0.0, 1.0);
        let outcome = rng.gen::<f64>() < p1;
        self.collapse_and_remove(id, outcome)?;
        Ok(outcome)
    }

    /// Uncompute and release `id`: undo the gates in its causal cone, check it is |0⟩, remove it.
    fn release(&mut self, id: u32) -> Result<(), SimError> {
        let start = *self.alloc_at.get(&id).ok_or_else(|| {
            SimError::new(codes::RUNTIME_ERROR, format!("qubit {id} was never allocated"))
        })?;
        // Forward pass over the history since allocation: collect the cone of entanglement.
        let mut cone: HashSet<u32> = HashSet::from([id]);
        let mut undo: Vec<usize> = Vec::new();
        for (k, rec) in self.history.iter().enumerate().skip(start) {
            if rec.undone {
                continue;
            }
            if rec.targets.iter().any(|t| cone.contains(t)) {
                cone.extend(rec.targets.iter().copied());
                undo.push(k);
            }
        }
        for &t in cone.iter() {
            if t != id && !self.ids.contains(&t) {
                return Err(SimError::new(
                    codes::CANNOT_UNCOMPUTE,
                    format!("qubit {id} is entangled with qubit {t}, which is no longer live"),
                ));
            }
        }
        // Undo in reverse program order: each adjoint restores the state before its gate.
        for &k in undo.iter().rev() {
            let rec = self.history[k].clone();
            self.apply(rec.gate, &rec.targets, rec.angle, true)?;
        }
        for &k in &undo {
            self.history[k].undone = true;
        }
        let p = self.pos(id)?;
        if self.prob_one(p) > ZERO_TOL {
            return Err(SimError::new(
                codes::CANNOT_UNCOMPUTE,
                format!("qubit {id} did not return to |0⟩ after uncomputation"),
            ));
        }
        self.collapse_and_remove(id, false)?;
        Ok(())
    }
}

/// Interpreter state for one shot.
struct Machine<'g, 'r, 's> {
    gir: &'g Gir,
    rng: &'r mut StdRng,
    shifts: &'s ShiftPlan,
    /// Depolarizing probability after each gate, per target qubit.
    depolarizing: f64,
    quantum: Quantum,
    output: Vec<String>,
    steps: u64,
    peak: usize,
}

impl<'g, 'r, 's> Machine<'g, 'r, 's> {
    fn new(gir: &'g Gir, rng: &'r mut StdRng, shifts: &'s ShiftPlan, depolarizing: f64) -> Self {
        Machine {
            gir,
            rng,
            shifts,
            depolarizing,
            quantum: Quantum::default(),
            output: Vec::new(),
            steps: 0,
            peak: 0,
        }
    }

    fn call(&mut self, func: usize, args: Vec<Value>) -> Result<Value, SimError> {
        let f =
            self.gir.funcs.get(func).ok_or_else(|| {
                SimError::new(codes::RUNTIME_ERROR, format!("no function #{func}"))
            })?;
        if f.blocks.is_empty() {
            return Err(SimError::new(
                codes::RUNTIME_ERROR,
                format!("`{}` has no body to run", f.name),
            ));
        }
        let mut slots = vec![Value::Unit; f.slot_count as usize];
        for (slot, arg) in f.params.iter().zip(args) {
            slots[*slot as usize] = arg;
        }
        let mut block = 0usize;
        loop {
            let b = &f.blocks[block];
            for (ii, inst) in b.insts.iter().enumerate() {
                self.steps += 1;
                if self.steps > STEP_LIMIT {
                    return Err(SimError::new(
                        codes::RUNTIME_ERROR,
                        "the program did not finish within the step limit",
                    ));
                }
                let shift = self.shifts.get(&(func, block as u32, ii)).copied().unwrap_or(0.0);
                self.exec(inst, &mut slots, shift)?;
            }
            self.peak = self.peak.max(self.quantum.peak);
            match &b.term {
                Term::Unreachable => {
                    return Err(SimError::new(codes::RUNTIME_ERROR, "reached an unreachable block"))
                }
                Term::Jump(t) => block = *t as usize,
                Term::Branch { cond, then_block, else_block } => {
                    let c = truthy(&slots[*cond as usize])?;
                    block = if c { *then_block as usize } else { *else_block as usize };
                }
                Term::Return(v) => {
                    return Ok(match v {
                        Some(s) => slots[*s as usize].clone(),
                        None => Value::Unit,
                    })
                }
            }
        }
    }

    fn exec(&mut self, inst: &Inst, slots: &mut [Value], shift: f64) -> Result<(), SimError> {
        match inst {
            Inst::Const { dst, value } => slots[*dst as usize] = from_const(value),
            Inst::Copy { dst, src } => slots[*dst as usize] = slots[*src as usize].clone(),
            Inst::Bin { dst, op, a, b } => {
                let v = bin(*op, &slots[*a as usize], &slots[*b as usize])?;
                slots[*dst as usize] = v;
            }
            Inst::Un { dst, op, src } => {
                let v = match (op, &slots[*src as usize]) {
                    (UnOp::Neg, Value::Int(i)) => Value::Int(-i),
                    (UnOp::Neg, Value::Float(f)) => Value::Float(-f),
                    (UnOp::Not, Value::Bool(b)) => Value::Bool(!b),
                    (_, other) => return Err(bad_operand("unary operator", other)),
                };
                slots[*dst as usize] = v;
            }
            Inst::Tuple { dst, items } => {
                slots[*dst as usize] =
                    Value::Tuple(items.iter().map(|s| slots[*s as usize].clone()).collect());
            }
            Inst::VecLit { dst, items } => {
                slots[*dst as usize] =
                    Value::Vec(items.iter().map(|s| slots[*s as usize].clone()).collect());
            }
            Inst::Index { dst, base, index } => {
                let i = as_index(&slots[*index as usize])?;
                let v = match &slots[*base as usize] {
                    Value::Register(ids) => {
                        Value::Qubit(*ids.get(i).ok_or_else(|| out_of_range(i, ids.len()))?)
                    }
                    Value::Vec(items) | Value::Tuple(items) => {
                        items.get(i).cloned().ok_or_else(|| out_of_range(i, items.len()))?
                    }
                    other => return Err(bad_operand("indexing", other)),
                };
                slots[*dst as usize] = v;
            }
            Inst::Len { dst, src } => {
                let n = match &slots[*src as usize] {
                    Value::Register(ids) => ids.len(),
                    Value::Vec(items) | Value::Tuple(items) => items.len(),
                    other => return Err(bad_operand("len", other)),
                };
                slots[*dst as usize] = Value::Int(n as i64);
            }
            Inst::TupleGet { dst, src, i } => {
                let v = match &slots[*src as usize] {
                    Value::Tuple(items) => items.get(*i).cloned().unwrap_or(Value::Unit),
                    other => return Err(bad_operand("destructuring", other)),
                };
                slots[*dst as usize] = v;
            }
            Inst::Call { dsts, func, args } => {
                let argv: Vec<Value> = args.iter().map(|s| slots[*s as usize].clone()).collect();
                let result = self.call(*func, argv)?;
                if let Some(d) = dsts.first() {
                    slots[*d as usize] = result;
                }
            }
            Inst::AllocQubit { dst } => {
                let id = self.quantum.alloc()?;
                slots[*dst as usize] = Value::Qubit(id);
            }
            Inst::AllocRegister { dst, n } => {
                let mut ids = Vec::with_capacity(*n);
                for _ in 0..*n {
                    ids.push(self.quantum.alloc()?);
                }
                slots[*dst as usize] = Value::Register(ids);
            }
            Inst::Gate { gate, targets, angle, dsts } => {
                let ids = qubit_ids(targets.iter().map(|s| &slots[*s as usize]))?;
                let theta = match angle {
                    Some(s) => as_float(&slots[*s as usize])? + shift,
                    None => 0.0,
                };
                self.quantum.apply(*gate, &ids, theta, false)?;
                self.quantum.history.push(Record {
                    gate: *gate,
                    targets: ids.clone(),
                    angle: theta,
                    undone: false,
                });
                // Physical error after the gate. It is not recorded: uncomputation undoes the
                // program's gates, not the noise.
                if self.depolarizing > 0.0 {
                    for id in &ids {
                        if self.rng.gen::<f64>() < self.depolarizing {
                            let pauli = match (self.rng.gen::<f64>() * 3.0) as u32 {
                                0 => GateKind::X,
                                1 => GateKind::Y,
                                _ => GateKind::Z,
                            };
                            self.quantum.apply(pauli, &[*id], 0.0, false)?;
                        }
                    }
                }
                for (d, id) in dsts.iter().zip(ids) {
                    slots[*d as usize] = Value::Qubit(id);
                }
            }
            Inst::Measure { dst, target } => {
                let v = match &slots[*target as usize] {
                    Value::Qubit(id) => Value::Bool(self.quantum.measure(*id, self.rng)?),
                    Value::Register(ids) => {
                        let mut bits = Vec::with_capacity(ids.len());
                        for id in ids.clone() {
                            bits.push(Value::Bool(self.quantum.measure(id, self.rng)?));
                        }
                        Value::Vec(bits)
                    }
                    other => return Err(bad_operand("measure", other)),
                };
                slots[*dst as usize] = v;
            }
            Inst::Release { target } => match &slots[*target as usize] {
                Value::Qubit(id) => self.quantum.release(*id)?,
                Value::Register(ids) => {
                    for id in ids.clone() {
                        self.quantum.release(id)?;
                    }
                }
                other => return Err(bad_operand("drop", other)),
            },
            Inst::Print { args } => {
                let parts: Vec<String> =
                    args.iter().map(|s| slots[*s as usize].to_string()).collect();
                self.output.push(parts.join(" "));
            }
        }
        Ok(())
    }
}

fn qubit_ids<'v>(values: impl Iterator<Item = &'v Value>) -> Result<Vec<u32>, SimError> {
    values
        .map(|v| match v {
            Value::Qubit(id) => Ok(*id),
            other => Err(bad_operand("a gate", other)),
        })
        .collect()
}

fn from_const(c: &Const) -> Value {
    match c {
        Const::Unit => Value::Unit,
        Const::Int(v) => Value::Int(*v),
        Const::Float(v) => Value::Float(*v),
        Const::Bool(v) => Value::Bool(*v),
        Const::Str(s) => Value::Str(s.clone()),
        Const::Complex(re, im) => Value::Complex(*re, *im),
    }
}

fn truthy(v: &Value) -> Result<bool, SimError> {
    match v {
        Value::Bool(b) => Ok(*b),
        other => Err(bad_operand("a condition", other)),
    }
}

fn as_float(v: &Value) -> Result<f64, SimError> {
    match v {
        Value::Float(f) => Ok(*f),
        Value::Int(i) => Ok(*i as f64),
        other => Err(bad_operand("an angle", other)),
    }
}

fn as_index(v: &Value) -> Result<usize, SimError> {
    match v {
        Value::Int(i) if *i >= 0 => Ok(*i as usize),
        Value::Int(i) => Err(SimError::new(codes::RUNTIME_ERROR, format!("negative index {i}"))),
        other => Err(bad_operand("an index", other)),
    }
}

fn out_of_range(i: usize, len: usize) -> SimError {
    SimError::new(codes::RUNTIME_ERROR, format!("index {i} is out of range for length {len}"))
}

fn bad_operand(what: &str, v: &Value) -> SimError {
    SimError::new(codes::RUNTIME_ERROR, format!("{what} cannot be applied to {v}"))
}

fn bin(op: BinOp, a: &Value, b: &Value) -> Result<Value, SimError> {
    use Value::*;
    Ok(match (a, b) {
        (Int(x), Int(y)) => match op {
            BinOp::Add => Int(x.wrapping_add(*y)),
            BinOp::Sub => Int(x.wrapping_sub(*y)),
            BinOp::Mul => Int(x.wrapping_mul(*y)),
            BinOp::Div => {
                if *y == 0 {
                    return Err(SimError::new(codes::RUNTIME_ERROR, "division by zero"));
                }
                Int(x.wrapping_div(*y))
            }
            BinOp::Mod => {
                if *y == 0 {
                    return Err(SimError::new(codes::RUNTIME_ERROR, "remainder by zero"));
                }
                Int(x.wrapping_rem(*y))
            }
            BinOp::Eq => Bool(x == y),
            BinOp::Ne => Bool(x != y),
            BinOp::Lt => Bool(x < y),
            BinOp::Le => Bool(x <= y),
            BinOp::Gt => Bool(x > y),
            BinOp::Ge => Bool(x >= y),
            _ => return Err(bad_operand("this operator", a)),
        },
        (Int(_) | Float(_), Int(_) | Float(_)) => {
            let x = as_float(a)?;
            let y = as_float(b)?;
            match op {
                BinOp::Add => Float(x + y),
                BinOp::Sub => Float(x - y),
                BinOp::Mul => Float(x * y),
                BinOp::Div => Float(x / y),
                BinOp::Eq => Bool(x == y),
                BinOp::Ne => Bool(x != y),
                BinOp::Lt => Bool(x < y),
                BinOp::Le => Bool(x <= y),
                BinOp::Gt => Bool(x > y),
                BinOp::Ge => Bool(x >= y),
                _ => return Err(bad_operand("this operator", a)),
            }
        }
        (Bool(x), Bool(y)) => match op {
            BinOp::And => Bool(*x && *y),
            BinOp::Or => Bool(*x || *y),
            BinOp::Eq => Bool(x == y),
            BinOp::Ne => Bool(x != y),
            _ => return Err(bad_operand("this operator", a)),
        },
        _ => return Err(bad_operand("this operator", a)),
    })
}

/// Exact output distribution of a unitary gate sequence on `num_qubits` qubits starting from
/// |0…0⟩. Index `k` of the result is the probability of basis state `k`, where bit `i` is qubit
/// `i`. Used to check that two gate sequences are equivalent (for example, a backend's output
/// against the source program).
pub fn exact_probabilities(
    num_qubits: usize,
    ops: &[(GateKind, Vec<u32>, f64)],
) -> Result<Vec<f64>, SimError> {
    let mut q = Quantum::default();
    for _ in 0..num_qubits {
        q.alloc()?;
    }
    for (gate, targets, angle) in ops {
        q.apply(*gate, targets, *angle, false)?;
    }
    // The state vector is indexed by position, and positions equal qubit IDs here.
    Ok(q.amps.iter().map(|a| a.norm2()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gala_hir::lower_file;
    use gala_parser::parse_source;
    use gala_span::FileId;

    fn gir_of(src: &str) -> Gir {
        let out = parse_source(FileId(0), src);
        assert!(!out.diagnostics.has_errors(), "parse: {:?}", out.diagnostics);
        let (pkg, _) = lower_file(FileId(0), out.items);
        let (info, td) = gala_types::check_package(&pkg);
        assert!(!td.has_errors(), "types: {:?}", td.errors());
        gala_gir::lower_package(&pkg, &info).expect("lowering")
    }

    fn run(src: &str, shots: u64) -> RunReport {
        run_entry(&gir_of(src), &SimConfig { shots, seed: 42, ..Default::default() }).expect("run")
    }

    #[test]
    fn classical_program_returns_value() {
        let r = run("fn main() -> Int { let a = 20\n a + 22 }", 1);
        assert_eq!(r.first_result, Some(Value::Int(42)));
    }

    #[test]
    fn print_writes_output() {
        let r = run("fn main() -> Int { print(\"hi\")\n 0 }", 1);
        assert_eq!(r.output, vec!["hi".to_string()]);
    }

    #[test]
    fn x_gate_flips_to_one_deterministically() {
        let r =
            run("fn main() -> Measured<Bool> prob { let q = qubit()\n x(q)\n measure(q) }", 200);
        assert_eq!(r.histogram.get("true"), Some(&200), "{:?}", r.histogram);
    }

    #[test]
    fn bell_pair_correlates_and_splits_evenly() {
        // Measuring both qubits of a Bell pair gives equal bits; the pair is never 01 or 10.
        let r = run(
            "fn main() -> Measured<Vec<Bool>> prob {\n let q = qubits<2>()\n h(q[0])\n cx(q[0], q[1])\n measure(q)\n}",
            2000,
        );
        let zeros = r.histogram.get("[false, false]").copied().unwrap_or(0);
        let ones = r.histogram.get("[true, true]").copied().unwrap_or(0);
        assert_eq!(zeros + ones, 2000, "only correlated outcomes: {:?}", r.histogram);
        assert!((zeros as f64 - 1000.0).abs() < 150.0, "roughly even: {zeros}");
    }

    #[test]
    fn deterministic_under_fixed_seed() {
        let src = "fn main() -> Measured<Bool> prob { let q = qubit()\n h(q)\n measure(q) }";
        assert_eq!(run(src, 300).histogram, run(src, 300).histogram);
    }

    #[test]
    fn uncompute_returns_ancilla_to_zero() {
        // The ancilla is entangled with d; releasing it must undo that and leave |0⟩.
        let r = run(
            "fn main() -> Measured<Bool> prob {\n let d = qubit()\n h(d)\n let a = qubit()\n cx(d, a)\n cx(d, a)\n drop(a)\n h(d)\n h(d)\n measure(d)\n}",
            1,
        );
        assert_eq!(r.first_result, Some(Value::Bool(false)));
    }

    #[test]
    fn released_entangled_ancilla_undoes_entanglement() {
        // a = d after cx; uncompute must restore a to |0⟩ even with a live partner in superposition.
        let r = run(
            "fn main() -> Measured<Bool> prob {\n let d = qubit()\n h(d)\n let a = qubit()\n cx(d, a)\n drop(a)\n measure(d)\n}",
            300,
        );
        assert!(
            r.histogram.contains_key("true") && r.histogram.contains_key("false"),
            "{:?}",
            r.histogram
        );
    }

    #[test]
    fn controlled_phase_and_swap_act_as_expected() {
        // X on q0, swap to q1: measuring q1 gives 1.
        let r = run(
            "fn main() -> Measured<Bool> prob {\n let a = qubit()\n let b = qubit()\n x(a)\n let (a, b) = swap(a, b)\n measure(b)\n}",
            50,
        );
        assert_eq!(r.histogram.get("true"), Some(&50));
    }

    #[test]
    fn capacity_limit_is_reported() {
        let gir = gir_of("fn main() -> Int { let q = qubits<30>()\n 0 }");
        let err = run_entry(&gir, &SimConfig::default()).unwrap_err();
        assert_eq!(err.code.to_string(), "E0702");
    }

    #[test]
    fn depolarizing_noise_flips_a_flipped_qubit_at_the_expected_rate() {
        // X takes |0⟩ to |1⟩. A depolarizing error of probability p after the gate turns |1⟩ back
        // into |0⟩ with X or Y (probability 2p/3), so P(false) = 2p/3.
        let src = "fn main() -> Measured<Bool> prob { let q = qubit()\n x(q)\n measure(q) }";
        let gir = gir_of(src);
        let p = 0.3;
        let report =
            run_entry(&gir, &SimConfig { shots: 20_000, seed: 9, depolarizing: p }).unwrap();
        let false_rate = report.histogram.get("false").copied().unwrap_or(0) as f64 / 20_000.0;
        assert!((false_rate - 2.0 * p / 3.0).abs() < 0.02, "rate {false_rate}");
        let clean = run_entry(&gir, &SimConfig { shots: 500, seed: 9, depolarizing: 0.0 }).unwrap();
        assert_eq!(clean.histogram.get("true"), Some(&500));
    }

    #[test]
    fn matrices_are_unitary() {
        for g in [
            GateKind::H,
            GateKind::X,
            GateKind::Y,
            GateKind::Z,
            GateKind::S,
            GateKind::T,
            GateKind::Rx,
            GateKind::Ry,
            GateKind::Rz,
        ] {
            for &theta in &[0.0, 0.7, -2.1] {
                let m = matrix1(g, theta, false);
                // Columns are orthonormal: M†M = I.
                for c in 0..2 {
                    for d in 0..2 {
                        let dot = m[0][c].re * m[0][d].re
                            + m[0][c].im * m[0][d].im
                            + m[1][c].re * m[1][d].re
                            + m[1][c].im * m[1][d].im;
                        let want = if c == d { 1.0 } else { 0.0 };
                        assert!((dot - want).abs() < 1e-12, "{g:?} {theta}");
                    }
                }
            }
        }
    }
}

/// Exact action of a register-to-register function on one computational basis input.
///
/// Prepares an `n`-qubit register in basis state `input` (bit `i` is register qubit `i`), calls
/// `func` on it, and returns the output register's amplitudes in the function's returned order
/// (index bit `j` is the `j`-th returned qubit). The function must leave no other qubit live.
pub fn basis_action(
    gir: &Gir,
    func: usize,
    n: usize,
    input: usize,
    as_register: bool,
) -> Result<Vec<(f64, f64)>, SimError> {
    let shifts = ShiftPlan::new();
    let mut rng = StdRng::seed_from_u64(0);
    let mut m = Machine::new(gir, &mut rng, &shifts, 0.0);
    let mut ids = Vec::with_capacity(n);
    for _ in 0..n {
        ids.push(m.quantum.alloc()?);
    }
    for (i, &id) in ids.iter().enumerate() {
        if input >> i & 1 == 1 {
            m.quantum.apply(GateKind::X, &[id], 0.0, false)?;
        }
    }
    let arg = if as_register { Value::Register(ids) } else { Value::Qubit(ids[0]) };
    let out = match m.call(func, vec![arg])? {
        Value::Register(v) => v,
        Value::Qubit(id) => vec![id],
        other => {
            return Err(SimError::new(
                codes::RUNTIME_ERROR,
                format!("the function returned {other}, not a qubit or register"),
            ))
        }
    };
    let mut live: Vec<u32> = m.quantum.ids.clone();
    let mut want = out.clone();
    live.sort_unstable();
    want.sort_unstable();
    if live != want {
        return Err(SimError::new(
            codes::RUNTIME_ERROR,
            "the function must return exactly the qubits it was given, with no others left live",
        ));
    }
    let positions: Vec<usize> =
        out.iter().map(|&id| m.quantum.pos(id)).collect::<Result<_, _>>()?;
    let dim = 1usize << out.len();
    let mut amps = Vec::with_capacity(dim);
    for x in 0..dim {
        let mut phys = 0usize;
        for (j, &p) in positions.iter().enumerate() {
            if x >> j & 1 == 1 {
                phys |= 1 << p;
            }
        }
        let a = m.quantum.amps[phys];
        amps.push((a.re, a.im));
    }
    Ok(amps)
}

/// Fidelity of `func` as a round trip on basis input `input`: run it, then apply the inverse of
/// every gate it applied (in reverse), and return the probability of getting `input` back.
/// A reversible function returns 1.0; the gates undone by a release are not undone twice.
pub fn reversible_roundtrip(
    gir: &Gir,
    func: usize,
    n: usize,
    input: usize,
    as_register: bool,
) -> Result<f64, SimError> {
    let shifts = ShiftPlan::new();
    let mut rng = StdRng::seed_from_u64(0);
    let mut m = Machine::new(gir, &mut rng, &shifts, 0.0);
    let mut ids = Vec::with_capacity(n);
    for _ in 0..n {
        ids.push(m.quantum.alloc()?);
    }
    for (i, &id) in ids.iter().enumerate() {
        if input >> i & 1 == 1 {
            m.quantum.apply(GateKind::X, &[id], 0.0, false)?;
        }
    }
    let arg = if as_register { Value::Register(ids.clone()) } else { Value::Qubit(ids[0]) };
    m.call(func, vec![arg])?;
    let records: Vec<Record> = m.quantum.history.iter().filter(|r| !r.undone).cloned().collect();
    for rec in records.iter().rev() {
        m.quantum.apply(rec.gate, &rec.targets, rec.angle, true)?;
    }
    let live = m.quantum.ids.clone();
    let mut want = ids.clone();
    let mut have = live.clone();
    want.sort_unstable();
    have.sort_unstable();
    if want != have {
        return Err(SimError::new(
            codes::RUNTIME_ERROR,
            "the round trip left a different set of qubits live",
        ));
    }
    // Index of `input` in the state's own ordering of the same qubits.
    let mut index = 0usize;
    for (i, &id) in ids.iter().enumerate() {
        if input >> i & 1 == 1 {
            index |= 1 << m.quantum.pos(id)?;
        }
    }
    Ok(m.quantum.amps[index].norm2())
}

#[cfg(test)]
mod basis_tests {
    use super::*;
    use gala_hir::lower_file;
    use gala_parser::parse_source;
    use gala_span::FileId;

    fn gir_of(src: &str) -> Gir {
        let out = parse_source(FileId(0), src);
        let (pkg, _) = lower_file(FileId(0), out.items);
        let (info, _) = gala_types::check_package(&pkg);
        gala_gir::lower_package(&pkg, &info).expect("lowers")
    }

    #[test]
    fn x_gate_maps_basis_zero_to_basis_one() {
        let gir =
            gir_of("fn flip(q: Qubit) -> Qubit quantum {\n    x(q)\n}\nfn main() -> Int { 0 }\n");
        let func = gir.func_named("flip").unwrap();
        let amps = basis_action(&gir, func, 1, 0, false).unwrap();
        assert!(amps[0].0.hypot(amps[0].1) < 1e-12);
        assert!((amps[1].0 - 1.0).abs() < 1e-12);
        assert!((reversible_roundtrip(&gir, func, 1, 0, false).unwrap() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn dropping_a_nonzero_input_is_refused_at_release() {
        // Dropping the input and returning a fresh |0⟩ maps both basis inputs to the same
        // output, so its columns coincide and it cannot be unitary.
        let gir = gir_of(
            "fn bad(q: Qubit) -> Qubit quantum {\n    drop(q)\n    qubit()\n}\nfn main() -> Int { 0 }\n",
        );
        let func = gir.func_named("bad").unwrap();
        // The |0⟩ input is released in |0⟩ and returns a fresh |0⟩ ...
        let zero = basis_action(&gir, func, 1, 0, false).unwrap();
        assert!((zero[0].0 - 1.0).abs() < 1e-12);
        // ... while |1⟩ fails the release check, because a release requires |0⟩.
        let one = basis_action(&gir, func, 1, 1, false).unwrap_err();
        assert_eq!(one.code.to_string(), "E0530");
    }
}
