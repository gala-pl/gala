---
name: gala-fmt
description: Work with Gala's canonical formatter — lossless CST, idempotent, one style, --check/--diff/--stdin
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's formatter (`crates/gala-fmt`) operates on the **lossless CST** from the parser (preserves comments, whitespace). Produces **one canonical style**, is **idempotent** (`fmt(fmt(x)) == fmt(x)`), and supports `--check`, `--diff`, `--stdin` for CI.

Key files:
- `crates/gala-fmt/src/format.rs` — Main formatting logic, CST → formatted string
- `crates/gala-fmt/src/shape.rs` — Layout shapes (line width, indentation, breaks)
- `crates/gala-fmt/src/rewrite.rs` — CST rewriting with trivia preservation
- `crates/gala-fmt/src/config.rs` — Configuration (minimal: line width only)

## Common Tasks

### Adjust formatting style
1. Edit `shape.rs` — modify `Shape` for constructs (fn, let, match, call, etc.)
2. Line width: default 100, configurable via `--width`
3. Indentation: 4 spaces (fixed, not configurable)
4. Test idempotence: `fmt(fmt(x)) == fmt(x)`

### Fix formatting bug
1. Reproduce: `cargo run --bin gala -- fmt --check examples/broken.gala`
2. Check `rewrite.rs` for trivia handling (comments, whitespace)
3. Ensure comments stay attached to correct nodes
4. Test on stdlib: `cargo run --bin gala -- fmt --check library/std/`

### Add formatting for new syntax
1. Parser must produce CST nodes for new syntax (in `gala-parser`)
2. Add case in `format.rs` `format_node` match
3. Define `Shape` in `shape.rs`
4. Test with new syntax examples

### CI integration
- `gala fmt --check` exits 1 if changes needed
- `gala fmt --diff` shows unified diff
- `gala fmt --stdin` formats stdin → stdout
- Pre-commit hook runs `gala fmt --check`

## Testing

```bash
cargo test -p gala-fmt                      # Unit tests
cargo test --workspace fmt                  # All fmt tests
cargo run --bin gala -- fmt --check library/std/  # Check stdlib
cargo run --bin gala -- fmt examples/hello_world.gala --diff  # Show diff
# Idempotence test
cargo run --bin gala -- fmt examples/hello_world.gala --stdin < examples/hello_world.gala | cargo run --bin gala -- fmt --stdin
```

## Conventions

- **CST-based** — never loses comments or original whitespace intent
- **One style** — no configuration beyond line width
- **Idempotent** — `fmt(fmt(x)) == fmt(x)` always
- **Fast** — incremental via salsa not needed; formats whole file
- **Deterministic** — same input = same output
- No `unwrap()` on user paths

## Key Formatting Rules

```rust
// Function definition
fn name<T>(params) -> Ret effect {
    body
}

// Let binding
let mut name: Type = expr;

// Match expression
match expr {
    Pat => expr,
    Pat => expr,
}

// Call with named args
call(arg1, arg2: value, arg3: value)

// Quantum gates (in-place sugar)
h(q)
cx(a, b)
```

## Debugging

```bash
# Format file in-place
cargo run --bin gala -- fmt examples/hello_world.gala

# Check only (CI)
cargo run --bin gala -- fmt --check examples/hello_world.gala

# Show diff
cargo run --bin gala -- fmt examples/hello_world.gala --diff

# Format stdin
cat examples/hello_world.gala | cargo run --bin gala -- fmt --stdin

# Debug CST
cargo run --bin gala -- --emit cst examples/hello_world.gala
```