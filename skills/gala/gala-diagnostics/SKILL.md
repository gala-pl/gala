---
name: gala-diagnostics
description: Work with Gala's diagnostic system — ariadne rendering, error codes (E01xx-E06xx), explain entries, JSON output
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's diagnostics (`crates/gala-diagnostics`) provides **rich diagnostic rendering** via **ariadne** and a **centralized error code registry**. Every diagnostic has a code (`E01xx` syntax, `E02xx` types, `E03xx` effects, `E04xx` linearity, `E05xx` uncomputation, `E06xx` backend/diff) and a long-form `gala explain <code>` entry.

> **Note on file layout:** The crate currently lives in `crates/gala-diagnostics/src/lib.rs` + `crates/gala-diagnostics/src/explain/*.md`. As it matures it may be split into `code.rs`, `render.rs`, `explain.rs`, `json.rs`, `fix.rs` (see `docs/ARCHITECTURE.md`). When in doubt, search for the relevant code in `lib.rs`.

Key files:
- `crates/gala-diagnostics/src/lib.rs` — `DiagnosticCode`, `Diagnostic`, `Diagnostics`, `codes` module, `Emitter`
- `crates/gala-diagnostics/src/explain/` — Long-form explanations (markdown files per code)

## Common Tasks

### Add new diagnostic code
1. Add constant to `codes` module in `lib.rs` (e.g., `E0412` for use-after-consume)
2. Define severity (Error/Warning/Note), default labels, help text
3. Add `explain` markdown entry in `src/explain/` with:
   - What the error means (in quantum terms)
   - Why it's a physical law violation
   - Code example showing wrong + correct
   - Workaround if applicable
4. Emit from relevant crate (`gala-types`, `gala-uncompute`, etc.)
5. Add conformance test in `tests/conformance/errors/`

### Improve diagnostic rendering
1. Edit `Emitter` in `lib.rs` — use ariadne `Report`, `Label`, `Source`
2. Add related labels (definition site, consumption site)
3. Ensure spans are accurate (byte offsets from `gala-span`)
4. Test with `cargo run --bin gala -- check examples/broken.gala`

### Add suggested fix (for `gala fix`)
1. Edit `lib.rs` — add `SuggestedFix` with `TextEdit` (span + replacement)
2. Implement for common errors (E0412, E0530, E03xx)
3. Test `gala fix --apply` on conformance suite
4. Validate fix by re-running checker

### Extend JSON output
1. Edit `lib.rs` — extend `DiagnosticJson` schema
2. Include: code, severity, message, labels (span, text), help, fix (if any)
3. Version schema: `diagnostic.v1.json`
4. Test: `gala check --json examples/broken.gala | jq`

## Testing

```bash
cargo test -p gala-diagnostics              # Unit tests
cargo test --workspace diagnostics          # All diagnostic tests
cargo test --workspace conformity/errors    # Error code conformance
cargo run --bin gala -- explain E0412       # Test explain
cargo run --bin gala -- check --json examples/broken.gala  # Test JSON
```

## Conventions

- **Every diagnostic has a code** — no anonymous errors
- **Codes are stable** — never reuse/rename codes
- **Teaching-grade messages** — explain physics, not just syntax
- **`help:` text is actionable** — shows correct pattern
- **`--explain` is comprehensive** — mini-tutorial per code
- **JSON schema versioned** — `diagnostic.v1.json`
- No `unwrap()` on user paths

## Error Code Ranges

| Range | Category | Examples |
|-------|----------|----------|
| E01xx | Syntax/Lexer | E0101 unexpected token, E0102 unclosed block comment |
| E02xx | Types | E0201 type mismatch, E0202 unresolved type |
| E03xx | Effects | E0301 prob in quantum, E0302 effect mismatch |
| E04xx | Linearity | E0412 use after consume, E0413 implicit drop, E0414 duplication |
| E05xx | Uncomputation | E0530 cannot auto-uncompute, E0531 non-liftable input |
| E06xx | Backend/Diff | E0601 capability mismatch, E0602 grad not supported |

## Debugging

```bash
# Render diagnostic for file
cargo run --bin gala -- check examples/broken.gala

# Get explain for code
cargo run --bin gala -- explain E0412
cargo run --bin gala -- explain E0412 --markdown

# JSON output for tools/agents
cargo run --bin gala -- check --json examples/broken.gala

# Test suggested fix
cargo run --bin gala -- fix --apply examples/broken.gala
```