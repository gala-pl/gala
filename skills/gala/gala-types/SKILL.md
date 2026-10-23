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

> **Note on file layout:** The crate currently lives in a single `crates/gala-types/src/lib.rs`. As it matures it may be split into `infer.rs`, `effect.rs`, `linearity.rs`, `const_generic.rs`, `unify.rs`, `diagnostic.rs`, `db.rs` (see `docs/ARCHITECTURE.md`). When in doubt, search for the relevant code in `lib.rs`.

Key files:
- `crates/gala-types/src/lib.rs` — Type inference, effect checking, linearity checking, unification

## Common Tasks

### Add new type / type constructor
1. Add to `Ty` enum in `lib.rs`
2. Add unification case in `unify_types`
3. Add inference rules in `check_hir_expr`
4. Add effect if quantum-relevant
5. Add const generic handling if size-parametric
6. Update linearity rules if linear
7. Add conformance tests in `tests/conformance/types/`

### Modify effect system
- Effect lattice in `Effect` enum: `Pure < Quantum < Prob`
- Subsumption: `quantum` may call `pure`; `prob` may call `quantum`/`pure`
- Effect polymorphism: generic `fn map<E>(...) -> ... E`
- Measurement (`measure`) has effect `prob` — boundary crossing

### Fix linearity diagnostic / add new linearity rule
1. Edit linearity checking in `lib.rs` (`check_linearity`, `check_block_linearity`)
2. Add diagnostic code in `crates/gala-diagnostics/src/lib.rs` (E04xx range)
3. Ensure teaching-grade `help:` text explains quantum principle
4. Test: use-after-consume, duplication, implicit-drop, aliasing

### Add const generic constraint
1. Edit `ast_type_to_ty` for solving `N` in `Qubits<N>`
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
enum Ty {
    Bool, Int, Float, Complex, String, Unit, Params,
    Qubit,
    Qubits(u64),  // Qubits<N>
    Measured(Box<Ty>),     // Measured<T>
    Tuple(Vec<Ty>),
    Array(Box<Ty>, u64),
    Fn { params: Vec<Ty>, ret: Box<Ty>, effect: Effect },
    Named(DefId, Vec<Ty>),
    Var(TyVar),
    TyError,
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