---
name: gala-fuzz
description: Shared fuzzing infrastructure for the Gala compiler — proptest/libfuzzer harnesses
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

`gala-fuzz` provides shared fuzzing infrastructure used by compiler crates. Key file: `crates/gala-fuzz/src/lib.rs`. Depends on `gala-driver`, `gala-parser`, `gala-diagnostics`, `gala-span`.

## Common Tasks

1. Add fuzz targets/harnesses in `lib.rs`
2. Run property tests via `cargo test -p gala-fuzz`

## Testing

```bash
cargo test -p gala-fuzz
```