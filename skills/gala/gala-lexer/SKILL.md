---
name: gala-lexer
description: Work with Gala's logos-based lexer — add tokens, fix tokenization issues, extend lexical syntax
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Bash(cargo run --bin gala *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's lexer (`crates/gala-lexer`) uses **logos** for fast, deterministic tokenization. It produces a token stream with byte spans for use by the parser and diagnostics.

Key files:
- `crates/gala-lexer/src/token.rs` — TokenKind enum, token struct
- `crates/gala-lexer/src/lexer.rs` — Logos derive macro, tokenization logic
- `crates/gala-lexer/tests/` — Tokenization tests (roundtrip, edge cases)

## Common Tasks

### Add a new keyword or token
1. Add variant to `TokenKind` in `token.rs`
2. Add regex/pattern in `lexer.rs` via `#[logos(...)]`
3. Update keyword list in `extensions/README.md` if user-facing
4. Add conformance test in `tests/conformance/lexer/` (or relevant category)
5. Run `cargo test -p gala-lexer`

### Fix tokenization bug
1. Reproduce with `cargo run --bin gala -- <file.gala>` or unit test
2. Check `logos` pattern ordering (longer patterns first)
3. Verify span accuracy (byte offsets)
4. Test Unicode identifiers (Greek letters θ, φ, ψ) and complex literals (2+3i)

### Extend numeric/complex literal syntax
- Modify `Number`/`Complex` patterns in `lexer.rs`
- Ensure `from_str_radix` handling in token.rs is correct
- Add property tests for roundtrip parsing

## Testing

```bash
cargo test -p gala-lexer                    # Unit tests
cargo test --workspace lexer               # All lexer-related tests
python -m pytest tests/test_syntax_fixtures.py -k lexer  # Syntax fixture tests
```

## Conventions

- Keywords are lowercase: `fn`, `let`, `qubit`, `measure`, `reverse`, `adjoint`, `control`, `grad`, `drop`
- Reserved quantum keywords: `qubit`, `qubits`, `measure`, `reverse`, `adjoint`, `control`, `grad`, `drop`
- Types: `Qubit`, `Qubits<N>`, `Measured<T>`, `Bool`, `Int`, `Float`, `Complex`, `Params`
- Effects: `pure`, `quantum`, `prob`
- No `unwrap()` on user paths — return `Result` or use `Diagnostic`
- Follow Rust edition 2021, max_width 100

## Debugging

```bash
# Debug token stream
cargo run --bin gala -- --emit tokens examples/hello_world.gala

# Enable logos debug (if compiled with debug-assertions)
RUST_LOG=debug cargo run --bin gala -- examples/hello_world.gala
```