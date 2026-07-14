@AGENTS.md

# Gala — Quantum-Classical Programming Language

Gala is a hybrid quantum-classical programming language. Its compiler enforces the laws of quantum mechanics at compile time. This is a Rust workspace monorepo.

## Build & Test Commands

- `cargo build --workspace` — build all crates
- `cargo build --release --workspace` — production build
- `cargo test --workspace` — run all tests
- `cargo clippy --workspace -- -D warnings` — lint (deny all)
- `cargo fmt --check` — check formatting
- `cargo fmt --all` — format all code
- `cargo doc --workspace --no-deps` — verify docs build
- `cargo deny check` — dependency audit
- `cargo run --bin gala -- <file>` — run compiler on a .gala file
- `pre-commit run --all-files` — run all pre-commit hooks
- `cd apps/docs && bun next dev` — documentation dev server
- `cd apps/docs && bun tsc --noEmit` — docs typecheck

## Code Style

- Rust edition 2021, max_width 100, reorder imports and modules
- Errors are values (Result, diagnostics); no unwrap() on user-reachable paths
- Public APIs documented with `///`; internal invariants asserted in debug builds
- Use designated crates: chumsky (parsing), logos (lexing), ariadne (diagnostics), salsa (incremental), inkwell (LLVM), roqoqo/roqoqo-quest (simulation), tower-lsp (LSP), clap (CLI)
- Conventional commits: feat:, fix:, refactor:, test:, docs:, chore:
- Feature branches named `wp-XXX-short-slug`; squash-merge to main

## Repository Structure

- `compiler/` — compiler binary crate with lexer, parser, AST, IR, codegen
- `library/std/` — Gala-language standard library (compiler intrinsics, gates, algorithms, ML, noise, hardware)
- `tools/gala-fmt/` — code formatter
- `tools/gala-lsp/` — language server protocol
- `apps/docs/` — documentation website (Next.js + Fumadocs via Bun)
- `extensions/` — editor extensions (VS Code, IntelliJ, Vim, Helix, Zed, etc.)
- `tests/` — integration and conformance tests
- `examples/` — example .gala programs
- `docs/` — language specification and design docs

## Compiler Pipeline

.gala source → Lexer (logos) → Tokens → Parser (chumsky) → AST/CST → HIR (desugared, name-resolved) via salsa → Types/Effects inference + linearity check → Uncomputation analysis → GIR (typed, effect-annotated op-graph) → gala-diff / gala-opt / gala-qir / gala-sim / gala-codegen-classical

## Workflow

1. Explore first — understand the codebase before making changes
2. Plan mode for multi-file changes or architectural decisions
3. Write tests before or alongside implementation
4. Run `cargo clippy --workspace -- -D warnings` and `cargo test --workspace` after changes
5. Verify docs build with `cargo doc --workspace --no-deps`
6. Commit with conventional commit messages
