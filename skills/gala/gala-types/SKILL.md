---
name: gala-types
description: Work with Gala's type & effect checker — bidirectional inference, effect lattice, linearity, const generics, ena unification
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's type checker (`crates/gala-types`) implements **bidirectional type inference** with:
- Hindley-Milner inference for classical fragment
- **Effect inference** over lattice `pure ⊑ quantum ⊑ prob`
- **Const-generic solving** for `Qubits<N>` sizes
- **Linearity checking** (no-cloning, no-implicit-drop, no-aliasing)
- Union-find (`ena`) for type unification

Key files:
- `crates/gala-types/src/infer.rs` — Bidirectional inference engine
- `crates/gala-types/src/effect.rs` — Effect lattice, subsumption, polymorphism
- `crates/gala-types/src/linearity.rs` — Ownership/liveness analysis
- `crates/gala-types/src/const_generic.rs` — Const generic solving for `Qubits<N>`
- `crates/gala-types/src/unify.rs` — Unification with `ena`
- `crates/gala-types/src/diagnostic.rs` — Type/effect/linearity diagnostic codes (E02xx, E03xx, E04xx)
- `crates/gala-types/src/db.rs` — Salsa queries for type checking

## Common Tasks

### Add new type / type constructor
1. Add to `TyKind` in `infer.rs` (or shared type definition)
2. Add unification case in `unify.rs`
3. Add inference rules in `infer.rs` (check/synth)
4. Add effect if quantum-relevant
5. Add const generic handling if size-parametric
6. Update linearity rules if linear
7. Add conformance tests in `tests/conformance/types/`

### Modify effect system
- Effect lattice in `effect.rs`: `pure < quantum < prob`
- Subsumption: `quantum` may call `pure`; `prob` may call `quantum`/`pure`
- Effect polymorphism: generic `fn map<E>(...) -> ... E`
- Measurement (`measure`) has effect `prob` — boundary crossing

### Fix linearity diagnostic / add new linearity rule
1. Edit `linearity.rs` ownership/liveness analysis
2. Add diagnostic code in `diagnostic.rs` (E04xx range)
3. Ensure teaching-grade `help:` text explains quantum principle
4. Test: use-after-consume, duplication, implicit-drop, aliasing

### Add const generic constraint
1. Edit `const_generic.rs` for solving `N` in `Qubits<N>`
2. Handle bounds, inference from usage
3. Add conformance tests for size mismatches

## Testing

```bash
cargo test -p gala-types                    # Unit tests
cargo test --workspace types               # All type-related tests
cargo test --workspace conformity/types    # Conformance suite
cargo run --bin gala -- --emit typed-hir examples/hello_world.gala  # Debug typed HIR
```

## Conventions

- **Bidirectional**: `check_expr(expr, expected_ty)` + `synth_expr(expr) -> Ty`
- **Effects inferred bottom-up**; annotations checked on public API
- **Linearity** = explicit ownership analysis over typed HIR (not Rust borrow checker)
- **Diagnostic codes**: E02xx types, E03xx effects, E04xx linearity
- **Every diagnostic has `--explain` entry** in `gala-diagnostics`
- No `unwrap()` on user paths

## Key Types

```rust
// Effect lattice
enum Effect { Pure, Quantum, Prob }  // Pure ⊑ Quantum ⊑ Prob

// Type kinds (simplified)
enum TyKind {
    Bool, Int, Float, Complex,
    Qubit,
    Qubits(ConstGeneric),  // Qubits<N>
    Measured(Box<Ty>),     // Measured<T>
    Fn(Vec<Ty>, Ty, Effect),
    Tuple(Vec<Ty>),
    Array(Ty, ConstGeneric),
    // ... structs, enums, type vars
}
```

## Debugging

```bash
# Debug type inference
RUST_LOG=gala_types=debug cargo run --bin gala -- examples/hello_world.gala

# Debug unification
RUST_LOG=gala_types::unify=debug cargo run --bin gala -- examples/hello_world.gala

# Debug linearity
RUST_LOG=gala_types::linearity=debug cargo run --bin gala -- examples/hello_world.gala
```