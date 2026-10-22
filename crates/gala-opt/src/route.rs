//! Target lowering: map a logical circuit onto a device's connectivity.
//!
//! The logical model lets any two qubits interact. A device has a coupling graph: two-qubit gates
//! are only native between adjacent physical qubits. Routing inserts SWAP gates so that every
//! two-qubit gate acts on an edge. The routed circuit computes the same distribution, up to the
//! final qubit placement, which [`verify_routing`] checks exactly on the simulator.

use gala_gir::GateKind;
use std::collections::VecDeque;

/// One operation of a circuit over qubit indices: gate, operands, angle.
pub type Op = (GateKind, Vec<usize>, f64);

/// A device's qubit connectivity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Topology {
    /// Number of physical qubits.
    pub qubits: usize,
    /// Undirected couplings between physical qubits.
    pub edges: Vec<(usize, usize)>,
}

impl Topology {
    /// A line: qubit `i` couples to `i + 1`.
    pub fn line(n: usize) -> Self {
        Topology { qubits: n, edges: (1..n).map(|i| (i - 1, i)).collect() }
    }

    /// A ring: a line that also couples the last qubit to the first.
    pub fn ring(n: usize) -> Self {
        let mut t = Topology::line(n);
        if n > 2 {
            t.edges.push((n - 1, 0));
        }
        t
    }

    fn adjacent(&self, a: usize, b: usize) -> bool {
        self.edges.iter().any(|&(x, y)| (x == a && y == b) || (x == b && y == a))
    }

    /// Shortest path between two physical qubits (inclusive), by breadth-first search.
    fn path(&self, from: usize, to: usize) -> Option<Vec<usize>> {
        let mut prev = vec![None; self.qubits];
        let mut seen = vec![false; self.qubits];
        let mut queue = VecDeque::from([from]);
        seen[from] = true;
        while let Some(u) = queue.pop_front() {
            if u == to {
                let mut path = vec![to];
                let mut cur = to;
                while let Some(p) = prev[cur] {
                    path.push(p);
                    cur = p;
                }
                path.reverse();
                return Some(path);
            }
            for &(a, b) in &self.edges {
                let next = if a == u {
                    b
                } else if b == u {
                    a
                } else {
                    continue;
                };
                if !seen[next] {
                    seen[next] = true;
                    prev[next] = Some(u);
                    queue.push_back(next);
                }
            }
        }
        None
    }
}

/// A routed circuit on physical qubits.
#[derive(Debug, Clone, PartialEq)]
pub struct Routed {
    /// Operations on physical qubits, SWAPs included.
    pub ops: Vec<Op>,
    /// Number of SWAP gates inserted.
    pub swaps: usize,
    /// Circuit depth: operations per layer, where a layer holds gates on disjoint qubits.
    pub depth: usize,
    /// `final_layout[logical] = physical` after the whole circuit.
    pub final_layout: Vec<usize>,
}

/// Route a logical circuit onto `topology`, starting from the identity placement.
///
/// Returns `None` when the topology is disconnected between two interacting qubits.
pub fn route(ops: &[Op], topology: &Topology) -> Option<Routed> {
    let n = topology.qubits;
    // layout[logical] = physical; the identity to start.
    let mut layout: Vec<usize> = (0..n).collect();
    let mut out: Vec<Op> = Vec::new();
    let mut swaps = 0usize;
    for (gate, qs, angle) in ops {
        if gate.qubits() == 2 {
            let (a, b) = (qs[0], qs[1]);
            let path = topology.path(layout[a], layout[b])?;
            // Walk qubit `a` along the path until it is adjacent to `b`.
            for step in path.windows(2).take(path.len().saturating_sub(2)) {
                let (p, q) = (step[0], step[1]);
                out.push((GateKind::Swap, vec![p, q], 0.0));
                swaps += 1;
                // Whichever logical qubits sit on p and q exchange places.
                for slot in layout.iter_mut() {
                    if *slot == p {
                        *slot = q;
                    } else if *slot == q {
                        *slot = p;
                    }
                }
            }
            debug_assert!(topology.adjacent(layout[a], layout[b]));
        }
        let phys: Vec<usize> = qs.iter().map(|&l| layout[l]).collect();
        out.push((*gate, phys, *angle));
    }
    let depth = layers(&out, n);
    Some(Routed { ops: out, swaps, depth, final_layout: layout })
}

