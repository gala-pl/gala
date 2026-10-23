---
name: gala-noise-modeling
description: Model and simulate noise in Gala — typed noise models, QuEST noise channels, error mitigation
disable-model-invocation: true
allowed-tools: Bash(cargo run --bin gala *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Overview

Gala treats **noise models as typed first-class values** (`gala.noise`). This enables compile-time noise-aware development and seamless transition from ideal → noisy simulator → hardware.

## Noise Model Types

```gala
import gala.noise.{ NoiseModel, depolarizing, amplitude_damping, phase_damping, thermal_relaxation, custom }
```

### Built-in Models
```gala
// Single-qubit depolarizing: ρ → (1-p)ρ + p/3 (XρX + YρY + ZρZ)
depolarizing(prob: Float) -> NoiseModel

// Amplitude damping (T1 decay): |1⟩ → |0⟩ with prob γ
amplitude_damping(gamma: Float) -> NoiseModel

// Phase damping (T2 dephasing): off-diagonal decay
phase_damping(gamma: Float) -> NoiseModel

// Combined T1/T2 thermal relaxation
thermal_relaxation(t1: Float, t2: Float, gate_time: Float) -> NoiseModel
```

### Composite Models
```gala
// Per-qubit noise (different per qubit)
let noise = NoiseModel::per_qubit([
    depolarizing(0.001),   // q0
    depolarizing(0.002),   // q1
    amplitude_damping(0.01), // q2
])

// Per-gate noise (different per gate type)
let noise = NoiseModel::per_gate({
    "cx": depolarizing(0.01),
    "rz": phase_damping(0.001),
    "measure": NoiseModel::readout_error(0.02, 0.03),  // P(0|1), P(1|0)
})

// Spatially correlated (crosstalk)
let noise = NoiseModel::correlated([
    (depolarizing(0.001), [0, 1]),  // Correlated on q0,q1
])
```

## Using Noise Models

### Noisy Simulator Backend
```gala
fn main() {
    let noise = depolarizing(0.01)
    let backend = backend.noisy(noise)
    let result = run(algorithm, on: backend, shots: 10000)
}
```

### CLI
```bash
# Depolarizing
gala run src/main.gala --noise depolarizing:0.01 --shots 10000

# Amplitude damping
gala run src/main.gala --noise amplitude_damping:0.05

# From file
gala run src/main.gala --noise-file noise_model.toml
```

### Noise Model File (TOML)
```toml
# noise_model.toml
[global]
depolarizing = 0.001

[per_qubit]
0 = { depolarizing = 0.0005 }
1 = { amplitude_damping = 0.01 }

[per_gate]
"cx" = { depolarizing = 0.01 }
"measure" = { readout_error = [0.02, 0.03] }
```

## Error Mitigation

### Zero-Noise Extrapolation (ZNE)
```gala
import gala.mitigation.{ zne_extrapolate, richardson_extrapolate }

fn mitigate_with_zne(circuit: fn() -> Measured<Bool> prob, noise_scales: [Float]) -> Float {
    // Run at different noise levels (stretch gates)
    let expectations = noise_scales.map(|scale| {
        let scaled_noise = scale_noise(base_noise, scale)
        let backend = backend.noisy(scaled_noise)
        run(circuit, on: backend, shots: 1000).expectation()
    })
    // Extrapolate to zero noise
    richardson_extrapolate(noise_scales, expectations)
}
```

### Readout Error Mitigation
```gala
import gala.mitigation.{ readout_mitigation, calibration_matrix }

// Calibrate once per backend
let cal_matrix = calibration_matrix(backend.hardware("ionq.aria"))

// Apply to results
fn mitigate_readout(raw: Measured<Bool>, cal: Matrix) -> Measured<Bool> {
    readout_mitigation(raw, cal)
}
```

### Probabilistic Error Cancellation (PEC)
```gala
import gala.mitigation.{ pec_mitigate, quasiprobability_decomposition }

// Requires noise model with known inverse operations
fn pec_expectation(circuit: fn() -> Measured<Bool>, noise: NoiseModel) -> Float {
    let decomposition = quasiprobability_decomposition(noise)
    pec_mitigate(circuit, decomposition, shots: 50000)
}
```

## Noise-Aware Compilation

### Compiler Optimizations with Noise
```bash
# Enable noise-aware optimization (experimental)
gala build src/main.gala --opt-noise-aware --noise depolarizing:0.01
```

Effects:
- Prefers gates with lower error rates
- May increase depth to use better gates
- Schedules to minimize idle time (T1 decay)

### Gate Selection by Noise
```gala
// In noise model, specify native gates with error rates
let noise = NoiseModel::per_gate({
    "cx": 0.01,
    "cz": 0.005,  // CZ better than CX on this device
    "rz": 0.0,    // Virtual Z, no error
})

// Compiler may decompose CX → CZ + H if beneficial
```

## Characterizing Hardware Noise

### Calibration Experiments
```gala
import gala.calibration.{ t1_measurement, t2_measurement, rb_benchmark }

// T1 measurement
fn t1_circuit(qubit: Qubit, delay: Float) -> Measured<Bool> prob {
    x(qubit)
    wait(delay)
    measure(qubit)
}

// T2 (Ramsey)
fn t2_circuit(qubit: Qubit, delay: Float) -> Measured<Bool> prob {
    h(qubit)
    wait(delay)
    h(qubit)
    measure(qubit)
}

// Randomized Benchmarking
fn rb_sequence(qubits: Qubits<N>, depth: Int) -> Measured<Bool> prob {
    rb_benchmark(qubits, depth)
}
```

Run: `gala run calibrate.gala --backend hardware:ionq.aria --shots 1000`

## Noise in VQA Training

```gala
// Train on noisy simulator → noise-resilient parameters
fn train_noise_aware(dataset, noise: NoiseModel) -> Params {
    let backend = backend.noisy(noise)
    let mut θ = Params::random(...)
    let opt = adam_optimizer(lr: 0.01)
    
    for epoch in 0..epochs {
        for (x, y) in dataset {
            // Forward + backward on noisy backend
            let loss = loss_fn(x, y, θ, backend)
            let g = grad(loss, wrt: θ)  // Runtime handles noisy grad
            θ = opt.step(θ, g)
        }
    }
    θ
}
```

## Verification

```bash
# Compare ideal vs noisy
gala run src/algo.gala --shots 10000 --seed 42 > ideal.txt
gala run src/algo.gala --shots 10000 --seed 42 --noise depolarizing:0.01 > noisy.txt

# Fidelity
gala fidelity ideal.txt noisy.txt

# Property test: noise model correctness
gala test --property noise_model src/noise_test.gala
```

## Roadmap

- **Error budget tracking in types** — `Qubit<ErrorBudget>` (see ROADMAP.md)
- **Dynamic decoupling insertion** — compiler pass
- **Mitigation as compiler pass** — automatic ZNE/PEC
- **Hardware noise model import** — from vendor calibration data