# ADR 0001: Hand-written lexer and parser

- Status: Accepted
- Date: 2026-10-22

## Context

[AGENTIC_BUILD_PLAN.md](../AGENTIC_BUILD_PLAN.md) and [CONTRIBUTING.md](../CONTRIBUTING.md) §3
name `logos` for lexing and `chumsky` for parsing. The front end has to produce a usable AST
even for broken input, because the teaching diagnostics and the language server depend on
recovering from errors and reporting several of them in one run.

## Decision

- The lexer is hand-written in `crates/gala-lexer`. It works over `source.chars()` and converts
  character positions to byte offsets when it emits a token (`byte_offsets` in the lexer state).
  Spans are byte offsets, so they index the original source directly.
- The parser is a hand-written recursive-descent parser in `crates/gala-parser`. It returns a
  `ParseOutput` with whatever items it could recover plus every diagnostic it recorded. Recovery
  unwinds to an item boundary (`recover_item`) and continues.
- The workspace still declares `logos` and `chumsky` in the root `Cargo.toml` and
  `crates/gala-lexer/Cargo.toml`, but no crate source imports them. They are not used.

## Consequences

- Error recovery is under our control, so one file can yield several diagnostics with exact spans.
- Byte-offset spans can be passed to `ariadne` and the LSP without a translation layer.
- The parser and lexer are more code to maintain than a generator-based version. Each new token
  or production is written by hand.
- Follow-up: remove the unused `logos` and `chumsky` declarations, or record a new ADR that
  re-adopts them. Until then the CONTRIBUTING §3 list is out of date for these two crates.