/// Number of layers when each operation is placed as early as its qubits allow.
pub fn layers(ops: &[Op], qubits: usize) -> usize {
    let mut ready = vec![0usize; qubits];
    let mut depth = 0;
    for (_, qs, _) in ops {
        let start = qs.iter().map(|&q| ready[q]).max().unwrap_or(0);
        for &q in qs {
            ready[q] = start + 1;
        }
        depth = depth.max(start + 1);
    }
    depth
}

/// Rewrite SWAP as three CNOTs, for targets whose native two-qubit gate is CX.
pub fn decompose_swaps_to_cx(ops: &[Op]) -> Vec<Op> {
    let mut out = Vec::with_capacity(ops.len());
    for (gate, qs, angle) in ops {
        if *gate == GateKind::Swap {
            let (a, b) = (qs[0], qs[1]);
            out.push((GateKind::Cx, vec![a, b], 0.0));
            out.push((GateKind::Cx, vec![b, a], 0.0));
            out.push((GateKind::Cx, vec![a, b], 0.0));
        } else {
            out.push((*gate, qs.clone(), *angle));
        }
    }
    out
}

/// Convert to the simulator's operand type.
fn to_sim(ops: &[Op]) -> Vec<(GateKind, Vec<u32>, f64)> {
    ops.iter().map(|(g, qs, a)| (*g, qs.iter().map(|&q| q as u32).collect(), *a)).collect()
}

/// Check that a routed circuit computes the same distribution as the logical one.
///
/// The logical distribution is over logical qubits. The routed distribution is over physical
/// qubits, read through the final layout.
pub fn verify_routing(logical: &[Op], n: usize, routed: &Routed) -> bool {
    let Ok(want) = gala_sim::exact_probabilities(n, &to_sim(logical)) else { return false };
    let Ok(got) = gala_sim::exact_probabilities(n, &to_sim(&routed.ops)) else { return false };
    // Map each physical basis index back to its logical index.
    let mut mapped = vec![0.0; want.len()];
    for (phys_index, p) in got.iter().enumerate() {
        let mut logical_index = 0usize;
        for (l, &ph) in routed.final_layout.iter().enumerate() {
            if phys_index & (1 << ph) != 0 {
                logical_index |= 1 << l;
            }
        }
        mapped[logical_index] += p;
    }
    want.iter().zip(mapped.iter()).all(|(a, b)| (a - b).abs() < 1e-9)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cx(a: usize, b: usize) -> Op {
        (GateKind::Cx, vec![a, b], 0.0)
    }

    #[test]
    fn adjacent_gates_need_no_swaps() {
        let ops = vec![(GateKind::H, vec![0], 0.0), cx(0, 1), cx(1, 2)];
        let r = route(&ops, &Topology::line(3)).unwrap();
        assert_eq!(r.swaps, 0);
        assert!(verify_routing(&ops, 3, &r));
    }

    #[test]
    fn distant_gates_are_routed_with_swaps_and_stay_equivalent() {
        // On a line of 4, CX(0,3) needs its endpoints moved together.
        let ops = vec![(GateKind::H, vec![0], 0.0), cx(0, 3), (GateKind::X, vec![3], 0.0)];
        let r = route(&ops, &Topology::line(4)).unwrap();
        assert!(r.swaps >= 1, "expected swaps, got {}", r.swaps);
        assert!(verify_routing(&ops, 4, &r));
    }

    #[test]
    fn ring_routes_no_further_than_line() {
        let ops = vec![cx(0, 4), cx(1, 3)];
        let line = route(&ops, &Topology::line(5)).unwrap();
        let ring = route(&ops, &Topology::ring(5)).unwrap();
        assert!(ring.swaps <= line.swaps);
        assert!(verify_routing(&ops, 5, &ring));
    }

    #[test]
    fn swap_decomposes_to_three_cnots_with_same_distribution() {
        let ops = vec![(GateKind::X, vec![0], 0.0), (GateKind::Swap, vec![0, 1], 0.0)];
        let decomposed = decompose_swaps_to_cx(&ops);
        assert_eq!(decomposed.iter().filter(|o| o.0 == GateKind::Cx).count(), 3);
        let a = gala_sim::exact_probabilities(2, &to_sim(&ops)).unwrap();
        let b = gala_sim::exact_probabilities(2, &to_sim(&decomposed)).unwrap();
        assert!(a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < 1e-9));
    }

    #[test]
    fn depth_counts_parallel_layers() {
        let ops = vec![(GateKind::H, vec![0], 0.0), (GateKind::H, vec![1], 0.0), cx(0, 1)];
        assert_eq!(layers(&ops, 2), 2);
    }
}
