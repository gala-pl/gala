---
paths:
  - "tests/**/*"
  - "**/*_test.rs"
  - "**/test_*.py"
---

# Testing Conventions

## Rust Unit Tests
- Use `#[test]` attributes; place tests in a `mod tests` block at bottom of file
- Use `proptest` for property-based testing of compiler passes
- Use `insta` for snapshot/golden tests (diagnostics, GIR dumps, formatter output)
- Name tests descriptively: `test_<function>_<scenario>_<expected>()`

## Integration Tests
- Conformance tests live in `tests/` with `.gala` fixture files
- Each fixture tests one language feature or error case
- Test both passing and failing compilations

## Property Tests (Quantum-Aware)
- `#[property(unitary)]` — verify a circuit implements a unitary transformation
- `#[property(reversible)]` — verify that reverse(circuit) * circuit = identity
- `#[property(uncomputes)]` — verify uncomputation correctly cleans up ancillae
- `#[property(grad_matches)]` — verify parameter-shift gradient matches finite difference

## Python Extension Tests
- Run with `python -m unittest tests/test_*.py`
- Test VS Code, Vim, Helix, and other editor extensions
- Validate syntax highlighting, tree-sitter grammar, and LSP integration

## Before Committing
- Run `cargo test --workspace` — all tests must pass
- Run `cargo clippy --workspace -- -D warnings` — no warnings
- Run `cargo fmt --check` — formatting must be clean
