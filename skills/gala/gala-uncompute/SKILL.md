---
name: gala-uncompute
description: Work with Gala's uncomputation analysis — provenance tracking, liftability, adjoint synthesis, E0530 refusal
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's uncomputation pass (`crates/gala-uncompute`) ensures **safe automatic uncomputation** of ancilla qubits. It tracks **provenance** (reversible op-graph) for each linear value and synthesizes `adjoint(provenance)` on `drop`/scope exit. If not liftable, emits `E0530` with blocking reason.

Key files:
- `crates/gala-uncompute/src/provenance.rs` — Provenance graph construction per value
- `crates/gala-uncompute/src/liftability.rs` — Liftability analysis (reversible? inputs live?)
- `crates/gala-uncompute/src/synthesis.rs` — Adjoint synthesis and insertion
- `crates/gala-uncompute/src/diagnostic.rs` — E05xx diagnostic codes
- `crates/gala-uncompute/src/db.rs` — Salsa queries

## Common Tasks

### Modify liftability analysis
1. Edit `liftability.rs` — a value is **liftable** iff:
   - Provenance contains only reversible (unitary) operations
   - All inputs to provenance are still live at drop point
   - No measurement or irreversible classical dependency in provenance
2. Add new blocking reason to `LiftabilityError` enum
3. Update `E0530` diagnostic with new `help:` text

### Extend provenance tracking
1. Edit `provenance.rs` to track additional operation types
2. Handle new gate intrinsics, controlled/adjoint variants
3. Ensure provenance composes correctly for function calls

### Improve adjoint synthesis
1. Edit `synthesis.rs` — generates `adjoint(provenance)` as GIR nodes
2. Handle controlled/adjoint of synthesized uncomputation
3. Verify synthesized uncompute restores ancillae to |0⟩

### Add new refusal case (E0530 sub-case)
1. Add variant to `UncomputeRefusalReason` in `diagnostic.rs`
2. Detect in `liftability.rs`
3. Emit diagnostic with specific `help:` explaining why and workaround (`uncompute_with(...)`)

## Testing

```bash
cargo test -p gala-uncompute                # Unit tests
cargo test --workspace uncompute           # All uncompute tests
cargo test --workspace conformity/uncompute # Conformance: success + refusal cases
cargo run --bin gala -- --emit uncompute-plan examples/hello_world.gala  # Debug plans
```

## Conventions

- **Never silently produce wrong program** — refuse with E0530 if uncertain
- **Provenance is per-value** — tracks exact reversible ops that produced it
- **Synthesis inserts at scope exit** — before variable goes out of scope
- **Diagnostic codes**: E0530 (cannot auto-uncompute), E0531 (non-liftable input), E0532 (measurement dependency)
- **Teaching diagnostics** — explain physical principle (leaked entanglement)
- No `unwrap()` on user paths

## Key Concepts

```rust
// Provenance: reversible op-graph producing a value
struct Provenance {
    nodes: Vec<ProvenanceNode>,  // unitary ops with input/output value refs
    output: ValueId,
}

// Liftability result
enum Liftability {
    Liftable(Provenance),
    NotLiftable(UncomputeRefusalReason),
}

// Refusal reasons
enum UncomputeRefusalReason {
    MeasurementDependency,    // provenance depends on measure()
    IrreversibleInput,        // depends on classical non-reversible input
    InputNotLive,             // provenance input already consumed
    NonUnitaryOp,             // provenance contains non-reversible op
    // ...
}
```

## Debugging

```bash
# Debug uncompute plans
cargo run --bin gala -- --emit uncompute-plan examples/hello_world.gala

# Debug provenance graph
RUST_LOG=gala_uncompute=debug cargo run --bin gala -- examples/hello_world.gala

# Test on simulator (once gala-sim exists) — verify ancillae return to |0⟩
cargo run --bin gala -- run examples/hello_world.gala --shots 1000
```