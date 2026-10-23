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

Key files:
- `crates/gala-driver/src/db.rs` — Salsa database definition, all queries
- `crates/gala-driver/src/compile.rs` — High-level compilation pipeline
- `crates/gala-driver/src/emit.rs` — Artifact emission (GIR, QIR, LLVM, etc.)
- `crates/gala-driver/src/incremental.rs` — Incremental compilation, cache invalidation

## Common Tasks

### Add new salsa query
1. Edit `db.rs` — add `#[salsa::query]` function
2. Implement in appropriate crate (hir, types, uncompute, gir, etc.)
3. Wire into `compile.rs` pipeline if needed
4. Test incremental: edit source, verify only affected queries rerun

### Modify compilation pipeline
1. Edit `compile.rs` — `CompilePipeline::run()` defines pass order
2. Add/remove/reorder passes (lex → parse → hir → types → linearity → uncompute → gir → ...)
3. Each pass is a salsa query or query group
4. Ensure diagnostics aggregated correctly

### Add emission target
1. Edit `emit.rs` — add `EmitTarget` variant
2. Implement emission function (e.g., `emit_gir`, `emit_qir`)
2. Register in CLI `--emit` flag
3. Test: `gala build --emit new-target examples/hello.gala`

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
// In db.rs
#[salsa::query]
fn source_text(db: &dyn Db, file: FileId) -> String;

#[salsa::query]
fn tokens(db: &dyn Db, file: FileId) -> Vec<Token>;

#[salsa::query]
fn ast(db: &dyn Db, file: FileId) -> Ast;

#[salsa::query]
fn hir(db: &dyn Db, file: FileId) -> Hir;

#[salsa::query]
fn type_of(db: &dyn Db, def: DefId) -> Ty;

#[salsa::query]
fn effect_of(db: &dyn Db, def: DefId) -> Effect;

#[salsa::query]
fn linearity_check(db: &dyn Db, def: DefId) -> Vec<Diagnostic>;

#[salsa::query]
fn uncompute(db: &dyn Db, def: DefId) -> UncomputePlan;

#[salsa::query]
fn gir(db: &dyn Db, def: DefId) -> Gir;

#[salsa::query]
fn diagnostics(db: &dyn Db, file: FileId) -> Vec<Diagnostic>;
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