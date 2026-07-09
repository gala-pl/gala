# Contributing to Gala

Thank you for your interest in contributing to Gala — the hybrid quantum-classical programming language.

## Getting Started

See `docs/CONTRIBUTING.md` for the full contribution guide, including:

- Definition of Done checklist
- Branch naming conventions (`wp-XXX-short-slug`)
- Commit message format (conventional commits)
- Code review expectations
- RFC process for language changes
- Agent/automation rules

## Quick Links

- [Architecture Overview](docs/ARCHITECTURE.md)
- [Compiler Pipeline](docs/COMPILER_PIPELINE.md)
- [Language Specification](docs/LANGUAGE_SPEC.md)
- [Type System](docs/TYPE_SYSTEM.md)
- [Testing & QA](docs/TESTING_QA.md)
- [Roadmap](docs/ROADMAP.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)

## Development Setup

```bash
pre-commit install
cargo build --workspace
cargo test --workspace
```

## Code Standards

- Rust edition 2021, `rustfmt` conventions (max_width 100)
- `cargo clippy --workspace -- -D warnings` must pass
- All errors are values; no `unwrap()` on user-reachable paths
- Public APIs documented with `///`
