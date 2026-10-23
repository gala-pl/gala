---
name: gala-cookbook
description: Gala recipes — common patterns, idioms, algorithms, VQA templates, error handling
disable-model-invocation: true
allowed-tools: Bash(cargo run --bin gala *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Overview

Practical Gala patterns and recipes for common tasks. Copy-paste starting points for real programs.

## Recipe Index

1. [Bell Pair & Entanglement](#bell-pair--entanglement)
2. [Quantum Teleportation](#quantum-teleportation)
3. [Superdense Coding](#superdense-coding)
4. [Phase Estimation](#phase-estimation)
5. [Variational Classifier](#variational-classifier)
6. [QAOA for MaxCut](#qaoa-for-maxcut)
7. [VQE for Molecular Hamiltonian](#vqe-for-molecular-hamiltonian)
8. [Error Mitigation Wrapper](#error-mitigation-wrapper)
9. [Classical Pre/Post Processing](#classical-prepost-processing)
10. [Mid-Circuit Measurement Patterns](#mid-circuit-measurement-patterns)
11. [Ancilla Management](#ancilla-management)
12. [Parameter Sharing](#parameter-sharing)

---

## Bell Pair & Entanglement

```gala
fn bell_pair() -> Qubits<2> quantum {
    let a = qubit()
    let b = qubit()
    h(a)
    cx(a, b)
    (a, b)
}

// GHZ state (N-qubit entanglement)
fn ghz_state<const N: Int>() -> Qubits<N> quantum {
    let qs = qubits<N>()
    h(qs[0])
    for i in 1..N {
        cx(qs[0], qs[i])
    }
    qs
}

// W state (different entanglement)
fn w_state<const N: Int>() -> Qubits<N> quantum {
    let qs = qubits<N>()
    x(qs[0])  // Start with |100...0⟩
    for i in 1..N {
        // Controlled rotation to spread amplitude
        let theta = 2.0 * asin(1.0 / sqrt((N - i + 1) as Float))
        cry(qs[i-1], qs[i], theta)
    }
    qs
}
```

## Quantum Teleportation

```gala
fn teleport(msg: Qubit, alice: Qubit, bob: Qubit) -> (Qubit, Qubit, Measured<Bool>, Measured<Bool>) prob {
    // Alice prepares Bell pair with Bob
    h(alice)
    cx(alice, bob)
    
    // Alice measures msg + her half
    cx(msg, alice)
    h(msg)
    let m1 = measure(msg)
    let m2 = measure(alice)
    
    // Bob applies corrections (classical feedforward)
    if m2 { x(bob) }  // X correction
    if m1 { z(bob) }  // Z correction
    
    (bob, msg, m1, m2)  // bob now has msg state
}
```

## Superdense Coding

```gala
fn superdense_encode(alice: Qubit, bob: Qubit, bits: (Bool, Bool)) -> (Qubit, Qubit) quantum {
    // Pre-shared Bell pair
    h(alice)
    cx(alice, bob)
    
    // Alice encodes 2 classical bits
    match bits {
        (false, false) => {},           // |00⟩: I
        (true, false) => { x(alice) },  // |10⟩: X
        (false, true) => { z(alice) },  // |01⟩: Z
        (true, true) => { x(alice); z(alice) },  // |11⟩: XZ
    }
    (alice, bob)
}

fn superdense_decode(alice: Qubit, bob: Qubit) -> (Measured<Bool>, Measured<Bool>) prob {
    // Bob decodes
    cx(alice, bob)
    h(alice)
    let b1 = measure(alice)
    let b2 = measure(bob)
    (b1, b2)
}
```

## Phase Estimation

```gala
fn phase_estimation<const N: Int, const P: Int>(
    unitary: fn(Qubits<N>) -> Qubits<N> quantum,
    eigenstate: Qubits<N>
) -> (Qubits<P>, Qubits<N>) quantum {
    let eval = qubits<P>()
    let state = eigenstate
    
    // Hadamards on evaluation register
    for i in 0..P { h(eval[i]) }
    
    // Controlled-U^(2^i)
    for i in 0..P {
        let power = 1 << i  // 2^i
        controlled_unitary_power(eval[i], state, unitary, power)
    }
    
    // Inverse QFT
    iqft(eval)
    
    (eval, state)
}

// Helper: controlled unitary power
fn controlled_unitary_power<const N: Int>(
    ctrl: Qubit,
    target: Qubits<N>,
    u: fn(Qubits<N>) -> Qubits<N> quantum,
    power: Int
) -> (Qubit, Qubits<N>) quantum {
    // Repeat controlled-U power times
    // (Compiler may optimize for specific unitaries)
    let mut t = target
    for _ in 0..power {
        // Controlled version of u
        let (c, t_new) = controlled_u(ctrl, t, u)
        t = t_new
    }
    (ctrl, t)
}
```

## Variational Classifier

```gala
import gala.vqa.{ angle_encode, layered_ansatz, adam_optimizer, Params, grad }

fn encode(x: Vec<Float>) -> Qubits<4> quantum {
    let q = qubits<4>()
    angle_encode(q, x)
    q
}

fn ansatz(q: Qubits<4>, θ: Params) -> Qubits<4> quantum {
    layered_ansatz(q, θ, depth: 3)
    q
}

fn classify(x: Vec<Float>, θ: Params) -> Measured<Bool> prob {
    let q = ansatz(encode(x), θ)
    measure(q[0])
}

// Native gradient!
let dθ = grad(classify, wrt: θ)

// Training
fn train(dataset: [(Vec<Float>, Bool)], epochs: Int) -> Params {
    let mut θ = Params::random(4, 3)
    let opt = adam_optimizer(lr: 0.01)
    
    for epoch in 0..epochs {
        for (x, y) in dataset {
            let g = dθ(x, θ)
            θ = opt.step(θ, g)
        }
    }
    θ
}

// Evaluate
fn accuracy(model: Params, test_set: [(Vec<Float>, Bool)]) -> Float pure {
    let mut correct = 0
    for (x, y) in test_set {
        let pred = classify(x, model)
        let p = pred.expectation()
        let predicted = p > 0.5
        if predicted == y { correct += 1 }
    }
    correct as Float / test_set.len() as Float
}
```

## QAOA for MaxCut

```gala
import gala.vqa.{ qaoa_ansatz, Params, grad }
import gala.gates.{ h, cx, rz }

// Problem: MaxCut on graph
struct Graph { edges: [(Int, Int)] }

fn maxcut_qaoa(graph: Graph, p: Int) -> fn(Params) -> Measured<Float> prob {
    let n = graph.edges.iter().flat_map(|(u,v)| [*u, *v]).max().unwrap() + 1
    
    // Cost Hamiltonian: ∑ (1 - Z_i Z_j)/2
    // Mixer: ∑ X_i
    
    fn circuit(θ: Params) -> Measured<Float> prob {
        let qs = qubits<n>()
        
        // Initial superposition
        for i in 0..n { h(qs[i]) }
        
        // QAOA layers
        for layer in 0..p {
            // Cost layer
            for (u, v) in graph.edges {
                rz(qs[u], θ[layer][0])
                rz(qs[v], θ[layer][0])
                cx(qs[u], qs[v])
                rz(qs[v], -θ[layer][0])
                cx(qs[u], qs[v])
            }
            // Mixer layer
            for i in 0..n {
                rx(qs[i], θ[layer][1])
            }
        }
        
        // Measure in Z basis, compute cut value
        let mut cut = 0.0
        for (u, v) in graph.edges {
            let mu = measure(qs[u])
            let mv = measure(qs[v])
            if mu != mv { cut += 1.0 }
        }
        Measured<Float>::new(cut)  // Simplified
    }
    
    circuit
}
```

## VQE for Molecular Hamiltonian

```gala
import gala.vqa.{ hardware_efficient_ansatz, Params, grad }
import gala.chemistry.{ molecular_hamiltonian, jordan_wigner }

fn vqe_h2o(bond_length: Float) -> Params {
    // 1. Build molecular Hamiltonian (classical)
    let hamiltonian = molecular_hamiltonian("H2O", geometry: bond_length)
    let qubit_hamiltonian = jordan_wigner(hamiltonian)  // → Pauli string sum
    
    // 2. Ansatz (hardware-efficient for NISQ)
    let n_qubits = qubit_hamiltonian.num_qubits()
    
    fn energy(θ: Params) -> Measured<Float> prob {
        let qs = qubits<n_qubits>()
        hardware_efficient_ansatz(qs, θ, topology: "linear")
        
        // Measure Pauli strings for expectation
        let mut energy = 0.0
        for (pauli_string, coeff) in qubit_hamiltonian.terms {
            let basis = pauli_string_to_basis(pauli_string)
            let expval = measure_in_basis(qs, basis).expectation()
            energy += coeff * expval
        }
        Measured<Float>::new(energy)
    }
    
    // 3. Optimize
    let dE = grad(energy, wrt: θ)
    let mut θ = Params::random(n_qubits, depth: 4)
    let opt = adam_optimizer(lr: 0.05)
    
    for _ in 0..100 {
        let g = dE(θ)
        θ = opt.step(θ, g)
    }
    θ
}
```

## Error Mitigation Wrapper

```gala
import gala.mitigation.{ zne_extrapolate, readout_mitigation }

fn run_with_mitigation<F, T>(
    circuit: F,
    backend: Backend,
    noise: NoiseModel,
    shots: Int
) -> Measured<T> prob
where F: fn() -> Measured<T> prob {
    
    // 1. Zero-Noise Extrapolation
    let noise_scales = [1.0, 1.5, 2.0, 3.0]
    let expectations = noise_scales.map(|scale| {
        let scaled = noise.scale(scale)
        let noisy_backend = backend.noisy(scaled)
        run(circuit, on: noisy_backend, shots: shots).expectation()
    })
    let zne_value = zne_extrapolate(noise_scales, expectations)
    
    // 2. Readout Mitigation (calibrate separately)
    let cal_matrix = calibrate_readout(backend)
    let mitigated = readout_mitigation(Measured::new(zne_value), cal_matrix)
    
    mitigated
}
```

## Classical Pre/Post Processing

```gala
// Pure classical preprocessing (compiles to native code)
fn preprocess(raw_data: String) -> Vec<Float> pure {
    // Parse, normalize, feature extract
    raw_data.split(",")
        .map(|s| s.trim().parse::<Float>().unwrap())
        .map(|x| (x - mean) / std)  // Normalize
        .collect()
}

// Pure classical postprocessing
fn postprocess(result: Measured<Bool>) -> String pure {
    let p = result.expectation()
    let label = if p > 0.5 { "CLASS_1" } else { "CLASS_0" }
    format!("Prediction: {} (confidence: {:.2})", label, p.max(1.0-p))
}

// Hybrid pipeline
fn pipeline(input: String) -> String prob {
    let features = preprocess(input)      // pure
    let θ = load_model()                  // pure (I/O at edges)
    let quantum_result = classify(features, θ)  // prob
    postprocess(quantum_result)           // pure
}
```

## Mid-Circuit Measurement Patterns

### Repeat-Until-Success
```gala
fn repeat_until_success(
    attempt: fn() -> (Qubit, Measured<Bool>) prob,
    max_tries: Int
) -> Qubit prob {
    for _ in 0..max_tries {
        let (q, success) = attempt()
        if success { return q }
        // q automatically dropped (uncomputed) on loop iteration
    }
    panic("Max retries exceeded")  // Or return error
}
```

### Adaptive Measurement
```gala
fn adaptive_phase_estimation(unitary: fn(Qubit) -> Qubit quantum) -> Float prob {
    let q = qubit()
    let mut phase = 0.0
    
    for i in 0..PRECISION {
        // Controlled-U^(2^i)
        controlled_power(q, unitary, 1 << i)
        
        // Measure and adapt
        h(q)
        let m = measure(q)
        phase += if m { 1.0 / (2.0_f64.powi(i + 1)) } else { 0.0 }
        
        // Prepare for next iteration
        if m { z(q) }  // Phase correction
    }
    phase
}
```

## Ancilla Management

### Scoped Ancilla (Auto-Uncompute)
```gala
fn with_ancilla<const N: Int>(
    f: fn(Qubits<N>, Qubit) -> Qubits<N> quantum
) -> fn(Qubits<N>) -> Qubits<N> quantum {
    |qs: Qubits<N>| -> Qubits<N> quantum {
        let anc = qubit()  // Allocated here
        let result = f(qs, anc)
        // anc dropped here → auto-uncomputed if liftable
        result
    }
}
```

### Manual Uncomputation (for E0530)
```gala
fn manual_uncompute_example() -> Qubit quantum {
    let a = qubit()
    let b = qubit()
    let anc = qubit()
    
    // ... circuit entangling anc with measurement ...
    let m = measure(b)  // Measurement makes anc non-liftable
    
    // Manual uncompute of anc
    uncompute_with(anc, |anc| {
        // Reverse of what entangled anc
        adjoint(cx)(a, anc)
        adjoint(h)(anc)
    })
    
    // Now anc is |0⟩, safe to drop
    drop(anc)
    a
}
```

## Parameter Sharing

```gala
// Tie parameters across layers (fewer params = cheaper grad)
fn shared_params_ansatz(q: Qubits<4>, θ: Params) -> Qubits<4> quantum {
    // θ has shape [2] instead of [depth, 4]
    for layer in 0..3 {
        for i in 0..4 {
            ry(q[i], θ[0])  // Same rotation angle all qubits
        }
        for i in 0..3 {
            rz(q[i], θ[1])  // Same phase all qubits
            cx(q[i], q[i+1])
        }
    }
    q
}
```

## Find More Recipes

- `library/std/algorithms.gala` — QFT, Grover, adders
- `library/std/vqa.gala` — Encoders, ansätze, optimizers
- `examples/` — Complete programs
- `tests/conformance/valid/` — Correct usage patterns

## Contributing Recipes

Add to `library/std/` or `examples/` with:
1. Property tests (`#[property(...)]`)
2. Documentation (`///`)
3. Benchmark in `gala bench`