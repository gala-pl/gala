---
name: gala-debug-programs
description: Debug Gala programs — compiler diagnostics, simulator inspection, REPL visualization, property test failures
disable-model-invocation: true
allowed-tools: Bash(cargo run --bin gala *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Overview

Debugging Gala programs involves understanding compiler diagnostics (which teach quantum principles), inspecting simulator state, and using the REPL for interactive exploration.

## Compiler Diagnostics

### Read the Error — It Teaches Physics

```bash
gala check src/main.gala
```

**Example: Use after measure (E0412)**
```
error[E0412]: qubit `a` used after measurement
  ┌─ src/main.gala:8:5
6 │     let r = measure(a)
  │             ---------- `a` is consumed here (measurement is destructive)
8 │     cx(a, b)
  │        ^ `a` used here after it was consumed
  = help: a qubit is a linear resource — measuring it ends its life.
          If you need its value afterward, use the classical result `r`.
```

**Fix:** Use `r` (the `Measured<Bool>`) instead of `a`.

### Get Full Explanation

```bash
gala explain E0412
gala explain E0412 --markdown  # For docs
```

### JSON Output for Tools

```bash
gala check --json src/main.gala | jq '.diagnostics[] | {code, message, help}'
```

## Common Error Categories

| Code Range | Category | Typical Cause |
|------------|----------|---------------|
| E01xx | Syntax | Missing semicolon, unclosed block |
| E02xx | Types | Type mismatch, unknown type |
| E03xx | Effects | `prob` in `quantum` function |
| E04xx | Linearity | Use after consume, implicit drop, clone |
| E05xx | Uncomputation | Ancilla not liftable (E0530) |
| E06xx | Backend/Diff | Capability mismatch, grad not supported |

### E0530: Cannot Auto-Uncompute

```
error[E0530]: cannot auto-uncompute ancilla `anc`
  ┌─ src/adder.gala:15:3
15 │     }  // anc goes out of scope here
  │    ^^
  = help: ancilla `anc` depends on measurement result `m`.
          Automatic uncomputation requires liftable (reversible) provenance.
          Use `uncompute_with(anc, manual_reverse)` or restructure to measure later.
```

**Fix options:**
1. Move measurement after ancilla drop
2. Add explicit `uncompute_with(anc, reverse_fn)`
3. Make ancilla liftable (remove measurement dependency)

## REPL for Interactive Debugging

```bash
gala repl
```

```gala
gala> let q = qubit()
gala> h(q)
gala> :state  # Dump state vector (small qubit counts)
gala> :circuit  # Show circuit diagram
gala> measure(q)
Measured<Bool> { distribution: { false: 0.5, true: 0.5 } }
gala> :help
```

**REPL Commands:**
- `:state` — Full state vector (amplitudes)
- `:bloch <qubit>` — Bloch sphere coordinates
- `:circuit` — ASCII circuit diagram
- `:types` — Show types of bindings
- `:effects` — Show effects of functions
- `:grad <fn> wrt <param>` — Compute gradient
- `:quit` — Exit

## Simulator Inspection

### Dump State Vector
```bash
gala run src/main.gala --shots 1 --dump-state
```

Output:
```
State vector (4 qubits):
|0000⟩: 0.5+0.0i
|0011⟩: 0.5+0.0i
|1100⟩: 0.5+0.0i
|1111⟩: 0.5+0.0i
```

### Deterministic Reproduction
```bash
gala run src/main.gala --seed 42 --shots 1000
# Same seed = same results
```

### Compare Noisy vs Ideal
```bash
# Ideal
gala run src/main.gala --shots 10000 --seed 42 > ideal.txt

# Noisy
gala run src/main.gala --shots 10000 --seed 42 --noise depolarizing:0.01 > noisy.txt

# Diff
diff ideal.txt noisy.txt
```

## Property Test Failures

```bash
gala test --property unitary src/algorithm.gala
```

**Failure output:**
```
FAILED: property(unitary) on my_circuit
  Counterexample: input state |01⟩
  f†f = [[1, 0], [0, 0.99]]  // Not identity!
  Shrinking...
  Minimal: single CX gate missing adjoint
```

**Debug steps:**
1. Check if all gates are unitary
2. Verify no measurement in `quantum` function
3. Ensure adjoint derived correctly

### Grad Matches Failure
```
FAILED: property(grad_matches) on variational
  Parameter θ[0][1]: analytic=0.123, finite-diff=0.125 (tol=1e-3)
  Max relative error: 1.6%
```

**Debug:**
- Increase shots for finite difference
- Check parameter-shift implementation
- Verify no classical side effects in quantum function

## Circuit Visualization

### In REPL
```gala
gala> :circuit
q0: ──H──●──M──
         │
q1: ─────X────
```

### From Compiler
```bash
gala build src/main.gala --emit gir | gala-viz circuit
```

### In VS Code (with extension)
- Hover over function → inline circuit diagram
- `Gala: Show Circuit` command

## GIR Inspection

```bash
# Human-readable
gala build src/main.gala --emit gir

# JSON for tools
gala build src/main.gala --emit gir=json --gir-version 1 | jq
```

**Look for:**
- Effect annotations on each op
- Linear value flow (ownership edges)
- Uncompute insertions (adjoint ops)

## LSP Diagnostics (VS Code)

Install extension: `code --install-extension extensions/vscode/`

Features:
- **Red squiggles** = errors (E04xx, E05xx)
- **Yellow squiggles** = warnings
- **Hover** = type + effect + circuit diagram
- **Go to definition** = jumps to gate/algorithm source
- **Code actions** = suggested fixes (E0412, E0530)

## Debugging Checklist

1. **`gala check`** — catches all compile-time errors
2. **`gala explain <code>`** — understand the quantum principle
3. **`gala repl`** — interactive state/circuit inspection
4. **`gala run --dump-state`** — verify amplitudes
5. **`gala test --property`** — verify physical invariants
6. **`gala build --emit gir`** — inspect lowered IR
7. **VS Code LSP** — live diagnostics + hover