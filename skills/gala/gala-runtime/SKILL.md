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

Key files:
- `crates/gala-runtime/src/dispatch.rs` — Backend dispatch, kernel submission
- `crates/gala-runtime/src/measurement.rs` — Mid-circuit measurement handling
- `crates/gala-runtime/src/batching.rs` — Parameter-shift batching
- `crates/gala-runtime/src/cache.rs` — Circuit submission cache
- `crates/gala-runtime/src/backend.rs` — Backend registry, capability descriptors

## Common Tasks

### Add new backend
1. Implement `Backend` trait in `backend.rs`
2. Define `BackendCapabilities` (qubits, gates, connectivity, mid-circuit, noise)
3. Implement `submit_circuit` / `submit_batch` for the backend
4. Register in backend registry
5. Add capability checking in `gala-qir` / `gala-sim`

### Improve gradient batching
1. Edit `batching.rs` — group parameter-shift evaluations
2. Single submission for all shifted circuits (major cost saver)
3. Handle result unpacking and gradient assembly
4. Benchmark: submissions/grad vs unbatched

### Add mid-circuit measurement support
1. Edit `measurement.rs` — handle `measure` in middle of circuit
2. For adaptive profile: submit measurement, get result, conditionally submit next kernel
3. For base profile: reject at compile time (handled in `gala-qir`)
4. Test: adaptive teleportation, repeat-until-success

### Implement caching strategy
1. Edit `cache.rs` — hash circuit + parameters for memoization
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
trait Backend {
    fn capabilities(&self) -> BackendCapabilities;
    fn submit(&self, circuit: Circuit, shots: u64) -> Result<MeasuredDistribution>;
    fn submit_batch(&self, circuits: Vec<Circuit>, shots: u64) -> Result<Vec<MeasuredDistribution>>;
}

struct BackendCapabilities {
    max_qubits: usize,
    native_gates: HashSet<GateName>,
    connectivity: ConnectivityGraph,
    supports_mid_circuit_measure: bool,
    supports_adaptive: bool,
    noise_model: Option<NoiseModel>,
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