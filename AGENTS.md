# Gala — Agent Instructions

This file is imported by CLAUDE.md via `@AGENTS.md`. It contains instructions for AI coding agents working on the Gala programming language project.

## Project Overview

Gala is a hybrid quantum-classical programming language. The compiler enforces the laws of quantum mechanics (no-cloning, reversibility, safe uncomputation) while making hybrid quantum-classical ML ergonomic end to end.

## Key Constraints

- Rust edition 2021, pinned toolchain `stable-2026-04-14`
- All errors are values — no `unwrap()` on user-reachable paths
- Conventional commits: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`
- Feature branches: `wp-XXX-short-slug`, squash-merge to main
- Pre-commit hooks enforce formatting, linting, and conventional commits

## Build & Test

```
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

## Architecture

The compiler pipeline: source → Lexer (logos) → Parser (chumsky) → AST → HIR (salsa) → Types/Effects → Uncomputation → GIR → Backends (LLVM/QIR/Sim)

See `docs/ARCHITECTURE.md` for the full crate map and `docs/COMPILER_PIPELINE.md` for the pass-by-pass flow.
