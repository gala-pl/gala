---
name: gala-opt
description: Work with Gala's GIR optimizer — quantum gate fusion, cancellation, rotation merge, classical DCE/inlining
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's optimizer (`crates/gala-opt`) performs **GIR → GIR rewrites** preserving effect annotations and linearity invariants:
- **Quantum**: gate fusion, adjacent cancellation, rotation merging, ancilla scheduling, depth/count reduction
- **Classical**: constant folding, DCE, inlining of small `pure` functions

> **Note on file layout:** The crate currently lives in a single `crates/gala-opt/src/lib.rs`. As it matures it may be split into `pass.rs`, `quantum.rs`, `classical.rs`, `validate.rs` (see `docs/ARCHITECTURE.md`). When in doubt, search for the relevant code in `lib.rs`.

Key files:
- `crates/gala-opt/src/lib.rs` — Pass infrastructure, optimizer entry point

## Common Tasks

### Add quantum optimization pass
1. Edit `lib.rs` — implement pass as function `Gir -> Gir`
2. Pattern match on `NodeKind` sequences (e.g., `H; H` → cancel)
3. For rotation merge: `RX(a); RX(b)` → `RX(a+b)`
4. For gate fusion: adjacent single-qubit → single fused op
5. Register in pass pipeline
6. Add benchmarks: gate count/depth on standard algorithms

### Add classical optimization pass
1. Edit `lib.rs` — constant folding, DCE, inlining
2. Use effect info: only inline `pure` functions
3. Preserve linearity: don't duplicate linear values
4. Register in pass pipeline

### Modify pass ordering / dependencies
1. Edit `lib.rs` — `optimize_gir` defines pipeline
2. Quantum passes typically before classical (quantum structure informs classical)
3. Some passes may need multiple runs (fixpoint)

### Debug optimization correctness
1. Enable validation checks in debug builds (`cfg(debug_assertions)`)
2. Property test: semantic equivalence on simulator (`gala-sim`)
3. Compare GIR before/after with `--emit gir`

## Testing

```bash
cargo test -p gala-opt                      # Unit tests
cargo test --workspace opt                  # All opt tests
cargo test --workspace -- bench_gate_count  # Gate count benchmarks
cargo test --workspace -- bench_depth       # Depth benchmarks
# Property test: optimized GIR == original on simulator
cargo test --workspace -- property/optimizer
```

## Conventions

- **Hardware-agnostic** — no device-specific passes here (see target lowering)
- **Preserve invariants** — linearity, effects checked in debug builds
- **Dumpable** — `--emit gir-opt` for inspection
- **Composable** — passes are pure GIR → GIR functions
- No `unwrap()` on user paths

## Key Passes

```rust
// Quantum passes (in typical order)
1. AdjacentCancellation    // H;H → I, X;X → I, etc.
2. RotationMerge           // RX(a);RX(b) → RX(a+b)
3. SingleQubitFusion       // U1;U2;U3 → fused U
4. AncillaScheduling       // Reorder to minimize live ancillae
5. TwoQubitCancellation    // CX(a,b);CX(a,b) → I

// Classical passes
1. ConstantFolding         // 2+3 → 5
2. DeadCodeElimination     // Remove unused pure computations
3. InlineSmallPure         // Inline small pure functions
```

## Debugging

```bash
# Dump GIR before/after optimization
cargo run --bin gala -- --emit gir examples/hello_world.gala
cargo run --bin gala -- --emit gir-opt examples/hello_world.gala

# Run specific pass
cargo run --bin gala -- --opt-pass=RotationMerge examples/hello_world.gala

# Validate invariants
cargo run --bin gala -- --validate-gir examples/hello_world.gala
```