---
name: gala-hir
description: Work with Gala's HIR — desugaring, name resolution, module graph, DefId management, salsa queries
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's HIR (`crates/gala-hir`) is the **High-Level Intermediate Representation**: desugared, name-resolved, with a module graph and stable `DefId`s. Built on **salsa** for incremental compilation — shared by batch compiler and LSP.

> **Note on file layout:** The crate currently lives in a single `crates/gala-hir/src/lib.rs`. As it matures it may be split into `hir.rs`, `lower.rs`, `resolve.rs`, `def_id.rs`, `db.rs`, `module.rs` (see `docs/ARCHITECTURE.md`). When in doubt, search for the relevant code in `lib.rs`.

Key files:
- `crates/gala-hir/src/lib.rs` — HIR node definitions, desugaring, name resolution, module graph

## Common Tasks

### Add new desugaring
1. Identify surface syntax in AST (e.g., in-place gate sugar `h(q)` → `let q = h(q)`)
2. Add lowering case in `lib.rs` `Desugarer::desugar_expr` / `desugar_stmt`
3. Ensure `DefId` assignment for new bindings
4. Add conformance test in `tests/conformance/hir/` or relevant category
5. Verify LSP sees desugared form (hover shows explicit form)

### Modify name resolution / module system
1. Edit `Resolver` in `lib.rs` for resolution logic (scope, imports, shadowing)
2. Update `ModuleGraph` construction in `lib.rs`
3. Update `DefId` allocation if new definition kinds added
4. Test cross-file resolution, re-exports, glob imports

### Add salsa query
1. Define query in `crates/gala-driver/src/db.rs` with `#[salsa::query]`
2. Implement in appropriate module
3. Use `db.query_name(input)` in consumers (type checker, LSP)
4. Test incremental recomputation: change source, verify only affected queries rerun

### Add new definition kind (e.g., new item type)
1. Add `HirItem` variant in `lib.rs`
2. Add `DefId` construction in `Desugarer`
3. Add resolution entry in `Resolver`
4. Update module graph

## Testing

```bash
cargo test -p gala-hir                      # Unit tests
cargo test --workspace hir                  # All HIR-related tests
cargo run --bin gala -- --emit hir examples/hello_world.gala  # Debug HIR output
```

## Conventions

- **No mutation of source** — HIR is pure function of AST + salsa DB
- **All queries memoized** — salsa handles invalidation on source change
- **DefId stability** — same source location = same DefId across edits (for LSP)
- **Desugaring is explicit** — in-place sugar becomes explicit rebindings
- **Error recovery** — resolver produces best-effort resolution + diagnostics
- No `unwrap()` on user paths — return `Result` or emit diagnostics via `gala-diagnostics`

## Key Salsa Queries

```rust
// In crates/gala-driver/src/db.rs
#[salsa::tracked]
fn hir_file(db, file: SourceFile) -> Arc<HirFile>;

#[salsa::tracked]
fn hir_file_resolved(db, file: SourceFile) -> Arc<HirFile>;

#[salsa::tracked]
fn hir_functions(db, file: SourceFile) -> Arc<HashMap<DefId, HirFnDef>>;
```

## Debugging

```bash
# Debug HIR for a file
cargo run --bin gala -- --emit hir examples/hello_world.gala

# Debug module graph
cargo run --bin gala -- --emit module-graph examples/

# Salsa debug (query execution trace)
RUST_LOG=gala_hir=debug cargo run --bin gala -- examples/hello_world.gala
```