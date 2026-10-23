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

> **Note on file layout:** The crate currently lives in a single `crates/gala-uncompute/src/lib.rs`. As it matures it may be split into `provenance.rs`, `liftability.rs`, `synthesis.rs`, `diagnostic.rs`, `db.rs` (see `docs/ARCHITECTURE.md`). When in doubt, search for the relevant code in `lib.rs`.

Key files:
- `crates/gala-uncompute/src/lib.rs` — Provenance tracking, liftability analysis, adjoint synthesis

## Common Tasks

### Modify liftability analysis
1. Edit `lib.rs` — a value is **liftable** iff:
   - Provenance contains only reversible (unitary) operations
   - All inputs to provenance are still live at drop point
   - No measurement or irreversible classical dependency in provenance
2. Add new blocking reason to refusal enum
3. Update diagnostic with new `help:` text

### Extend provenance tracking
1. Edit `lib.rs` to track additional operation types
2. Handle new gate intrinsics, controlled/adjoint variants
3. Ensure provenance composes correctly for function calls

### Improve adjoint synthesis
1. Edit `lib.rs` — generates `adjoint(provenance)` as GIR nodes
2. Handle controlled/adjoint of synthesized uncomputation
3. Verify synthesized uncompute restores ancillae to |0⟩

### Add new refusal case (E0530 sub-case)
1. Add variant to refusal enum in `lib.rs`
2. Detect in liftability analysis
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
    nodes: Vec<ProvenanceNode>,
}

enum ProvenanceNode {
    Gate { gate: Ident, inputs: Vec<DefId> },
    Alloc { ty: Ty },
    Measure,
    External,
}

// Uncompute plan
struct UncomputePlan {
    steps: Vec<UncomputeStep>,
}

enum UncomputeStep {
    Adjoint { target: DefId, provenance: Provenance },
    Free { target: DefId },
    Measure { target: DefId },
}
```

## Debugging

```bash
# Debug uncompute plans
cargo run --bin gala -- --emit uncompute-plan examples/hello_world.gala

# Debug provenance graph
RUST_LOG=gala_uncompute=debug cargo run --bin gala -- examples/hello_world.gala

# Test on simulator — verify ancillae return to |0⟩
cargo run --bin gala -- run examples/hello_world.gala --shots 1000
```