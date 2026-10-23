//! GIR optimizer. Every rewrite preserves the program's behavior on the simulator.
//!
//! Rewrites:
//! - **Self-inverse cancellation.** Two adjacent applications of the same self-inverse gate
//!   (`H`, `X`, `Y`, `Z`, `CX`, `CZ`, `SWAP`) on the same qubits cancel. The second gate's outputs
//!   are renamed to the first gate's inputs, which is sound because they are the same qubits.
//!   A cancellation is skipped when the input slot is reassigned anywhere in the function.

pub mod route;

use gala_gir::{GateKind, Gir, Inst, Slot, Term};
use std::collections::{HashMap, HashSet};

/// Optimize every function of a program.
pub fn optimize(gir: &Gir) -> Gir {
    let mut out = gir.clone();
    for f in &mut out.funcs {
        cancel_self_inverse(f);
        merge_rotations(f);
        fuse_single_qubit(f);
    }
    out
}

/// Merge adjacent rotations about the same axis on the same qubit into one rotation by the sum of
/// their angles. Applies only when both angles are literal constants defined once, so the merged
/// angle is known at compile time. Rotations about one axis commute and compose additively.
fn merge_rotations(f: &mut gala_gir::GirFunc) {
    let mut defs: HashMap<Slot, usize> = HashMap::new();
    let mut consts: HashMap<Slot, f64> = HashMap::new();
    for b in &f.blocks {
        for i in &b.insts {
            for d in defined_slots(i) {
                *defs.entry(d).or_insert(0) += 1;
            }
            if let Inst::Const { dst, value } = i {
                let v = match value {
                    gala_gir::Const::Float(v) => Some(*v),
                    gala_gir::Const::Int(v) => Some(*v as f64),
                    _ => None,
                };
                if let Some(v) = v {
                    consts.insert(*dst, v);
                }
            }
        }
    }
    let single = |s: &Slot| defs.get(s) == Some(&1);
    let mut next_slot = f.slot_count;
    let mut rename: HashMap<Slot, Slot> = HashMap::new();
    for b in &mut f.blocks {
        let mut kept: Vec<Inst> = Vec::with_capacity(b.insts.len());
        for inst in b.insts.drain(..) {
            if let Inst::Gate { gate, targets, angle: Some(a2), dsts } = &inst {
                if matches!(gate, GateKind::Rx | GateKind::Ry | GateKind::Rz) {
                    // Constants between the two rotations do not touch the qubit, so look back
                    // past them to the previous instruction that does.
                    let prev = kept.iter().rposition(|i| !matches!(i, Inst::Const { .. }));
                    let found = prev.and_then(|j| match &kept[j] {
                        Inst::Gate { gate: g0, targets: t0, angle: Some(a1), dsts: d0 } => {
                            Some((j, *g0, t0.clone(), *a1, d0.clone()))
                        }
                        _ => None,
                    });
                    if let Some((j, g0, t0, a1, d0)) = found {
                        if let (Some(x), Some(y)) = (consts.get(&a1), consts.get(a2)) {
                            if g0 == *gate && t0 == *targets && single(&a1) && single(a2) {
                                let total = x + y;
                                let angle_slot = next_slot;
                                next_slot += 1;
                                kept.remove(j);
                                kept.push(Inst::Const {
                                    dst: angle_slot,
                                    value: gala_gir::Const::Float(total),
                                });
                                kept.push(Inst::Gate {
                                    gate: g0,
                                    targets: t0,
                                    angle: Some(angle_slot),
                                    dsts: d0.clone(),
                                });
                                // The second rotation's outputs are the same qubits as the merged one.
                                for (out, keep) in dsts.iter().zip(d0.iter()) {
                                    rename.insert(*out, *keep);
                                }
                                continue;
                            }
                        }
                    }
                }
            }
            kept.push(inst);
        }
        b.insts = kept;
    }
    f.slot_count = next_slot;
    if rename.is_empty() {
        return;
    }
    let resolve = |mut s: Slot| {
        while let Some(&n) = rename.get(&s) {
            s = n;
        }
        s
    };
    for b in &mut f.blocks {
        for inst in &mut b.insts {
            rewrite_uses(inst, &resolve);
        }
        match &mut b.term {
            Term::Branch { cond, .. } => *cond = resolve(*cond),
            Term::Return(Some(s)) => *s = resolve(*s),
            _ => {}
        }
    }
}

fn self_inverse(g: GateKind) -> bool {
    matches!(
        g,
        GateKind::H
            | GateKind::X
            | GateKind::Y
            | GateKind::Z
            | GateKind::Cx
            | GateKind::Cz
            | GateKind::Swap
    )
}

