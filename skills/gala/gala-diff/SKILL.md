---
name: gala-diff
description: Work with Gala's native differentiation — parameter-shift for quantum params, reverse-mode autodiff for classical, chain-rule composition
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's differentiation pass (`crates/gala-diff`) lowers `grad(f, wrt: p)` to GIR. It handles:
- **Quantum parameters** (rotation angles) → **parameter-shift rule**
- **Classical parameters** → **reverse-mode autodiff**
- **Chain rule composition** at quantum/classical boundary

> **Note on file layout:** The crate currently lives in a single `crates/gala-diff/src/lib.rs`. As it matures it may be split into `lower.rs`, `param_shift.rs`, `autodiff.rs`, `compose.rs`, `gradient.rs` (see `docs/ARCHITECTURE.md`). When in doubt, search for the relevant code in `lib.rs`.

Key files:
- `crates/gala-diff/src/lib.rs` — `grad` lowering, parameter-shift, reverse-mode autodiff, chain rule

## Common Tasks

### Add parameter-shift for new quantum gate
1. Edit `lib.rs` — add shift rule for gate's parameter
2. For `RX(θ)`, `RY(θ)`, `RZ(θ)`: shift by ±π/2, evaluate, combine
3. For controlled variants: ensure control qubit handled correctly
4. Add test in `tests/conformance/diff/` with known gradient

### Extend classical autodiff
1. Edit `lib.rs` — add reverse-mode rule for new classical op
2. Define `vjp` (vector-Jacobian product) for the op
3. Handle higher-order derivatives if needed
4. Test against finite differences

### Modify chain rule composition
1. Edit `lib.rs` — handles quantum output → classical input boundary
2. Parameter-shift produces classical gradients; autodiff consumes them
3. Ensure correct gradient flow through hybrid functions

### Add new `grad` feature (e.g., Hessian, Jacobian)
1. Extend gradient representation in `lib.rs`
2. Add lowering
3. Update `grad_matches` property test harness

## Testing

```bash
cargo test -p gala-diff                     # Unit tests
cargo test --workspace diff                 # All diff tests
cargo test --workspace conformity/diff      # Conformance suite
# Property test: grad_matches finite difference
cargo test --workspace -- grad_matches
```

## Conventions

- **Gradients are first-class GIR** — visible to optimizer, backends
- **Parameter-shift is exact** (up to floating point) for quantum params
- **Autodiff is reverse-mode** — efficient for many inputs, one output
- **Chain rule at boundary** — quantum grads feed classical autodiff
- **No runtime library** — all at compile time via GIR lowering
- Diagnostic codes: E06xx for differentiation errors

## Key Concepts

```rust
// Parameter-shift for RX(θ): d/dθ RX(θ) = 0.5 * (RX(θ+π/2) - RX(θ-π/2))
pub fn param_shift_gradient_gir(
    gir: &Gir,
    func_id: FuncId,
    param_idx: usize,
    shift: f64,
) -> Result<Gir, Diagnostics>

// Reverse-mode autodiff for classical: build adjoint computation graph
pub fn reverse_mode_autodiff_gir(
    gir: &Gir,
    source_id: FuncId,
    param_indices: &[usize],
) -> Result<(Gir, Vec<ValueId>), Diagnostics>

// Main entry point: lower all gradients for pure functions
pub fn lower_gradients(gir: &mut Gir)
```

## Debugging

```bash
# Debug gradient lowering
cargo run --bin gala -- --emit gir-diff examples/variational.gala

# Compare with finite difference
cargo run --bin gala -- run examples/variational.gala --check-grad

# Property test output
cargo test --workspace -- grad_matches -- --nocapture
```