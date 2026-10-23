---
name: gala-parser
description: Work with Gala's chumsky parser — extend grammar, fix parsing issues, improve error recovery
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Bash(cargo run --bin gala *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's parser (`crates/gala-parser`) uses **chumsky** for parser combinators with **error recovery**. It produces both an AST (semantic) and a CST (concrete syntax tree) for the formatter and LSP.

> **Note on file layout:** The crate currently lives in a single `crates/gala-parser/src/lib.rs`. As it matures it may be split into `parser.rs`, `expr.rs`, `stmt.rs`, `item.rs`, `recovery.rs` (see `docs/ARCHITECTURE.md`). When in doubt, search for the relevant code in `lib.rs`.

Key files:
- `crates/gala-parser/src/lib.rs` — Main parser construction, expression/statement/item parsing, error recovery
- `crates/gala-ast/` — AST node definitions (shared crate)

## Common Tasks

### Add new syntax / grammar rule
1. Define AST node in `gala-ast` if new construct
2. Add parser function in `lib.rs`
3. Wire into main parser
4. Add error recovery for the new construct
5. Add conformance tests in `tests/conformance/` or relevant category
6. Test error recovery: feed broken code, verify partial AST + diagnostics

### Fix parsing bug / improve error recovery
1. Reproduce with `cargo run --bin gala -- <file.gala>`
2. Check error recovery strategies in `lib.rs`
3. Ensure diagnostics have correct `E01xx` codes and spans
4. Test that LSP still works on broken files (partial AST)

### Extend expression grammar
- Operator precedence defined in `lib.rs`
- Add new binary/unary operators with correct precedence
- Update `Token` in `gala-lexer` if new tokens needed

## Testing

```bash
cargo test -p gala-parser                    # Unit tests
cargo test --workspace parser               # All parser-related tests
cargo run --bin gala -- --emit ast examples/hello_world.gala  # Debug AST output
python -m pytest tests/test_syntax_fixtures.py -k parser  # Syntax fixture tests
```

## Conventions

- Grammar follows [LANGUAGE_SPEC.md](../docs/LANGUAGE_SPEC.md) §3 (EBNF)
- Error recovery: every parser function should have a `.recover_with(...)` strategy
- Produce both AST and CST: use `chumsky::prelude::Parser::map_with_span` for CST nodes
- No `unwrap()` on user paths — propagate `Result` or emit diagnostics
- Keywords from lexer: `fn`, `let`, `mut`, `if`, `else`, `match`, `for`, `in`, `while`, `return`, `import`, `type`, `struct`, `enum`, `trait`, `impl`, `const`, `where`, `pure`, `quantum`, `prob`

## Debugging

```bash
# Debug parse tree
cargo run --bin gala -- --emit ast examples/hello_world.gala

# Debug CST (for formatter)
cargo run --bin gala -- --emit cst examples/hello_world.gala

# Parse with verbose recovery info
RUST_LOG=gala_parser=debug cargo run --bin gala -- examples/broken.gala
```