fn cancel_self_inverse(f: &mut gala_gir::GirFunc) {
    // A slot with more than one static definition is reassigned (a `mut` local, for example), so
    // renaming through it is unsafe.
    let mut defs: HashMap<Slot, usize> = HashMap::new();
    for b in &f.blocks {
        for i in &b.insts {
            for d in defined_slots(i) {
                *defs.entry(d).or_insert(0) += 1;
            }
        }
    }
    let reassigned: HashSet<Slot> = defs.iter().filter(|(_, n)| **n > 1).map(|(s, _)| *s).collect();
    let mut rename: HashMap<Slot, Slot> = HashMap::new();
    for b in &mut f.blocks {
        let mut kept: Vec<Inst> = Vec::with_capacity(b.insts.len());
        for inst in b.insts.drain(..) {
            if let Inst::Gate { gate, targets, angle: None, dsts } = &inst {
                if let Some(Inst::Gate { gate: g0, targets: t0, angle: None, dsts: d0 }) =
                    kept.last()
                {
                    // Both gates act on the same qubits (same input slots): applying a
                    // self-inverse gate twice in a row is the identity on those qubits.
                    let same = g0 == gate
                        && self_inverse(*gate)
                        && t0 == targets
                        && t0.iter().all(|s| !reassigned.contains(s));
                    if same {
                        // Every output of both gates carries the same qubits as the inputs.
                        for (i, in_slot) in t0.iter().enumerate() {
                            if let Some(o) = dsts.get(i) {
                                rename.insert(*o, *in_slot);
                            }
                            if let Some(o) = d0.get(i) {
                                rename.insert(*o, *in_slot);
                            }
                        }
                        kept.pop();
                        continue;
                    }
                }
            }
            kept.push(inst);
        }
        b.insts = kept;
    }
    if rename.is_empty() {
        return;
    }
    let resolve = |mut s: Slot| {
        while let Some(&n) = rename.get(&s) {
            s = n;
        }
        s
    };
    for b in &mut f.blocks {
        for inst in &mut b.insts {
            rewrite_uses(inst, &resolve);
        }
        match &mut b.term {
            Term::Branch { cond, .. } => *cond = resolve(*cond),
            Term::Return(Some(s)) => *s = resolve(*s),
            _ => {}
        }
    }
}

/// Slots an instruction writes.
fn defined_slots(inst: &Inst) -> Vec<Slot> {
    match inst {
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
        | Inst::Measure { dst, .. }
        | Inst::Run { dst, .. } => vec![*dst],
        Inst::Call { dsts, .. } | Inst::Gate { dsts, .. } => dsts.clone(),
        Inst::Release { .. } | Inst::Print { .. } => Vec::new(),
    }
}

