---
name: gala-runtime
description: Work with Gala's hybrid runtime — kernel dispatch, mid-circuit measurement, shot management, batching, caching
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's runtime (`crates/gala-runtime`) coordinates **hybrid program execution** at runtime:
- **Kernel dispatch** — sends quantum kernels to backend, marshals data
- **Mid-circuit measurement & feedback** — adaptive profile support
- **Shot management** — builds `Measured<T>` empirical distributions
- **Gradient batching** — batches parameter-shift evaluations
- **Caching** — memoizes identical circuit submissions

> **Note on file layout:** The crate currently lives in a single `crates/gala-runtime/src/lib.rs`. As it matures it may be split into `dispatch.rs`, `measurement.rs`, `batching.rs`, `cache.rs`, `backend.rs` (see `docs/ARCHITECTURE.md`). When in doubt, search for the relevant code in `lib.rs`.

Key files:
- `crates/gala-runtime/src/lib.rs` — Backend dispatch, runtime program/result types, simulator backend

## Common Tasks

### Add new backend
1. Implement `Backend` trait in `lib.rs`
2. Define `BackendCapabilities` (qubits, gates, connectivity, mid-circuit, noise)
3. Implement `submit_circuit` / `submit_batch` for the backend
4. Register in backend registry
5. Add capability checking in `gala-qir` / `gala-sim`

### Improve gradient batching
1. Edit `lib.rs` — group parameter-shift evaluations
2. Single submission for all shifted circuits (major cost saver)
3. Handle result unpacking and gradient assembly
4. Benchmark: submissions/grad vs unbatched

### Add mid-circuit measurement support
1. Edit `lib.rs` — handle `measure` in middle of circuit
2. For adaptive profile: submit measurement, get result, conditionally submit next kernel
3. For base profile: reject at compile time (handled in `gala-qir`)
4. Test: adaptive teleportation, repeat-until-success

### Implement caching strategy
1. Edit `lib.rs` — hash circuit + parameters for memoization
2. Invalidate on backend capability change
3. Configurable TTL / size limits
4. Test: repeated `grad` calls hit cache

## Testing

```bash
cargo test -p gala-runtime                  # Unit tests
cargo test --workspace runtime              # All runtime tests
cargo test --workspace -- bench_batching    # Batching benchmarks
cargo run --bin gala -- run examples/hybrid.gala --backend simulator  # Debug
```

## Conventions

- **Backend as capability value** — `backend.simulator()`, `backend.hardware("ionq.aria")`
- **Capability mismatch = compile error** — `E06xx` from `gala-qir` validation
- **Batching is transparent** — user writes `grad`, runtime batches
- **`Measured<T>` is empirical** — built from shots, not exact probabilities
- **Deterministic with seed** — for reproducible CI
- No `unwrap()` on user paths

## Key Traits

```rust
pub trait Backend: Send + Sync {
    fn capability(&self) -> &BackendCapability;
    fn execute(&self, program: &RuntimeProgram) -> Result<RuntimeResult, Diagnostics>;
}

struct BackendCapability {
    name: String,
    num_qubits: usize,
    native_gates: Vec<String>,
    connectivity: Vec<(usize, usize)>,
    supports_mid_circuit_measurement: bool,
    supports_reset: bool,
}

struct RuntimeProgram {
    operations: Vec<RuntimeOp>,
    num_qubits: usize,
    num_clbits: usize,
    shots: u64,
}

enum RuntimeOp {
    Gate { gate: String, qubits: Vec<usize>, params: Vec<f64> },
    Measure { qubit: usize, clbit: usize },
    Reset { qubit: usize },
    Barrier { qubits: Vec<usize> },
    Conditional { condition: RuntimeCondition, body: Vec<RuntimeOp> },
}
```

## Debugging

```bash
# Run with simulator backend
cargo run --bin gala -- run examples/bell.gala --backend simulator --shots 1000

# Run with hardware backend (requires credentials)
cargo run --bin gala -- run examples/bell.gala --backend hardware:ionq.aria --shots 1000

# Debug batching
RUST_LOG=gala_runtime::batching=debug cargo run --bin gala -- run examples/variational.gala

# Debug cache
RUST_LOG=gala_runtime::cache=debug cargo run --bin gala -- run examples/variational.gala
```