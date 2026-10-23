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

Key files:
- `crates/gala-parser/src/parser.rs` — Main parser construction
- `crates/gala-parser/src/expr.rs` — Expression parsing
- `crates/gala-parser/src/stmt.rs` — Statement parsing
- `crates/gala-parser/src/item.rs` — Top-level item parsing (fn, struct, etc.)
- `crates/gala-parser/src/recovery.rs` — Error recovery strategies
- `crates/gala-ast/` — AST node definitions (shared crate)
- `crates/gala-parser/tests/` — Parsing tests (golden, recovery)

## Common Tasks

### Add new syntax / grammar rule
1. Define AST node in `gala-ast` if new construct
2. Add parser function in appropriate module (`expr.rs`, `stmt.rs`, `item.rs`)
3. Wire into main parser in `parser.rs`
4. Add error recovery for the new construct
5. Add conformance tests in `tests/conformance/parser/` or relevant category
6. Test error recovery: feed broken code, verify partial AST + diagnostics

### Fix parsing bug / improve error recovery
1. Reproduce with `cargo run --bin gala -- <file.gala>`
2. Check `recovery.rs` for recovery strategies (delimiters, panic mode)
3. Ensure diagnostics have correct `E01xx` codes and spans
4. Test that LSP still works on broken files (partial AST)

### Extend expression grammar
- Operator precedence defined in `expr.rs` via `precedence!` macro
- Add new binary/unary operators with correct precedence
- Update `TokenKind` in `gala-lexer` if new tokens needed

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