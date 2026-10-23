---
name: gala-language-basics
description: Learn Gala language fundamentals — syntax, types, effects, linear qubits, basic quantum operations
disable-model-invocation: true
allowed-tools: Bash(cargo run --bin gala *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Overview

Gala is a **hybrid quantum-classical language** where quantum mechanics laws are enforced at compile time. This skill covers the core language concepts you need to write Gala programs.

## Key Concepts

### 1. Linear Qubits (No-Cloning)
```gala
// Qubits are linear — must be used exactly once
let q = qubit()          // Allocate |0⟩
h(q)                     // Apply Hadamard (in-place sugar)
// q is consumed by h, rebound to result
measure(q)               // Consumes q, returns Measured<Bool>
```

**Rules:**
- No duplication: `let q2 = q` ❌ (compile error E0413)
- No implicit drop: every qubit must be `measure`d, `return`ed, or `drop`ped
- No aliasing: can't have two references to same qubit

### 2. Effects System (Three Worlds)
```gala
fn pure_fn(x: Int) -> Int pure { x + 1 }           // Classical only
fn quantum_fn(q: Qubit) -> Qubit quantum { h(q) }  // Unitary, reversible
fn prob_fn(q: Qubit) -> Measured<Bool> prob { measure(q) }  // Measures
```

**Effect lattice:** `pure ⊑ quantum ⊑ prob`
- `quantum` can call `pure`
- `prob` can call `quantum` and `pure`
- `quantum` **cannot** call `prob` (measurement boundary)

### 3. Core Types
| Type | Linear? | Description |
|------|---------|-------------|
| `Qubit` | Yes | Single qubit |
| `Qubits<N>` | Yes | Register of N qubits (const generic) |
| `Measured<T>` | No | Classical result with distribution |
| `Bool`, `Int`, `Float`, `Complex` | No | Classical scalars |
| `Params` | No | Trainable parameters for VQA |

### 4. Basic Quantum Operations
```gala
// Allocation
qubit() -> Qubit quantum
qubits<N>() -> Qubits<N> quantum

// Single-qubit gates (in-place sugar)
h(q)      // Hadamard
x(q), y(q), z(q)  // Pauli
s(q), t(q)        // Phase
rx(q, θ), ry(q, θ), rz(q, θ)  // Rotations

// Two-qubit gates
cx(ctrl, target) -> (Qubit, Qubit) quantum  // CNOT
cz(ctrl, target) -> (Qubit, Qubit) quantum  // CZ
swap(a, b) -> (Qubit, Qubit) quantum

// Measurement
measure(q: Qubit) -> Measured<Bool> prob

// Safe release (auto-uncomputes if liftable)
drop(q: Qubit) quantum
```

### 5. In-Place Sugar
```gala
// Explicit (always valid)
let q = h(q)
let (a, b) = cx(a, b)

// Sugar (compiler rewrites to explicit)
h(q)                    // → let q = h(q)
cx(a, b)                // → let (a, b) = cx(a, b)
```

## Quick Start

```bash
# Create new project
gala new my_project
cd my_project

# Write program in src/main.gala
# Run on simulator
gala run --shots 1000

# Type-check only
gala check

# Format
gala fmt
```

## Hello World: Bell Pair

```gala
// src/main.gala
fn bell() -> Qubits<2> quantum {
    let a = qubit()
    let b = qubit()
    h(a)
    cx(a, b)
    (a, b)
}

fn main() -> Measured<Bool> prob {
    let (a, b) = bell()
    measure(a)
}
```

Run: `gala run --shots 1000` → ~50% `true`, ~50% `false` (entangled)

## Common Patterns

### Qubit Register Operations
```gala
fn hadamard_all(qs: Qubits<4>) -> Qubits<4> quantum {
    for i in 0..4 {
        h(qs[i])  // In-place on each qubit
    }
    qs
}
```

### Classical Control from Measurement
```gala
fn measure_and_branch(q: Qubit) -> Int prob {
    let m = measure(q)
    if m { 1 } else { 0 }  // Classical branch on Measured<Bool>
}
```

### Parameterized Gates (for VQA)
```gala
fn ry_layer(qs: Qubits<4>, θ: Params) -> Qubits<4> quantum {
    for i in 0..4 {
        ry(qs[i], θ[i])
    }
    qs
}
```

## Error Codes You'll See

| Code | Meaning |
|------|---------|
| E0412 | Qubit used after measurement/consumption |
| E0413 | Qubit implicitly dropped (add `drop(q)` or `measure(q)`) |
| E0414 | Qubit duplicated (no cloning!) |
| E0301 | `prob` function called from `quantum` context |
| E0530 | Cannot auto-uncompute ancilla (needs manual `uncompute_with`) |

## Next Steps

- **Quantum algorithms** → `gala-quantum-algorithms`
- **Variational ML** → `gala-vqa-programming`
- **Debugging** → `gala-debug-programs`
- **Hardware** → `gala-hardware-backends`