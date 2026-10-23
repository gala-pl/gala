---
name: gala-span
description: Source spans, file database, string interning for Gala — byte offsets, Span, FileId, lasso interning
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

`gala-span` provides source location tracking used by lexer/parser/diagnostics. Key file: `crates/gala-span/src/lib.rs`. Uses `lasso` for string interning.

## Common Tasks

1. Modify `Span`, `FileId`, or interning APIs in `lib.rs`
2. Ensure all consumers (lexer, parser, diagnostics) still compile
3. Run `cargo build --workspace` and `cargo test -p gala-span`

## Testing

```bash
cargo test -p gala-span
cargo build --workspace
```
