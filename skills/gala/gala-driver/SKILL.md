---
name: gala-driver
description: Work with Gala's compilation driver — salsa DB orchestration, incremental queries, pipeline wiring
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's driver (`crates/gala-driver`) **orchestrates compilation** by wiring the **salsa query database**. It defines the `Db` trait, constructs the query graph, and provides high-level compilation entry points used by `gala-cli` and `gala-lsp`.

> **Note on file layout:** The crate currently lives in `crates/gala-driver/src/lib.rs` + `crates/gala-driver/src/db.rs`. As it matures it may be split into `compile.rs`, `emit.rs`, `incremental.rs` (see `docs/ARCHITECTURE.md`). When in doubt, search for the relevant code in `lib.rs` or `db.rs`.

Key files:
- `crates/gala-driver/src/db.rs` — Salsa database definition, all queries
- `crates/gala-driver/src/lib.rs` — High-level compilation API

## Common Tasks

### Add new salsa query
1. Edit `db.rs` — add `#[salsa::tracked]` function
2. Implement in appropriate crate (hir, types, uncompute, gir, etc.)
3. Wire into pipeline if needed
4. Test incremental: edit source, verify only affected queries rerun

### Modify compilation pipeline
1. Edit `db.rs` — queries define pass order (lex → parse → hir → types → linearity → uncompute → gir → ...)
2. Each pass is a salsa query or query group
3. Ensure diagnostics aggregated correctly

### Add emission target
1. Edit `lib.rs` — add `EmitTarget` variant
2. Implement emission function (e.g., `emit_gir`, `emit_qir`)
3. Register in CLI `--emit` flag
4. Test: `gala build --emit new-target examples/hello.gala`

### Optimize incremental performance
1. Profile salsa: `RUST_LOG=salsa=debug`
2. Minimize query inputs (use fine-grained queries)
3. Use `salsa::durability` for rarely-changing inputs
4. Benchmark: cold vs incremental build time

## Testing

```bash
cargo test -p gala-driver                   # Unit tests
cargo test --workspace driver               # All driver tests
cargo run --bin gala -- build examples/hello.gala  # Full compile
cargo run --bin gala -- check examples/hello.gala  # Check only
# Incremental test: run twice, second should be faster
```

## Conventions

- **Single Db trait** — all crates implement/query through it
- **Queries are pure** — no side effects, deterministic
- **Diagnostics aggregated** — `diagnostics(file)` query collects all
- **Backend-agnostic above GIR** — driver never imports backend crates
- **Shared with LSP** — same Db, same queries
- No `unwrap()` on user paths

## Key Queries (from COMPILER_PIPELINE.md)

```rust
// In crates/gala-driver/src/db.rs
#[salsa::input]
pub struct SourceFile { ... }

#[salsa::tracked]
pub fn tokens(db, file: SourceFile) -> Vec<(Token, Span)>;

#[salsa::tracked]
pub fn ast(db, file: SourceFile) -> Arc<Vec<Item>>;

#[salsa::tracked]
pub fn hir_file(db, file: SourceFile) -> Arc<HirFile>;

#[salsa::tracked]
pub fn hir_file_resolved(db, file: SourceFile) -> Arc<HirFile>;

#[salsa::tracked]
pub fn hir_functions(db, file: SourceFile) -> Arc<HashMap<DefId, HirFnDef>>;

#[salsa::tracked]
pub fn module_graph(db, file: SourceFile) -> Arc<ModuleGraph>;

#[salsa::tracked]
pub fn type_of_query(db, def_id: DefIdInput) -> Result<Arc<Ty>, Arc<Diagnostics>>;

#[salsa::tracked]
pub fn effect_of_query(db, def_id: DefIdInput) -> Result<Arc<TyEffect>, Arc<Diagnostics>>;

#[salsa::tracked]
pub fn linearity_check_query(db, def_id: DefIdInput) -> Result<Arc<Diagnostics>, Arc<Diagnostics>>;

#[salsa::tracked]
pub fn uncompute_query(db, def_id: DefIdInput) -> Result<Arc<UncomputePlan>, Arc<Diagnostics>>;

#[salsa::tracked]
pub fn gir_query(db, def_id: DefIdInput) -> Result<Arc<Gir>, Arc<Diagnostics>>;

#[salsa::tracked]
pub fn gir_diff_query(db, def_id: DefIdInput) -> Result<Arc<Gir>, Arc<Diagnostics>>;

#[salsa::tracked]
pub fn gir_opt_query(db, def_id: DefIdInput) -> Result<Arc<Gir>, Arc<Diagnostics>>;

#[salsa::tracked]
pub fn diagnostics(db, file: SourceFile) -> Arc<Vec<Diagnostic>>;
```

## Debugging

```bash
# Full compilation with timing
RUST_LOG=gala_driver=info cargo run --bin gala -- build examples/hello.gala

# Salsa query trace
RUST_LOG=salsa=debug cargo run --bin gala -- build examples/hello.gala

# Incremental: run twice
cargo run --bin gala -- build examples/hello.gala
# Edit file
cargo run --bin gala -- build examples/hello.gala  # Should be faster

# Emit specific artifact
cargo run --bin gala -- build examples/hello.gala --emit gir
```