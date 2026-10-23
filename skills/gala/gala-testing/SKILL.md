---
name: gala-testing
description: Work with Gala's testing infrastructure — conformance suite, property testing, golden tests, CI pipeline
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's testing is a **language feature**, not just internal practice. It includes:

- **Conformance suite** (`tests/conformance/`) — executable language spec
- **Quantum property testing** — `#[property(unitary)]`, `reversible`, `uncomputes`, `grad_matches`
- **Golden/snapshot tests** — diagnostics, GIR, QIR, formatter output via `insta`
- **Integration tests** — end-to-end on simulator, hardware smoke tests

Key files:
- `tests/conformance/` — Source + expectation pairs by category
- `tests/conformance_test_parse.rs` — Conformance test harness
- `tests/integration_test.rs` — End-to-end integration tests
- `crates/gala-test/` (if exists) — Property test harness
- `crates/*/tests/` — Per-crate unit/property tests

## Common Tasks

### Add conformance test case
1. Choose category: `types/`, `effects/`, `linearity/`, `uncompute/`, `diff/`, `qir/`, `run/`, `valid/`, `errors/`
2. Create `tests/conformance/<category>/<name>.gala` — source code
3. Create `tests/conformance/<category>/<name>.expect` — expectation:
   - For valid: expected type, GIR snapshot, or runtime result
   - For errors: diagnostic code(s) expected (e.g., `E0412`)
4. Run: `cargo test --workspace conformity`

### Add property test
1. In Gala source, add attribute: `#[property(unitary)] fn test_name() { ... }`
2. Property types: `unitary`, `reversible`, `uncomputes`, `grad_matches`, `effect_honesty`
3. Harness runs on simulator with randomized inputs
4. Failures shrink to minimal case

### Add golden/snapshot test
1. In crate tests, use `insta::assert_snapshot!` or `assert_json_snapshot!`
2. Snapshots stored in `crates/<crate>/tests/snapshots/`
3. Update: `cargo test --workspace -- --update-snapshots`
4. Review diff before committing

### Run full test suite
```bash
# Unit + property + snapshot
cargo test --workspace

# Conformance suite
cargo test --workspace conformity

# Property tests only
cargo test --workspace -- property

# Integration tests (simulator)
cargo test --workspace -- integration

# Hardware smoke (nightly, manual)
cargo test --workspace -- hardware --ignored
```

## Conventions

- **Conformance = executable spec** — every language RFC adds cases
- **Property tests = physical invariants** — not just example assertions
- **Snapshots reviewed** — never auto-commit snapshot updates
- **CI runs all levels** — see [TESTING_QA.md](../docs/TESTING_QA.md) §4
- **Benchmarks tracked** — compile-time, circuit quality, gradient cost

## Conformance Categories

| Category | Tests |
|----------|-------|
| `valid/` | Well-typed programs, expected types/GIR |
| `types/` | Type inference results, const generic solving |
| `effects/` | Effect rows, boundary crossing errors |
| `linearity/` | Use-after-consume, duplication, implicit drop, aliasing |
| `uncompute/` | Auto-uncompute success + E0530 refusal |
| `diff/` | Gradient lowering correctness |
| `qir/` | Profile selection, golden QIR |
| `run/` | End-to-end simulated outputs |
| `errors/` | Specific diagnostic codes with spans |

## Debugging

```bash
# Run single conformance test
cargo test --workspace conformity/types/my_test

# Run property test with output
cargo test --workspace -- property/unitary -- --nocapture

# Update snapshots
cargo test --workspace -- --update-snapshots

# Run integration test
cargo test --workspace -- integration_test
```