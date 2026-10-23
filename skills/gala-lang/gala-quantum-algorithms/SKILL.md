---
name: gala-quantum-algorithms
description: Implement quantum algorithms in Gala — QFT, Grover, phase estimation, adders, oracles
disable-model-invocation: true
allowed-tools: Bash(cargo run --bin gala *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Overview

Gala's standard library (`gala.algorithms`) provides building blocks, but you often need custom algorithms. This skill covers implementing core quantum algorithms with Gala's linear types and automatic uncomputation.

## Standard Library Algorithms

```gala
import gala.algorithms.{ qft, iqft, grover, phase_estimation, ripple_adder }
```

| Algorithm | Module | Properties |
|-----------|--------|------------|
| QFT/iQFT | `gala.algorithms.qft` | `#[property(unitary)]`, `#[property(reversible)]` |
| Grover | `gala.algorithms.grover` | `#[property(unitary)]` |
| Phase Estimation | `gala.algorithms.phase_est` | `#[property(unitary)]` |
| Ripple Adder | `gala.algorithms.arithmetic` | `#[property(uncomputes)]` |

## Writing Custom Algorithms

### 1. Quantum Fourier Transform (QFT)

```gala
import gala.gates.{ h, cx, rz }

fn qft(qs: Qubits<N>) -> Qubits<N> quantum {
    let n = N  // const generic
    for i in 0..n {
        h(qs[i])
        for j in (i+1)..n {
            // Controlled rotation: R_k where k = j-i+1
            let angle = 2.0 * PI / (2.0_f64.powi((j - i + 1) as Int))
            // Controlled RZ
            let (ctrl, target) = (qs[i], qs[j])
            // Use controlled-phase or decompose
            cp(ctrl, target, angle)
        }
    }
    // Swap for bit-reversal
    for i in 0..(n/2) {
        swap(qs[i], qs[n-1-i])
    }
    qs
}
```

### 2. Grover's Search

```gala
fn grover(oracle: fn(Qubits<N>) -> Qubits<N> quantum, iterations: Int) -> Qubits<N> quantum {
    let qs = qubits<N>()
    // Initial superposition
    for i in 0..N { h(qs[i]) }
    
    for _ in 0..iterations {
        // Oracle
        qs = oracle(qs)
        // Diffusion
        qs = diffusion(qs)
    }
    qs
}

fn diffusion(qs: Qubits<N>) -> Qubits<N> quantum {
    for i in 0..N { h(qs[i]); x(qs[i]) }
    // Multi-controlled Z
    mcz(qs)
    for i in 0..N { x(qs[i]); h(qs[i]) }
    qs
}
```

### 3. Reversible Arithmetic (with Auto-Uncomputation)

```gala
fn ripple_adder(a: Qubits<N>, b: Qubits<N>) -> (Qubits<N>, Qubits<N>) quantum {
    // Uses ancilla for carry — automatically uncomputed on scope exit
    let carry = qubit()  // Ancilla, auto-uncomputed at }
    // ... adder circuit using carry
    (a, b)  // carry goes out of scope → adjoint synthesized
}
```

**Key:** Ancilla `carry` is liftable → compiler inserts `adjoint(carry_provenance)` at `}`

### 4. Phase Estimation

```gala
fn phase_estimation(
    unitary: fn(Qubits<N>) -> Qubits<N> quantum,
    eigenstate: Qubits<N>,
    precision: Int
) -> (Qubits<precision>, Qubits<N>) quantum {
    let eval_reg = qubits<precision>()
    let state = eigenstate
    
    // Hadamards on evaluation register
    for i in 0..precision { h(eval_reg[i]) }
    
    // Controlled unitary powers
    for i in 0..precision {
        let power = 2^i
        controlled_unitary_power(eval_reg[i], state, unitary, power)
    }
    
    // Inverse QFT on eval register
    iqft(eval_reg)
    
    (eval_reg, state)
}
```

## Property Testing Your Algorithms

Add attributes to enable compiler-verified properties:

```gala
#[property(unitary)]        // Matrix is unitary
fn my_unitary_op(qs: Qubits<4>) -> Qubits<4> quantum { ... }

#[property(reversible)]     // adjoint(f) ∘ f = identity
fn my_reversible(qs: Qubits<4>) -> Qubits<4> quantum { ... }

#[property(uncomputes)]     // Ancillae return to |0⟩
fn my_with_ancilla(qs: Qubits<4>) -> Qubits<4> quantum { ... }

#[property(grad_matches)]   // grad matches finite difference
fn my_variational(qs: Qubits<4>, θ: Params) -> Qubits<4> quantum { ... }
```

Run: `gala test --property unitary`

## Uncomputation Patterns

### When Auto-Uncompute Works (Liftable)
```gala
fn liftable_example() -> Qubit quantum {
    let a = qubit()
    let b = qubit()
    cx(a, b)  // b entangled with a
    // b goes out of scope → compiler synthesizes adjoint(cx) to uncompute b
    a
}
```

### When It Fails (E0530) — Manual Strategy
```gala
fn non_liftable() -> Qubit quantum {
    let a = qubit()
    let m = measure(a)  // Measurement = irreversible
    // a consumed, cannot auto-uncompute anything depending on m
    // If you had ancilla depending on m:
    // uncompute_with(ancilla, manual_reverse_circuit)
    qubit()  // Fresh qubit
}
```

**Workarounds for E0530:**
1. Restructure to keep ancilla liftable (measure later)
2. Use `uncompute_with(ancilla, reverse_fn)` — explicit reverse circuit
3. Use `drop(ancilla)` only if you're sure it's |0⟩ (unsafe, requires proof)

## Circuit Optimization Tips

1. **Fuse adjacent single-qubit gates** — compiler does this via `gala-opt`
2. **Cancel adjacent self-inverse gates** — `h; h` → identity
3. **Merge rotations** — `rx(a); rx(b)` → `rx(a+b)`
4. **Minimize ancilla lifetime** — allocate late, drop early

## Running Algorithms

```bash
# Simulate with state-vector
gala run src/algorithm.gala --shots 10000

# Check properties
gala test src/algorithm.gala --property unitary

# Emit GIR for inspection
gala build src/algorithm.gala --emit gir

# Profile gate count/depth
gala build src/algorithm.gala --emit gir-opt
```