fn rewrite_uses(inst: &mut Inst, r: &impl Fn(Slot) -> Slot) {
    match inst {
        Inst::Copy { src, .. } => *src = r(*src),
        Inst::Bin { a, b, .. } => {
            *a = r(*a);
            *b = r(*b);
        }
        Inst::Un { src, .. } | Inst::Len { src, .. } | Inst::TupleGet { src, .. } => *src = r(*src),
        Inst::Tuple { items, .. } | Inst::VecLit { items, .. } | Inst::Print { args: items } => {
            for s in items {
                *s = r(*s);
            }
        }
        Inst::Index { base, index, .. } => {
            *base = r(*base);
            *index = r(*index);
        }
        Inst::Call { args, .. } => {
            for s in args {
                *s = r(*s);
            }
        }
        Inst::Gate { targets, angle, .. } => {
            for s in targets {
                *s = r(*s);
            }
            if let Some(a) = angle {
                *a = r(*a);
            }
        }
        Inst::Measure { target, .. } | Inst::Release { target } => *target = r(*target),
        Inst::Run { shots, backend, .. } => {
            *shots = r(*shots);
            *backend = r(*backend);
        }
        Inst::Const { .. } | Inst::AllocQubit { .. } | Inst::AllocRegister { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gala_hir::lower_file;
    use gala_parser::parse_source;
    use gala_span::FileId;

    fn count_gates(gir: &Gir) -> usize {
        gir.funcs
            .iter()
            .flat_map(|f| f.blocks.iter())
            .flat_map(|b| b.insts.iter())
            .filter(|i| matches!(i, Inst::Gate { .. }))
            .count()
    }

    fn compile(src: &str) -> Gir {
        let out = parse_source(FileId(0), src);
        let (pkg, _) = lower_file(FileId(0), out.items);
        let (info, _) = gala_types::check_package(&pkg);
        gala_gir::lower_package(&pkg, &info).expect("lower")
    }

    #[test]
    fn adjacent_hadamards_cancel() {
        let gir = compile(
            "fn main() -> Measured<Bool> prob {\n let q = qubit()\n h(q)\n h(q)\n measure(q)\n}",
        );
        assert_eq!(count_gates(&gir), 2);
        assert_eq!(count_gates(&optimize(&gir)), 0);
    }

    #[test]
    fn adjacent_rotations_merge_and_keep_statistics() {
        let src = "fn main() -> Measured<Bool> prob {\n let q = qubit()\n h(q)\n rz(q, 0.3)\n rz(q, 0.4)\n h(q)\n measure(q)\n}";
        let gir = compile(src);
        let opt = optimize(&gir);
        assert_eq!(count_gates(&gir), 4);
        assert_eq!(count_gates(&opt), 3, "two rz become one");
        let cfg = gala_sim::SimConfig { shots: 4000, seed: 3, ..Default::default() };
        let a = gala_sim::run_entry(&gir, &cfg).unwrap();
        let b = gala_sim::run_entry(&opt, &cfg).unwrap();
        assert_eq!(a.histogram, b.histogram, "merged rotation behaves identically");
    }

    #[test]
    fn non_adjacent_gates_are_kept() {
        let gir = compile("fn main() -> Measured<Bool> prob {\n let q = qubit()\n h(q)\n x(q)\n h(q)\n measure(q)\n}");
        assert_eq!(count_gates(&optimize(&gir)), 3);
    }

    #[test]
    fn cancellation_preserves_measurement_statistics() {
        let src = "fn main() -> Measured<Bool> prob {\n let q = qubit()\n x(q)\n h(q)\n h(q)\n x(q)\n measure(q)\n}";
        let gir = compile(src);
        let opt = optimize(&gir);
        assert!(count_gates(&opt) < count_gates(&gir));
        let cfg = gala_sim::SimConfig { shots: 200, seed: 7, ..Default::default() };
        let a = gala_sim::run_entry(&gir, &cfg).unwrap();
        let b = gala_sim::run_entry(&opt, &cfg).unwrap();
        assert_eq!(a.histogram, b.histogram, "optimized program behaves identically");
        // X H H X is the identity on |0⟩, so every shot measures 0.
        assert_eq!(a.histogram.get("false"), Some(&200));
    }
}

/// Fuse runs of single-qubit gates on the same qubit.
///
/// A run of `k` gates on one qubit (with constant or no angles) has a single 2×2 unitary `U`. Up to
/// a global phase, `U = Rz(α)·Ry(β)·Rz(γ)`, so a run of four or more gates becomes three rotations.
/// Shorter runs are left alone, since three rotations would not be fewer gates.
fn fuse_single_qubit(f: &mut gala_gir::GirFunc) {
    let mut consts: HashMap<Slot, f64> = HashMap::new();
    let mut defs: HashMap<Slot, usize> = HashMap::new();
    for b in &f.blocks {
        for i in &b.insts {
            for d in defined_slots(i) {
                *defs.entry(d).or_insert(0) += 1;
            }
            if let Inst::Const { dst, value } = i {
                match value {
                    gala_gir::Const::Float(v) => {
                        consts.insert(*dst, *v);
                    }
                    gala_gir::Const::Int(v) => {
                        consts.insert(*dst, *v as f64);
                    }
                    _ => {}
                }
            }
        }
    }
    let single = |s: &Slot| defs.get(s) == Some(&1);
    let mut next_slot = f.slot_count;
    let mut rename: HashMap<Slot, Slot> = HashMap::new();
    for b in &mut f.blocks {
        let mut out: Vec<Inst> = Vec::with_capacity(b.insts.len());
        let insts = std::mem::take(&mut b.insts);
        let mut i = 0;
        while i < insts.len() {
            // A run: consecutive single-qubit gates on one slot, no other use of that slot between.
            let fusable = |inst: &Inst| -> Option<(GateKind, Slot, f64, Vec<Slot>)> {
                let Inst::Gate { gate, targets, angle, dsts } = inst else { return None };
                if gate.qubits() != 1 || targets.len() != 1 {
                    return None;
                }
                let a = match angle {
                    None => 0.0,
                    Some(s) if single(s) => *consts.get(s)?,
                    Some(_) => return None,
                };
                Some((*gate, targets[0], a, dsts.clone()))
            };
            let Some((g0, t0, a0, d0)) = fusable(&insts[i]) else {
                out.push(insts[i].clone());
                i += 1;
                continue;
            };
            let mut run = vec![(g0, a0)];
            let mut dsts_in_run = vec![d0];
            let mut consts_in_run: Vec<Inst> = Vec::new();
            let mut j = i + 1;
            while j < insts.len() {
                // Constants (such as rotation angles) do not touch the qubit: step over them.
                if let Inst::Const { .. } = &insts[j] {
                    consts_in_run.push(insts[j].clone());
                    j += 1;
                    continue;
                }
                match fusable(&insts[j]) {
                    Some((g, t, a, d)) if t == t0 => {
                        run.push((g, a));
                        dsts_in_run.push(d);
                        j += 1;
                    }
                    _ => break,
                }
            }
            if run.len() < 4 || !single(&t0) {
                out.push(insts[i].clone());
                i += 1;
                continue;
            }
            out.extend(consts_in_run);
            // Compose the run: later gates multiply on the left.
            let mut u = [[(1.0f64, 0.0f64), (0.0, 0.0)], [(0.0, 0.0), (1.0, 0.0)]];
            for (g, a) in &run {
                u = mul2(&gala_sim::gate_matrix(*g, *a), &u);
            }
            let (alpha, beta, gamma) = zyz(&u);
            let final_dst = next_slot + 6;
            let fresh = |k: u32| next_slot + k;
            let angle_slots = [fresh(0), fresh(2), fresh(4)];
            let qubit_slots = [fresh(1), fresh(3), final_dst];
            for (k, value) in [(0usize, gamma), (1, beta), (2, alpha)] {
                out.push(Inst::Const { dst: angle_slots[k], value: gala_gir::Const::Float(value) });
            }
            // Rz(γ), then Ry(β), then Rz(α) on the same qubit.
            out.push(Inst::Gate {
                gate: GateKind::Rz,
                targets: vec![t0],
                angle: Some(angle_slots[0]),
                dsts: vec![qubit_slots[0]],
            });
            out.push(Inst::Gate {
                gate: GateKind::Ry,
                targets: vec![qubit_slots[0]],
                angle: Some(angle_slots[1]),
                dsts: vec![qubit_slots[1]],
            });
            out.push(Inst::Gate {
                gate: GateKind::Rz,
                targets: vec![qubit_slots[1]],
                angle: Some(angle_slots[2]),
                dsts: vec![qubit_slots[2]],
            });
            // Every original output carries the same qubit as the fused result.
            for dsts in &dsts_in_run {
                for d in dsts {
                    rename.insert(*d, final_dst);
                }
            }
            next_slot += 7;
            i = j;
        }
        b.insts = out;
    }
    f.slot_count = next_slot;
    if rename.is_empty() {
        return;
    }
    let resolve = |mut s: Slot| {
        while let Some(&n) = rename.get(&s) {
            s = n;
        }
        s
    };
    for b in &mut f.blocks {
        for inst in &mut b.insts {
            rewrite_uses(inst, &resolve);
        }
        match &mut b.term {
            Term::Branch { cond, .. } => *cond = resolve(*cond),
            Term::Return(Some(s)) => *s = resolve(*s),
            _ => {}
        }
    }
}

type C2 = (f64, f64);

fn cmul(a: C2, b: C2) -> C2 {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}

fn cadd(a: C2, b: C2) -> C2 {
    (a.0 + b.0, a.1 + b.1)
}

/// Product `a · b` of two 2×2 complex matrices.
fn mul2(a: &[[C2; 2]; 2], b: &[[C2; 2]; 2]) -> [[C2; 2]; 2] {
    let mut out = [[(0.0, 0.0); 2]; 2];
    for (i, row) in out.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = cadd(cmul(a[i][0], b[0][j]), cmul(a[i][1], b[1][j]));
        }
    }
    out
}

