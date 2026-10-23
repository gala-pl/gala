---
name: gala-sim
description: Work with Gala's simulator backends — state-vector (QuEST), noisy simulation, GIR execution
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's simulator (`crates/gala-sim`) provides in-box simulation for zero-setup development. Backed by **roqoqo + roqoqo-quest (QuEST)** for state-vector and noisy simulation. Consumes GIR directly (fast dev loop) or QIR (artifact validation).

> **Note on file layout:** The crate currently lives in a single `crates/gala-sim/src/lib.rs`. As it matures it may be split into `execute.rs`, `state_vector.rs`, `noisy.rs`, `bridge.rs`, `result.rs` (see `docs/ARCHITECTURE.md`). When in doubt, search for the relevant code in `lib.rs`.

Key files:
- `crates/gala-sim/src/lib.rs` — GIR interpreter / executor, state-vector backend, noise models

## Common Tasks

### Add new gate to simulator
1. Edit `lib.rs` — map GIR `NodeKind` to `roqoqo::operations::Operation`
2. Handle parameterized gates (RX, RY, RZ with float params)
3. Handle controlled/adjoint variants
4. Test: simulate Bell pair, verify distribution

### Add noise model
1. Edit `lib.rs` — define noise model types (depolarizing, amplitude damping, etc.)
2. Map `gala.noise` typed values to QuEST noise channels
3. Apply during circuit execution
4. Property test: noisy vs noiseless divergence within bounds

### Optimize simulation performance
1. Batch shots in `lib.rs` (QuEST supports batched execution)
2. Cache circuit compilation for repeated runs (parameter-shift)
3. Use QuEST distributed/GPU for large qubit counts
4. Benchmark: shots/sec vs qubit count

### Add new simulator backend (tensor network, etc.)
1. Implement `SimulatorBackend` trait in new module
2. Register in `gala-runtime` backend registry
3. Add capability descriptor (max qubits, supported gates)
4. Conformance: agree with state-vector where both apply

## Testing

```bash
cargo test -p gala-sim                      # Unit tests (requires QuEST)
cargo test --workspace sim                  # All sim tests
cargo test --workspace conformity/run       # End-to-end simulation tests
cargo run --bin gala -- run examples/hello_world.gala --shots 1000  # Debug run
```

## Conventions

- **Deterministic under fixed seed** — `gala run --seed 42`
- **GIR direct execution** — fast dev loop, no QIR roundtrip needed
- **Also consumes QIR** — validate exact hardware artifact
- **`Measured<T>` carries distribution** — `.sample()`, `.expectation()`, `.distribution()`
- **Noise models are typed values** — `gala.noise` module
- No `unwrap()` on user paths

## Key Types

```rust
// Simulator backend trait
pub trait SimulatorBackend {
    fn execute(&self, gir: &Gir, shots: u64, seed: u64) -> Result<SimResult, Diagnostics>;
    fn capabilities(&self) -> BackendCapabilities;
}

// State-vector result
struct SimResult {
    outcomes: HashMap<String, f64>,  // probability
    shots: u64,
    seed: u64,
}

// Noise model (typed)
enum NoiseModel {
    Depolarizing { prob: f64 },
    AmplitudeDamping { gamma: f64 },
    PhaseDamping { gamma: f64 },
    Custom(Box<dyn NoiseChannel>),
}
```

## Debugging

```bash
# Run with state-vector simulator
cargo run --bin gala -- run examples/bell.gala --shots 1000

# Run with noise model
cargo run --bin gala -- run examples/bell.gala --shots 1000 --noise depolarizing:0.01

# Debug state vector (small qubit counts)
cargo run --bin gala -- run examples/bell.gala --shots 1 --dump-state

# Deterministic seed
cargo run --bin gala -- run examples/bell.gala --shots 1000 --seed 42
```