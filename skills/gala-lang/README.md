# Gala Language Skills Index

This directory contains skills for **writing Gala programs** — the language user perspective. For compiler development skills, see [`../gala/`](../gala/).

## Language Skills

| Skill | Description |
|-------|-------------|
| [`gala-language-basics`](gala-language-basics/SKILL.md) | Syntax, types, effects, linear qubits, basic operations |
| [`gala-quantum-algorithms`](gala-quantum-algorithms/SKILL.md) | QFT, Grover, phase estimation, adders, oracles |
| [`gala-vqa-programming`](gala-vqa-programming/SKILL.md) | Variational algorithms, encoders, ansätze, native `grad` |
| [`gala-debug-programs`](gala-debug-programs/SKILL.md) | Compiler diagnostics, REPL, simulator inspection, property tests |
| [`gala-hardware-backends`](gala-hardware-backends/SKILL.md) | Backend selection, capability checking, QIR, vendor SDKs |
| [`gala-performance`](gala-performance/SKILL.md) | Gate count, depth, gradient cost, compilation time |
| [`gala-noise-modeling`](gala-noise-modeling/SKILL.md) | Typed noise models, error mitigation, calibration |
| [`gala-testing`](gala-testing/SKILL.md) | Property testing, conformance, unit tests, CI |
| [`gala-cookbook`](gala-cookbook/SKILL.md) | Recipes: teleportation, QAOA, VQE, mid-circuit patterns |
| [`gala-python-interop`](gala-python-interop/SKILL.md) | PyO3 bridge, PyTorch, Jupyter, NumPy, async hardware |

## Quick Start

```bash
# New project
gala new my_project
cd my_project

# Write code in src/main.gala
# Type-check
gala check

# Run on simulator
gala run --shots 1000

# Property test
gala test --property unitary,reversible,uncomputes

# Format
gala fmt
```

## Core Language Concepts

### Linear Qubits (No-Cloning)
```gala
let q = qubit()      // Allocate
h(q)                 // Use (consumes + returns)
measure(q)           // Consume, get classical result
```

### Effects System
```gala
fn pure_fn() -> Int pure { 1 }           // Classical
fn quantum_fn(q: Qubit) -> Qubit quantum { h(q) }  // Unitary
fn prob_fn(q: Qubit) -> Measured<Bool> prob { measure(q) }  // Measures
```

**Lattice:** `pure ⊑ quantum ⊑ prob`

### Native Differentiation
```gala
fn circuit(θ: Params) -> Measured<Float> prob { ... }
let dθ = grad(circuit, wrt: θ)  // Parameter-shift + autodiff
```

### Automatic Uncomputation
```gala
fn with_ancilla() -> Qubit quantum {
    let anc = qubit()  // Auto-uncomputed at scope exit if liftable
    // ...
    qubit()  // anc dropped here → adjoint synthesized
}
```

## Error Codes Quick Reference

| Range | Category | Examples |
|-------|----------|----------|
| E01xx | Syntax | Unexpected token, unclosed comment |
| E02xx | Types | Mismatch, unknown type |
| E03xx | Effects | `prob` in `quantum` context |
| E04xx | Linearity | Use after consume (E0412), implicit drop (E0413), clone (E0414) |
| E05xx | Uncomputation | Cannot auto-uncompute (E0530) |
| E06xx | Backend | Capability mismatch, grad unsupported |

## Property Testing

```gala
#[property(unitary)]       // f†f = I
#[property(reversible)]    // adjoint(f) ∘ f = id
#[property(uncomputes)]    // Ancillae ≈ |0⟩
#[property(grad_matches)]  // grad ≈ finite diff
#[property(effect_honesty)]// No measure in quantum
```

Run: `gala test --property unitary,reversible,uncomputes,grad_matches`

## Backend Model

```gala
let backend = backend.simulator()           // State-vector
let backend = backend.noisy(depolarizing(0.01))  // Noisy sim
let backend = backend.hardware("ionq.aria")      // Real hardware

run(circuit, on: backend, shots: 1000)
```

## Standard Library

```gala
import gala.core.{ qubit, measure, reverse, adjoint, control, grad }
import gala.gates.{ h, x, y, z, s, t, rx, ry, rz, cx, cz, swap }
import gala.algorithms.{ qft, grover, phase_estimation, ripple_adder }
import gala.vqa.{ angle_encode, layered_ansatz, adam_optimizer, Params }
import gala.noise.{ depolarizing, NoiseModel }
```

## Documentation

- **Language Spec:** [`../../docs/LANGUAGE_SPEC.md`](../../docs/LANGUAGE_SPEC.md)
- **Type System:** [`../../docs/TYPE_SYSTEM.md`](../../docs/TYPE_SYSTEM.md)
- **Compiler Pipeline:** [`../../docs/COMPILER_PIPELINE.md`](../../docs/COMPILER_PIPELINE.md)
- **Backends & Runtime:** [`../../docs/BACKENDS_RUNTIME.md`](../../docs/BACKENDS_RUNTIME.md)
- **Standard Library:** [`../../docs/STANDARD_LIBRARY.md`](../../docs/STANDARD_LIBRARY.md)
- **Testing Guide:** [`../../docs/TESTING_QA.md`](../../docs/TESTING_QA.md)

## Editor Support

Install VS Code extension:
```bash
code --install-extension ../../extensions/vscode/
```

Features: syntax highlighting, LSP diagnostics, hover (types+circuit), go-to-def, snippets.

## Contributing

For compiler contributions, see [`../gala/gala-contributing`](../gala/gala-contributing/SKILL.md).