/// Euler angles (α, β, γ) with `U ∝ Rz(α)·Ry(β)·Rz(γ)`, for a 2×2 unitary `U`.
///
/// Up to a global phase, `U` is `[[a, b], [-b*, a*]]` with `|a| = cos(β/2)`, `|b| = sin(β/2)`,
/// `arg a = −(α+γ)/2`, and `arg b = π − (α−γ)/2`.
fn zyz(u: &[[C2; 2]; 2]) -> (f64, f64, f64) {
    // Divide out the global phase so that det = 1 (a square root of det fixes it).
    let det = cadd(cmul(u[0][0], u[1][1]), (-cmul(u[0][1], u[1][0]).0, -cmul(u[0][1], u[1][0]).1));
    let half_arg = det.1.atan2(det.0) / 2.0;
    let w = (half_arg.cos(), -half_arg.sin());
    let a = cmul(u[0][0], w);
    let b = cmul(u[0][1], w);
    let beta = 2.0 * b.0.hypot(b.1).atan2(a.0.hypot(a.1));
    let sum = -2.0 * a.1.atan2(a.0); // α + γ
    let diff = 2.0 * (std::f64::consts::PI - b.1.atan2(b.0)); // α − γ
    ((sum + diff) / 2.0, beta, (sum - diff) / 2.0)
}

