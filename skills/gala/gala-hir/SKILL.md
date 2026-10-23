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

Key files:
- `crates/gala-hir/src/hir.rs` — HIR node definitions (desugared Expr, Stmt, Item)
- `crates/gala-hir/src/lower.rs` — AST → HIR lowering (desugaring)
- `crates/gala-hir/src/resolve.rs` — Name resolution, module graph
- `crates/gala-hir/src/def_id.rs` — `DefId`, `DefPath`, `LocalDefId` definitions
- `crates/gala-hir/src/db.rs` — Salsa database definition and queries
- `crates/gala-hir/src/module.rs` — Module graph, imports, visibility

## Common Tasks

### Add new desugaring
1. Identify surface syntax in AST (e.g., in-place gate sugar `h(q)` → `let q = h(q)`)
2. Add lowering case in `lower.rs` `LowerAst::lower_expr` / `lower_stmt`
3. Ensure `DefId` assignment for new bindings
4. Add conformance test in `tests/conformance/hir/` or relevant category
5. Verify LSP sees desugared form (hover shows explicit form)

### Modify name resolution / module system
1. Edit `resolve.rs` for resolution logic (scope, imports, shadowing)
2. Edit `module.rs` for module graph construction
3. Update `DefId` allocation if new definition kinds added
4. Test cross-file resolution, re-exports, glob imports

### Add salsa query
1. Define query in `db.rs` with `#[salsa::query]`
2. Implement in appropriate module
3. Use `db.query_name(input)` in consumers (type checker, LSP)
4. Test incremental recomputation: change source, verify only affected queries rerun

### Add new definition kind (e.g., new item type)
1. Add `DefKind` variant in `def_id.rs`
2. Add `DefId` construction in `lower.rs`
3. Add resolution entry in `resolve.rs`
4. Update module graph in `module.rs`

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
// In db.rs
#[salsa::query]
fn hir(db: &dyn Db, file: FileId) -> Hir { ... }

#[salsa::query]
fn module_graph(db: &dyn Db, crate_id: CrateId) -> ModuleGraph { ... }

#[salsa::query]
fn resolve(db: &dyn Db, def_id: DefId) -> Resolution { ... }
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