# Changelog

All notable changes to the Gala programming language are documented here.

## [Unreleased]

### Added

- **Decision log** — `docs/decisions/` with ADRs 0001-0006 (hand-written front end, pure-Rust
  simulator, textual QIR, uncomputation by recorded history, effect lattice and in-place gates,
  error code scheme) and an index README.
- **RFC template** — `docs/RFC_TEMPLATE.md`, with the process steps from CONTRIBUTING §4.
- **Formal core sketch** — `docs/FORMAL_CORE.md`: operational semantics and progress,
  preservation, and linearity sketches for the implemented core. Not mechanized.
- **Tutorial** — `docs/TUTORIAL/`, five modules covering classical code, a Bell pair, linearity
  errors, effects, and a hybrid classifier. Human course validation is not done.
- **`gala-doc`** — library crate that renders Markdown API docs from `///` comments on top-level
  functions (`document_source`, `document_file`).

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