#[cfg(test)]
mod fusion_tests {
    use super::*;

    /// Rz(α)·Ry(β)·Rz(γ) as a matrix, for checking the decomposition.
    fn euler_matrix(alpha: f64, beta: f64, gamma: f64) -> [[C2; 2]; 2] {
        let rz = |t: f64| gala_sim::gate_matrix(GateKind::Rz, t);
        let ry = gala_sim::gate_matrix(GateKind::Ry, beta);
        mul2(&mul2(&rz(alpha), &ry), &rz(gamma))
    }

    /// True when two matrices are equal up to a global phase.
    fn same_up_to_phase(a: &[[C2; 2]; 2], b: &[[C2; 2]; 2]) -> bool {
        // Find the phase from the largest entry of b.
        let mut best = (0, 0);
        for i in 0..2 {
            for j in 0..2 {
                if b[i][j].0.hypot(b[i][j].1) > b[best.0][best.1].0.hypot(b[best.0][best.1].1) {
                    best = (i, j);
                }
            }
        }
        let (x, y) = (a[best.0][best.1], b[best.0][best.1]);
        let ratio = cmul(x, (y.0, -y.1));
        let norm = ratio.0.hypot(ratio.1);
        let phase = (ratio.0 / norm, ratio.1 / norm);
        (0..2).all(|i| {
            (0..2).all(|j| {
                let lhs = cmul(a[i][j], (phase.0, -phase.1));
                (lhs.0 - b[i][j].0).abs() < 1e-9 && (lhs.1 - b[i][j].1).abs() < 1e-9
            })
        })
    }

    #[test]
    fn euler_decomposition_reproduces_random_unitaries() {
        // Unitaries built from products of gates with varied angles.
        let samples: Vec<[[C2; 2]; 2]> = vec![
            gala_sim::gate_matrix(GateKind::H, 0.0),
            mul2(
                &gala_sim::gate_matrix(GateKind::S, 0.0),
                &gala_sim::gate_matrix(GateKind::T, 0.0),
            ),
            mul2(
                &gala_sim::gate_matrix(GateKind::Rx, 0.7),
                &gala_sim::gate_matrix(GateKind::Rz, -1.9),
            ),
            mul2(
                &gala_sim::gate_matrix(GateKind::Ry, 2.4),
                &gala_sim::gate_matrix(GateKind::X, 0.0),
            ),
            gala_sim::gate_matrix(GateKind::Z, 0.0),
        ];
        for u in &samples {
            let (a, b, c) = zyz(u);
            assert!(same_up_to_phase(&euler_matrix(a, b, c), u), "decomposition of {u:?}");
        }
    }

    #[test]
    fn fused_run_keeps_the_measurement_statistics() {
        // Four single-qubit gates on one qubit: fused to three rotations, same outcome statistics.
        let src = "fn main() -> Measured<Bool> prob {\n let q = qubit()\n h(q)\n s(q)\n t(q)\n ry(q, 0.7)\n measure(q)\n}";
        let out = gala_parser::parse_source(gala_span::FileId(0), src);
        let (pkg, _) = gala_hir::lower_file(gala_span::FileId(0), out.items);
        let (info, _) = gala_types::check_package(&pkg);
        let gir = gala_gir::lower_package(&pkg, &info).unwrap();
        let fused = optimize(&gir);
        let count = |g: &Gir| {
            g.funcs[0]
                .blocks
                .iter()
                .flat_map(|b| b.insts.iter())
                .filter(|i| matches!(i, Inst::Gate { .. }))
                .count()
        };
        assert_eq!(count(&gir), 4);
        assert_eq!(count(&fused), 3);
        let cfg = gala_sim::SimConfig { shots: 4000, seed: 13, ..Default::default() };
        let a = gala_sim::run_entry(&gir, &cfg).unwrap();
        let b = gala_sim::run_entry(&fused, &cfg).unwrap();
        assert_eq!(a.histogram, b.histogram);
    }
}
