# Changelog

All notable changes to the Gala programming language are documented here.

## [0.1.0] - Unreleased

### Added

- **Lexer** — tokenizes Gala source into operators, literals, identifiers, and keywords
- **Parser** — Pratt parser for expressions; recursive descent for definitions and blocks
- **AST** — strongly-typed node definitions for the full Gala grammar
- **Codegen** — pretty-printer that emits valid Gala source from the AST
- **IR module** — intermediate representation builder for future optimization passes
- **Diagnostics** — structured error reporting with spans and notes
- **CLI** — `gala` binary with `--input`, `--output`, and `--emit` flags
- **Core library** — `gala-core` with `Int`, `Float`, `Bool`, and tuple primitives (no_std)
- **Standard library** — `crates/gala-std` with `io`, `vec`, and `str` modules (Rust runtime backing)
- **Gala stdlib sources** — `library/std/` with `core`, `gates`, `algorithms`, `ml`, `noise`, `hardware`, `classical`
- **gala-fmt** — basic code formatter
- **gala-lsp** — language server scaffolding
- **Example programs** — `hello_world.gala` and `fib.gala`
