---
name: gala-testing
description: Test Gala programs — property testing, conformance, unit tests, CI integration
disable-model-invocation: true
allowed-tools: Bash(cargo run --bin gala *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Overview

Testing in Gala is a **language feature**. The compiler provides quantum-aware property testing that verifies physical invariants, not just example assertions.

## Property Testing (Signature Feature)

Add attributes to functions to enable automatic property verification:

```gala
#[property(unitary)]        // f†f = I (for quantum functions)
#[property(reversible)]     // adjoint(f) ∘ f = identity
#[property(uncomputes)]     // Ancillae return to |0⟩
#[property(grad_matches)]   // grad ≈ finite difference
#[property(effect_honesty)] // quantum fn has no measurement
```

### Example: Testing a Circuit

```gala
// src/adder.gala
#[property(unitary)]
#[property(reversible)]
#[property(uncomputes)]
fn ripple_adder(a: Qubits<4>, b: Qubits<4>) -> (Qubits<4>, Qubits<4>) quantum {
    // ... adder circuit with ancilla ...
    (a, b)  // ancilla auto-uncomputed
}

// Run property tests
gala test src/adder.gala --property unitary
gala test src/adder.gala --property reversible
gala test src/adder.gala --property uncomputes
```

### How It Works

1. **Compiler generates** randomized input states
2. **Simulator executes** circuit and properties
3. **Property checker** verifies invariant
4. **On failure:** shrinks to minimal counterexample

### Output
```
PASS: property(unitary) on ripple_adder
  Tested 100 random states, max ||f†f - I|| = 2.3e-15

FAIL: property(uncomputes) on bad_adder
  Counterexample: input |1010⟩|0000⟩
  Ancilla q4: |0.001⟩ + |0.999⟩  (should be |0⟩)
  Shrinking...
  Minimal: missing adjoint on q4
```

## Property Types

| Property | Checks | Applicable To |
|----------|--------|---------------|
| `unitary` | Matrix is unitary | `quantum` functions |
| `reversible` | `adjoint(f)(f(x)) ≈ x` | `quantum` functions |
| `uncomputes` | Ancillae ≈ \|0⟩ at scope exit | Functions with ancilla |
| `grad_matches` | `grad(f) ≈ finite_diff(f)` | Hybrid functions with `grad` |
| `effect_honesty` | No `measure` in `quantum` | `quantum` functions |

## Unit Testing

### Classical Pure Functions
```gala
// src/math.gala
fn add(a: Int, b: Int) -> Int pure { a + b }
fn mul(a: Int, b: Int) -> Int pure { a * b }

// Tests in same file or test/ directory
#[test]
fn test_add() {
    assert(add(2, 3) == 5)
    assert(add(-1, 1) == 0)
}

#[test]
fn test_mul() {
    assert(mul(3, 4) == 12)
}
```

Run: `gala test src/math.gala`

### Quantum Functions (with simulator)
```gala
#[test]
fn test_bell_pair() {
    let (a, b) = bell()
    let m1 = measure(a)
    let m2 = measure(b)
    // Perfect correlation
    assert(m1 == m2)  // With enough shots
}

// Run with shots
gala test src/bell_test.gala --shots 1000
```

## Conformance Testing

Gala includes a **conformance suite** (`tests/conformance/`) that defines expected language behavior. You can run it on your programs:

```bash
# Run full conformance suite
gala test --conformance

# Run specific category
gala test --conformance linearity
gala test --conformance effects
gala test --conformance uncompute
```

### Adding Conformance Cases

If you implement a new language feature, add cases:

```
tests/conformance/
  your_feature/
    valid_case.gala      # Should compile + run
    valid_case.expect    # Expected type/GIR/result
    error_case.gala      # Should produce E0xxx
    error_case.expect    # Expected error code
```

## Snapshot Testing

For deterministic outputs (GIR, QIR, diagnostics):

```bash
# Golden file testing
gala build src/main.gala --emit gir > tests/snapshots/main.gir.golden

# In CI: compare
gala build src/main.gala --emit gir | diff - tests/snapshots/main.gir.golden
```

### Using `insta` (Rust-side)
```rust
// In crate tests
#[test]
fn test_gir_emission() {
    let gir = compile_and_lower("src/main.gala");
    insta::assert_snapshot!(gir, @r###"
    // expected GIR
    "###);
}
```

Update: `cargo test --workspace -- --update-snapshots`

## CI Integration

```yaml
# .github/workflows/test.yml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Gala
        run: cargo install --path . --locked
      - name: Format check
        run: gala fmt --check .
      - name: Type check
        run: gala check .
      - name: Property tests
        run: gala test --property unitary,reversible,uncomputes,grad_matches .
      - name: Unit tests
        run: gala test .
      - name: Conformance
        run: gala test --conformance
```

### JSON Output for CI
```bash
# Machine-readable results
gala test --json > test_results.json

# Parse in CI
cat test_results.json | jq '.tests[] | select(.passed == false)'
```

## Test Organization

```
project/
├── src/
│   ├── algorithm.gala       # Main code with #[property]
│   └── utils.gala
├── tests/
│   ├── unit/
│   │   └── test_utils.gala  # #[test] functions
│   ├── property/
│   │   └── test_algo.gala   # Property tests
│   └── integration/
│       └── test_e2e.gala    # Full program tests
└── gala.toml
```

## Running Tests

```bash
# All tests
gala test

# Specific file
gala test src/algorithm.gala

# Specific property
gala test src/algorithm.gala --property unitary

# With options
gala test --shots 1000 --seed 42 --property grad_matches

# Verbose
gala test --verbose

# Generate training corpus (for ML)
gala test --generate-corpus > corpus.jsonl
```

## Best Practices

1. **Property test every quantum function** — catches physics bugs
2. **Test uncomputation explicitly** — ancilla leaks are silent killers
3. **Use `grad_matches` for VQA** — verify gradients before training
4. **Run conformance on CI** — ensures language compliance
5. **Snapshot GIR/QIR** — catch compiler regressions
6. **Deterministic seeds** — `gala test --seed 42` for reproducibility

## Debugging Test Failures

```bash
# Property failure with details
gala test src/algo.gala --property unitary --verbose

# Shrink counterexample manually
gala repl
gala> :load src/algo.gala
gala> :state  # Inspect failing state

# Check GIR for property test
gala build src/algo.gala --emit gir --property-test unitary
```