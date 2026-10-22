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
    }
    out
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
        | Inst::Measure { dst, .. } => vec![*dst],
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
