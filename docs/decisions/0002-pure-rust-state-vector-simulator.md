# ADR 0002: Pure-Rust state-vector simulator as the default backend

- Status: Accepted
- Date: 2026-10-22

## Context

The plan names `roqoqo` and `roqoqo-quest` for simulation. `roqoqo-quest` links the QuEST C
library, which needs a C toolchain and a longer build. Every contributor, and every CI job that
runs the conformance suite, would need that toolchain just to run `gala run`.

## Decision

- `crates/gala-sim` is a pure-Rust state-vector simulator. It runs a GIR entry point with
  `SimConfig` (shots, seed, depolarizing noise) and returns a `RunReport`. Gradient support
  (`ShiftPlan`, `exact_probabilities`) is built on the same simulator.
- `roqoqo-quest` is not a dependency of any crate. It is left as a future optional feature behind
  the simulator interface, to be added only through a new ADR.

## Consequences

- `cargo test --workspace` and the conformance `run/` cases need no system libraries.
- Simulation is exponential in qubit count. Capacity is enforced with diagnostic `E0702`
  (simulator capacity exceeded): a 40-qubit register reports "needs more than 24 live qubits" in
  the current build, so the failure is a diagnostic and not a crash.
- Results from the Rust simulator are not yet cross-checked against QuEST. That comparison is
  the first task of the optional feature, if it is added.
