# Developer Guide

Quick-start guide for developers working on the Gala programming language.

## Prerequisites

- Rust toolchain (pinned in `rust-toolchain.toml`): `stable-2026-04-14`
- For the docs site: Bun (JavaScript runtime)
- pre-commit (for git hooks)

## First-Time Setup

```bash
# Install pre-commit hooks
pre-commit install

# Build all crates
cargo build --workspace

# Run the full test suite
cargo test --workspace
```

## Project Layout

| Directory | Contents |
|-----------|----------|
| `compiler/` | Core compiler: lexer, parser, AST, IR, codegen, diagnostics |
| `library/` | Gala language libraries |
| `library/std/` | Standard library (intrinsics, gates, algorithms, ML, noise, hardware) |
| `tools/gala-cli/` | Unified CLI |
| `tools/gala-fmt/` | Code formatter |
| `tools/gala-lint/` | Linter |
| `tools/gala-lsp/` | Language server |
| `tools/gala-pkg/` | Package manager |
| `apps/docs/` | Documentation website (Next.js + Fumadocs) |
| `extensions/` | Editor extensions (VS Code, Vim, Helix, Zed, IntelliJ, etc.) |
| `examples/` | Example Gala programs |
| `tests/` | Integration and conformance tests |
| `docs/` | Language specification, architecture docs |

## Common Tasks

```bash
# Build
cargo build --workspace

# Run tests
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all
cargo fmt --check

# Run the compiler
cargo run --bin gala -- --input examples/hello_world.gala

# Format a Gala file
cargo run --bin gala-fmt -- examples/hello_world.gala

# Run the docs site
cd apps/docs && bun dev

# Pre-commit checks
pre-commit run --all-files
```

## Coding Standards

- Follow `rustfmt` conventions (max_width 100, reorder imports)
- No `unwrap()` on user-reachable paths
- Public APIs must have `///` doc comments
- Conventional commit messages: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`
- Feature branches: `wp-XXX-short-slug`

## Architecture

See `docs/ARCHITECTURE.md` for the crate map and data flow.
See `docs/COMPILER_PIPELINE.md` for the pass-by-pass compilation flow.
See `docs/CONTRIBUTING.md` for the full contribution guide.
