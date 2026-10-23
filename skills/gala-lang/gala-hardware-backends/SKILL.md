---
name: gala-hardware-backends
description: Run Gala programs on quantum hardware — backend selection, capability checking, QIR emission, vendor SDKs
disable-model-invocation: true
allowed-tools: Bash(cargo run --bin gala *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Overview

Gala programs are **backend-agnostic**. You write once, run anywhere — simulator, noisy simulator, or real hardware (IBM, IonQ, Quantinuum, Braket, etc.) via QIR.

## Backend Model

```gala
fn main() {
    // Backend is a capability value — checked at compile time
    let backend = backend.simulator()              // State-vector
    // let backend = backend.noisy(depolarizing(0.01))  // Noisy sim
    // let backend = backend.hardware("ionq.aria")      // Real hardware
    // let backend = backend.hardware("ibm.osprey")     // IBM
    
    let result = run(classify, on: backend, shots: 1000)
}
```

**Key:** Same program, one-line backend change.

## Available Backends

| Backend | Command | Profile | Use Case |
|---------|---------|---------|----------|
| Simulator | `backend.simulator()` | Base | Development, exact results |
| Noisy Sim | `backend.noisy(model)` | Base | Error-aware development |
| IonQ Aria | `backend.hardware("ionq.aria")` | Adaptive | Trapped ion, all-to-all |
| Quantinuum | `backend.hardware("quantinuum.h1")` | Adaptive | Trapped ion, mid-circuit |
| IBM Osprey | `backend.hardware("ibm.osprey")` | Base/Adaptive | Superconducting |
| AWS Braket | `backend.hardware("braket.rigetti")` | Base | Multiple vendors |

## Capability Checking (Compile-Time)

The compiler **rejects** programs that don't fit the backend:

```gala
// This requires mid-circuit measurement → needs Adaptive profile
fn adaptive_algo() -> Measured<Bool> prob {
    let q = qubit()
    h(q)
    let m = measure(q)  // Mid-circuit!
    if m { x(q) }       // Feedback
    measure(q)
}

// Compile for base-profile backend → ERROR
let backend = backend.hardware("ibm.osprey")  // Base profile only
run(adaptive_algo, on: backend)  // E0601: capability mismatch
```

**Error:**
```
error[E0601]: backend capability mismatch
  ┌─ src/main.gala:10:5
10 │     run(adaptive_algo, on: backend)
   │     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   = help: program requires Adaptive profile (mid-circuit measurement)
           but backend 'ibm.osprey' only supports Base profile.
           Use a backend with mid-circuit support (ionq.aria, quantinuum.h1)
           or restructure to defer measurement.
```

## QIR Emission

```bash
# Emit QIR (LLVM IR) for inspection or manual submission
gala build src/main.gala --emit qir

# Output: main.ll (LLVM IR text) or main.bc (bitcode)
```

### Profile Selection
- **Base profile** — straight-line circuits, terminal measurement
- **Adaptive profile** — mid-circuit measurement, feedback, `if measure(q) { ... }`

Compiler chooses automatically based on program structure.

## Running on Hardware

### Prerequisites
```bash
# Configure credentials (once)
gala config set ionq.api_key YOUR_KEY
gala config set ibm.token YOUR_TOKEN
# Or use environment variables
export IONQ_API_KEY=...
export IBM_QUANTUM_TOKEN=...
```

### Submit Job
```bash
# Simulator (local, instant)
gala run src/main.gala --backend simulator --shots 1000

# Hardware (queued, async)
gala run src/main.gala --backend hardware:ionq.aria --shots 1000

# Poll for results
gala run src/main.gala --backend hardware:ionq.aria --shots 1000 --wait

# Or get job ID and check later
gala run src/main.gala --backend hardware:ionq.aria --shots 1000 --async
# Returns job ID
gala job status <job-id>
gala job results <job-id>
```

## Noise-Aware Development

```gala
import gala.noise.{ depolarizing, amplitude_damping, NoiseModel }

// Define noise model
let noise = depolarizing(0.01)  // 1% depolarizing

// Test on noisy simulator first
let backend = backend.noisy(noise)
let result = run(algorithm, on: backend, shots: 10000)

// Compare with ideal
let ideal = run(algorithm, on: backend.simulator(), shots: 10000)
```

## Gradient on Hardware

```gala
// grad works on hardware — runtime batches parameter-shift evaluations
let dθ = grad(classify, wrt: θ)

// Runtime automatically:
// 1. Generates shifted circuits (θ ± π/2)
// 2. Batches into minimal submissions
// 3. Submits to hardware
// 4. Assembles gradient from results

// Check batching efficiency
gala run src/vqa.gala --backend hardware:ionq.aria --bench-grad
```

## Vendor-Specific Features

### IonQ (Native all-to-all, adaptive)
```gala
// Full connectivity — no SWAP routing needed
// Supports mid-circuit measurement natively
```

### IBM (Heavy-hex, base/adaptive)
```gala
// Requires routing for non-adjacent qubits
// Compiler inserts SWAPs automatically (target lowering)
```

### Quantinuum (All-to-all, adaptive, high fidelity)
```gala
// Best for algorithms needing mid-circuit measurement
// Low error rates
```

## Target Lowering (Automatic)

```bash
# See routed circuit
gala build src/main.gala --emit gir --target ionq.aria

# See QIR for specific backend
gala build src/main.gala --emit qir --target ibm.osprey
```

**Pipeline:** GIR → Target Lowering (routing, decomposition, scheduling) → QIR

## CI/CD for Hardware

```yaml
# .github/workflows/hardware.yml
jobs:
  hardware-smoke:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run on IonQ (nightly)
        if: github.event_name == 'schedule'
        env:
          IONQ_API_KEY: ${{ secrets.IONQ_API_KEY }}
        run: |
          gala run src/bell.gala --backend hardware:ionq.aria --shots 100
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `E0601` capability mismatch | Check backend profile (Base vs Adaptive) |
| Job stuck in queue | Use `--async` + poll, or check vendor status |
| High error rates | Test on noisy simulator first, adjust noise model |
| Gradient too slow | Enable batching, reduce shots, use SPSA |
| QIR validation fails | Check `gala build --validate-qir` |

## Next Steps

- **Noise modeling** → `gala-noise-modeling`
- **Target lowering details** → `gala-target-lowering`
- **Benchmarking** → `gala-benchmarking`