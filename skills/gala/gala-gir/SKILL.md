---
name: gala-gir
description: Work with Gala's GIR — typed effect-annotated op-graph, petgraph-based IR, lowering from HIR, serialization
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's GIR (`crates/gala-gir`) is the **Gala Intermediate Representation**: a typed, effect-annotated operation graph where nodes are classical or quantum operations, edges carry linear-value ownership and classical data dependencies. It is the **stable contract** for all backends.

Key files:
- `crates/gala-gir/src/gir.rs` — GIR data model: `Gir`, `Op`, `Value`, `Block`, `Func`
- `crates/gala-gir/src/lower.rs` — Typed HIR → GIR lowering
- `crates/gala-gir/src/serialize.rs` — JSON serialization (versioned schema)
- `crates/gala-gir/src/validate.rs` — GIR invariants (linearity, effect consistency)
- `crates/gala-gir/src/builder.rs` — GIR construction API for passes

## Common Tasks

### Add new GIR operation
1. Add `OpKind` variant in `gir.rs`
2. Add type/effect signature in `lower.rs` lowering rules
3. Add validation in `validate.rs` (linearity, effects)
4. Update all backends: `gala-qir`, `gala-sim`, `gala-codegen-classical`, `gala-diff`
5. Add serialization support in `serialize.rs`
6. Add conformance test in `tests/conformance/gir/`

### Modify GIR lowering
1. Edit `lower.rs` — pattern match on typed HIR, emit GIR ops
2. Handle effect annotations correctly (`pure`/`quantum`/`prob`)
3. Preserve linearity: each linear value used exactly once in GIR
4. Test with `--emit gir` and `--emit gir=json`

### Add GIR optimization pass (or modify existing)
1. Passes live in `gala-opt` but consume/produce GIR
2. Use `Gir::map_ops` or `Gir::map_blocks` for transformations
3. Run `validate.rs` after pass in debug builds
4. Property test: semantic equivalence on simulator

### Extend GIR serialization
1. Update `GirSchema` in `serialize.rs` (versioned: `gir.v1.json`)
2. Maintain backward compatibility
3. Test round-trip: `--emit gir=json --gir-version 1` → `--from-gir`

## Testing

```bash
cargo test -p gala-gir                      # Unit tests
cargo test --workspace gir                  # All GIR tests
cargo run --bin gala -- --emit gir examples/hello_world.gala  # Debug GIR
cargo run --bin gala -- --emit gir=json --gir-version 1 examples/hello_world.gala  # JSON
```

## Conventions

- **GIR is the only thing backends see** — no frontend imports in backends
- **Typed + effect-annotated** — every Value has Ty + Effect
- **Linear values** — ownership edges explicit in graph
- **Serializable** — versioned JSON schema for tooling/agents
- **Dumpable** — `gala build --emit gir` for inspection
- No `unwrap()` on user paths

## Key Types

```rust
// GIR function
struct Func {
    name: Symbol,
    params: Vec<Value>,      // typed, with ownership
    ret_ty: Ty,
    effect: Effect,
    body: Block,
}

// GIR block (basic block)
struct Block {
    ops: Vec<Op>,
    terminator: Terminator,
}

// GIR operation
struct Op {
    kind: OpKind,
    inputs: Vec<Value>,      // data dependencies
    outputs: Vec<Value>,     // results (linear or classical)
    effect: Effect,          // effect of this op
    span: Span,              // source location
}

// Op kinds (quantum + classical)
enum OpKind {
    // Quantum
    AllocQubit, AllocQubits(ConstGeneric),
    H, X, Y, Z, S, T, RX, RY, RZ, CX, CZ, SWAP,
    Measure,
    Adjoint(Box<Func>), Control(Box<Func>),
    // Classical
    Const, Add, Sub, Mul, Div, Cmp, Select,
    Call(FuncId), Return,
    // Control flow
    Branch, Loop,
}
```

## Debugging

```bash
# Dump GIR as text
cargo run --bin gala -- --emit gir examples/hello_world.gala

# Dump GIR as JSON (for tools/agents)
cargo run --bin gala -- --emit gir=json --gir-version 1 examples/hello_world.gala

# Validate GIR invariants
cargo run --bin gala -- --validate-gir examples/hello_world.gala
```