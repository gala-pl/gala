---
name: gala-lint
description: Linter for Gala source files — rules, CLI binary gala-lint
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Bash(cargo run --bin gala-lint *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

`gala-lint` is a standalone linter binary + library.

Key files:
- `crates/gala-lint/src/lib.rs` — Lint rules
- `crates/gala-lint/src/main.rs` — CLI binary

## Common Tasks

1. Add lint rules in `lib.rs`
2. Update CLI in `main.rs`
3. Run `cargo test -p gala-lint` and `cargo run --bin gala-lint`

## Testing

```bash
cargo test -p gala-lint
cargo build -p gala-lint
```