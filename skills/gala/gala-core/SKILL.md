---
name: gala-core
description: Core primitive types for Gala — shared foundational types used across compiler crates
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

`gala-core` provides foundational primitive types shared by the compiler crates. Key file: `crates/gala-core/src/lib.rs`.

## Common Tasks

1. Add/modify primitive types in `lib.rs`
2. Update dependents (all crates using `gala-core`)
3. Run `cargo build --workspace` and `cargo test -p gala-core`

## Testing

```bash
cargo test -p gala-core
cargo build --workspace
```
