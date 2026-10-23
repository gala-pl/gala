---
name: gala-vqa-programming
description: Build variational quantum algorithms in Gala — encoders, ansätze, optimizers, native grad
disable-model-invocation: true
allowed-tools: Bash(cargo run --bin gala *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Overview

Gala has **native differentiation** (`grad`) that works end-to-end across hybrid quantum-classical code. This skill covers building VQA/VQE/QAOA programs with parameter-shift gradients.

## VQA Standard Library

```gala
import gala.vqa.{ angle_encode, amplitude_encode, layered_ansatz, hardware_efficient_ansatz }
import gala.vqa.{ gd_optimizer, adam_optimizer, spsa_optimizer, Params }
```

## Complete VQA Example

```gala
import gala.vqa.{ angle_encode, layered_ansatz, adam_optimizer, Params }
import gala.core.{ grad, measure }

// 1. Data encoding (classical → quantum)
fn encode(x: Vec<Float>) -> Qubits<4> quantum {
    let q = qubits<4>()
    angle_encode(q, x)  // Ry(x[i]) on each qubit
    q
}

// 2. Variational ansatz (parameterized quantum circuit)
fn ansatz(q: Qubits<4>, θ: Params) -> Qubits<4> quantum {
    layered_ansatz(q, θ, depth: 3)  // Ry + CZ layers
    // Or custom:
    // for layer in 0..θ.depth {
    //     for i in 0..4 { ry(q[i], θ[layer][i]) }
    //     for i in 0..3 { cx(q[i], q[i+1]) }
    // }
    q
}

// 3. Classifier (hybrid: quantum + measure)
fn classify(x: Vec<Float>, θ: Params) -> Measured<Bool> prob {
    let q = ansatz(encode(x), θ)
    measure(q[0])  // Measure first qubit
}

// 4. Native gradient (parameter-shift for θ, autodiff for x)
let dθ = grad(classify, wrt: θ)  // Returns fn(Vec<Float>, Params) -> Params

// 5. Training loop
fn train(dataset: [(Vec<Float>, Bool)], epochs: Int) -> Params {
    let mut θ = Params::random(4, 3)  // 4 qubits, 3 layers
    let opt = adam_optimizer(lr: 0.01)
    
    for epoch in 0..epochs {
        for (x, y) in dataset {
            let g = dθ(x, θ)  // Gradient wrt θ
            θ = opt.step(θ, g)
        }
    }
    θ
}
```

## Key Components

### Encoders
```gala
// Angle encoding: Ry(x[i]) on qubit i
angle_encode(q: Qubits<N>, x: Vec<Float>) -> Qubits<N> quantum

// Amplitude encoding: prepare state ∑ x[i]|i⟩ (requires normalization)
amplitude_encode(q: Qubits<N>, x: Vec<Float>) -> Qubits<N> quantum

// Basis encoding: |x⟩ for binary data
basis_encode(q: Qubits<N>, x: Vec<Bool>) -> Qubits<N> quantum
```

### Ansatz Zoo
```gala
// Layered: alternating Ry + entangling layer
layered_ansatz(q: Qubits<N>, θ: Params, depth: Int) -> Qubits<N> quantum

// Hardware-efficient: native gate set for target backend
hardware_efficient_ansatz(q: Qubits<N>, θ: Params, topology: Topology) -> Qubits<N> quantum

// QAOA: problem + mixer Hamiltonians
qaoa_ansatz(q: Qubits<N>, θ: Params, problem: Hamiltonian) -> Qubits<N> quantum

// Custom: build from gates
fn my_ansatz(q: Qubits<4>, θ: Params) -> Qubits<4> quantum {
    for i in 0..4 { ry(q[i], θ[0][i]) }
    cx(q[0], q[1]); cx(q[2], q[3])
    for i in 0..4 { rz(q[i], θ[1][i]) }
    q
}
```

### Optimizers
```gala
// Gradient descent
gd_optimizer(lr: Float) -> Optimizer

// Adam (adaptive moments)
adam_optimizer(lr: Float, beta1: Float, beta2: Float) -> Optimizer

// SPSA (simultaneous perturbation — 2 circuit evals/step)
spsa_optimizer(lr: Float, c: Float) -> Optimizer

// Usage
let opt = adam_optimizer(lr: 0.01)
θ = opt.step(θ, gradient)
```

## How `grad` Works

```
grad(f, wrt: θ) where f: (X, θ) → Measured<T>
```

1. **Quantum params (θ in rotation gates)** → **Parameter-shift rule**
   - Exact gradient: ∂f/∂θ = ½[f(θ+π/2) - f(θ-π/2)]
   - Compiler emits shifted circuit evaluations

2. **Classical params (x in preprocessing)** → **Reverse-mode autodiff**
   - Standard backprop through classical ops

3. **Boundary** → **Chain rule composition**
   - Quantum gradients flow into classical autodiff

## Gradient Batching (Runtime Optimization)

```gala
// Multiple gradient components batched into single submission
// Automatic in gala-runtime — no code changes needed

// Before: 2N submissions for N params (parameter-shift)
// After:  2 submissions (all +π/2 in one batch, all -π/2 in another)
```

Check: `gala run --bench-grad src/vqa.gala`

## Noisy Training

```gala
import gala.noise.{ depolarizing, NoiseModel }

fn noisy_classify(x: Vec<Float>, θ: Params, noise: NoiseModel) -> Measured<Bool> prob {
    let backend = backend.noisy(noise)
    let q = ansatz(encode(x), θ)
    // Runtime handles noisy execution
    measure(q[0])
}

// Train on noisy simulator → noise-resilient parameters
let noise = depolarizing(0.01)
let θ = train_noisy(dataset, noise)
```

## Property Testing VQA

```gala
#[property(grad_matches)]  // grad ≈ finite difference
fn test_ansatz_grad() {
    let x = [0.1, 0.2, 0.3, 0.4]
    let θ = Params::random(4, 2)
    classify(x, θ)
}
```

Run: `gala test --property grad_matches`

## Common Patterns

### Loss Functions
```gala
fn cross_entropy_loss(pred: Measured<Bool>, target: Bool) -> Float pure {
    let p = pred.expectation()  // P(true) ∈ [0,1]
    if target { -log(p) } else { -log(1.0 - p) }
}

fn mse_loss(pred: Measured<Float>, target: Float) -> Float pure {
    let diff = pred.expectation() - target
    diff * diff
}
```

### Data Loading
```gala
// Classical data stays classical until encode()
fn load_batch() -> [(Vec<Float>, Bool)] pure {
    // Read from file, preprocess — all pure
    ...
}
```

## Debugging VQA

```bash
# Check gradient correctness
gala run src/vqa.gala --check-grad

# Profile gradient cost (submissions per grad)
gala run src/vqa.gala --bench-grad

# Visualize circuit at iteration
gala repl --visualize src/vqa.gala

# Compare noisy vs ideal
gala run src/vqa.gala --noise depolarizing:0.01 --shots 10000
```