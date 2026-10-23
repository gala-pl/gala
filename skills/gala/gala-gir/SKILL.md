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

> **Note on file layout:** The crate currently lives in a single `crates/gala-gir/src/lib.rs`. As it matures it may be split into `gir.rs`, `lower.rs`, `serialize.rs`, `validate.rs`, `builder.rs` (see `docs/ARCHITECTURE.md`). When in doubt, search for the relevant code in `lib.rs`.

Key files:
- `crates/gala-gir/src/lib.rs` — GIR data model, lowering, serialization

## Common Tasks

### Add new GIR operation
1. Add `NodeKind` variant in `lib.rs`
2. Add type/effect signature in lowering rules
3. Add validation in debug builds (linearity, effects)
4. Update all backends: `gala-qir`, `gala-sim`, `gala-codegen-classical`, `gala-diff`
5. Add serialization support

### Modify GIR lowering
1. Edit `lower_hir_to_gir` in `lib.rs` — pattern match on typed HIR, emit GIR ops
2. Handle effect annotations correctly (`pure`/`quantum`/`prob`)
3. Preserve linearity: each linear value used exactly once in GIR
4. Test with `--emit gir` and `--emit gir=json`

### Add GIR optimization pass (or modify existing)
1. Passes live in `gala-opt` but consume/produce GIR
2. Use `Gir` APIs for transformations
3. Run validation after pass in debug builds
4. Property test: semantic equivalence on simulator

### Extend GIR serialization
1. Update `GirSchema` (versioned: `gir.v1.json`)
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
struct GirFunc {
    id: FuncId,
    name: Ident,
    params: Vec<ValueId>,
    param_tys: Vec<TypeId>,
    ret_ty: Option<TypeId>,
    effect: Effect,
    blocks: HashMap<BlockId, Block>,
    entry_block: BlockId,
    span: Span,
}

// GIR block (basic block)
struct Block {
    id: BlockId,
    nodes: Vec<NodeId>,
    terminator: Option<Terminator>,
    span: Span,
}

// GIR operation
struct GirNode {
    id: NodeId,
    kind: NodeKind,
    operands: Vec<ValueId>,
    results: Vec<ValueId>,
    span: Span,
}

// Node kinds (quantum + classical)
enum NodeKind {
    Gate(GateKind),
    Classical(ClassicalOp),
    Alloc(AllocKind),
    Free { target: ValueId },
    // ...
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