---
name: gala-std
description: Work with Gala's standard library — intrinsics, gates, algorithms, VQA, noise, hardware modules
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's standard library (`crates/gala-std` + `library/std/`) provides **compiler intrinsics** and **Gala-source modules** for quantum computing:

- `gala.core` — intrinsics: `qubit()`, `measure`, `drop`, `reverse`, `adjoint`, `control`, `grad`
- `gala.gates` — standard gates with compiler-derived adjoint/controlled
- `gala.algorithms` — QFT, Grover, phase estimation, adders, etc.
- `gala.vqa` — encoders, ansatz zoo, optimizers, gradient utilities
- `gala.noise` — typed noise models
- `gala.hardware` — backend capability descriptors

Key files:
- `crates/gala-std/src/lib.rs` — Rust-side intrinsics, compiler hooks
- `library/std/core.gala` — Core intrinsics (compiler-known)
- `library/std/gates.gala` — Gate definitions
- `library/std/algorithms.gala` — Algorithm implementations
- `library/std/vqa.gala` — VQA components
- `library/std/noise.gala` — Noise models
- `library/std/hardware.gala` — Hardware backend descriptors
- `library/std/classical.gala` — Classical standard library

## Common Tasks

### Add new gate intrinsic
1. Edit `library/std/gates.gala` with `quantum` effect
2. Register in `gala-std` compiler intrinsics (for codegen/sim)
3. Compiler auto-derives `adjoint`/`controlled` — verify
4. Property test: `#[property(unitary)]` on gate
5. Add to `extensions/README.md` keyword list

### Add algorithm to stdlib
1. Edit or create file in `library/std/`
2. Use only `gala.core` + `gala.gates` (no external deps)
3. Add `#[property(reversible)]` / `#[property(uncomputes)]` if quantum
4. Test on simulator: `gala run library/std/algorithms.gala`
5. Document in `docs/STANDARD_LIBRARY.md`

### Add VQA component
1. Edit `library/std/vqa.gala` — encoders, ansatz, optimizers
2. Ensure `grad` works end-to-end (parameter-shift + autodiff)
3. Benchmark: gradient cost (submissions/grad)
4. Property test: `#[property(grad_matches)]`

### Define noise model
1. Edit `library/std/noise.gala` — typed noise model structs
2. Map to QuEST noise channels in `gala-sim`
3. Test: noisy vs noiseless simulation divergence

## Testing

```bash
cargo test -p gala-std                      # Unit tests (Rust intrinsics)
cargo test --workspace stdlib               # All stdlib tests
# Property tests on Gala source
cargo test --workspace -- property/stdlib
# Run stdlib algorithms on simulator
cargo run --bin gala -- run library/std/algorithms.gala
```

## Conventions

- **Intrinsics in Rust** (`gala-std`) — compiler-known, zero-overhead
- **Algorithms in Gala** (`library/std/`) — user-extensible, property-tested
- **Compiler derives adjoint/controlled** — no hand-written inverses
- **Effects explicit** — `quantum` for unitary, `prob` for measurement
- **Linear types enforced** — qubits used exactly once
- No `unwrap()` on user paths

## Key Modules

```gala
// gala.core — compiler intrinsics
qubit() -> Qubit quantum
qubits<N>() -> Qubits<N> quantum
measure(q: Qubit) -> Measured<Bool> prob
drop(q: Qubit) quantum
reverse(f) / adjoint(f)  // derived
control(f)               // derived
grad(f, wrt: p)          // native

// gala.gates — standard gates (all quantum)
h, x, y, z, s, t
rx, ry, rz
cx, cz, swap

// gala.algorithms — property-tested algorithms
qft, iqft, grover, phase_estimation
ripple_adder, qft_adder

// gala.vqa — variational quantum algorithms
angle_encode, amplitude_encode
layered_ansatz, hardware_efficient_ansatz
gd_optimizer, adam_optimizer, spsa_optimizer
```

## Debugging

```bash
# Check stdlib compiles
cargo run --bin gala -- check library/std/

# Run algorithm on simulator
cargo run --bin gala -- run library/std/algorithms.gala --shots 1000

# Test property
cargo run --bin gala -- test library/std/gates.gala --property unitary

# Format stdlib
cargo run --bin gala -- fmt library/std/
```