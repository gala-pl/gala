---
name: gala-performance
description: Optimize Gala program performance — gate count, circuit depth, gradient cost, compilation time
disable-model-invocation: true
allowed-tools: Bash(cargo run --bin gala *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Overview

Performance in Gala has multiple dimensions: **circuit quality** (gates/depth), **gradient cost** (device submissions), **simulation speed**, and **compile time**.

## Circuit Optimization (gala-opt)

### Automatic Passes
```bash
# See optimized GIR
gala build src/main.gala --emit gir-opt

# Compare gate counts
gala build src/main.gala --emit gir  # Before
gala build src/main.gala --emit gir-opt  # After
```

### Key Optimizations
| Pass | Effect |
|------|--------|
| `AdjacentCancellation` | `H;H` → `I`, `X;X` → `I`, `CX(a,b);CX(a,b)` → `I` |
| `RotationMerge` | `RX(a);RX(b)` → `RX(a+b)` |
| `SingleQubitFusion` | `U1;U2;U3` → single `Unitary` |
| `TwoQubitCancellation` | Adjacent inverse pairs |
| `AncillaScheduling` | Reorder to minimize live ancillae |
| `ConstantFolding` | Classical: `2+3` → `5` |
| `DeadCodeElimination` | Remove unused pure computations |
| `InlineSmallPure` | Inline small `pure` functions |

### Manual Optimization Tips

**1. Fuse rotations yourself when possible:**
```gala
// Instead of:
rx(q, a); ry(q, b); rz(q, c)
// Use single unitary if available, or let optimizer fuse
```

**2. Minimize ancilla scope:**
```gala
// Good: ancilla allocated late, dropped early
fn good() {
    let a = qubit()
    let b = qubit()
    cx(a, b)
    b  // ancilla dropped here → auto-uncomputed
}

// Avoid: ancilla lives longer than needed
fn bad() {
    let anc = qubit()  // Lives through entire function
    // ... many ops ...
    drop(anc)  // Late drop = more qubits simultaneously
}
```

**3. Use `Qubits<N>` for register operations:**
```gala
// Efficient: single loop over register
for i in 0..N { h(qs[i]) }

// Less efficient: individual qubits
let q0 = qubit(); let q1 = qubit(); ...
```

## Gradient Cost Optimization

### Parameter-Shift Batching (Automatic)
```gala
// Before optimization: 2N submissions for N params
// After: 2 submissions (all +π/2, all -π/2)

let dθ = grad(f, wrt: θ)  // Runtime handles batching
```

### Check Gradient Cost
```bash
gala run src/vqa.gala --bench-grad
```

Output:
```
Gradient benchmark:
  Parameters: 24
  Submissions (unbatched): 48
  Submissions (batched): 2
  Speedup: 24x
  Wall time: 2.3s (vs 55s unbatched)
```

### Reduce Gradient Cost
1. **Fewer parameters** — smaller ansatz, parameter sharing
2. **SPSA optimizer** — 2 evals/step vs 2N for gradient descent
3. **Analytic gradients where possible** — some gates have known gradients
4. **Shot allocation** — more shots for sensitive params, fewer for others

```gala
// SPSA: only 2 circuit evaluations per step
let opt = spsa_optimizer(lr: 0.01, c: 0.1)
θ = opt.step(θ, loss_fn)  // No explicit grad needed!
```

## Compilation Performance

### Incremental Compilation
```bash
# First build (cold)
gala build src/main.gala

# Edit one file, rebuild (incremental — uses salsa)
gala build src/main.gala  # Much faster
```

### Frontend-Only Check (No LLVM/QuEST)
```bash
# Fast type-check without backend compilation
cargo build --workspace --features frontend-only
gala check src/main.gala
```

### Profile Compilation
```bash
# Time each pass
RUST_LOG=gala_driver=info gala build src/main.gala

# Salsa query statistics
RUST_LOG=salsa=debug gala build src/main.gala
```

## Simulation Performance

### State-Vector (QuEST)
```bash
# QuEST uses multi-threading automatically
export OMP_NUM_THREADS=8
gala run src/large.gala --shots 1000

# GPU acceleration (if QuEST built with CUDA)
gala run src/large.gala --backend simulator:gpu
```

### Shot Batching
```bash
# Single shot: full state vector each time
# Multi-shot: batched in QuEST
gala run src/main.gala --shots 10000  # Batched automatically
```

### Memory Estimation
```
Qubits | State Vector Size
-------|------------------
20     | 16 MB
25     | 512 MB
30     | 16 GB
35     | 512 GB
```

**Tip:** Use tensor-network simulator (`gala-sim` tensor-network feature) for >30 qubits with low entanglement.

## Benchmarking

### Standard Benchmark Suite
```bash
# Gate count / depth on standard algorithms
gala bench --suite standard

# Custom benchmark
gala bench src/my_algo.gala --metric gates,depth,width
```

### Track Over Time
```bash
# Save baseline
gala bench --save-baseline

# Compare current to baseline
gala bench --compare-baseline
```

## Profiling Tools

```bash
# Compiler passes timing
RUST_LOG=gala_driver=info,gala_opt=info gala build src/main.gala

# Simulator profiling
gala run src/main.gala --profile

# Gradient profiling
gala run src/vqa.gala --bench-grad --verbose
```

## Common Anti-Patterns

| Anti-Pattern | Problem | Fix |
|--------------|---------|-----|
| `measure` in `quantum` fn | Effect error E0301 | Make function `prob` |
| Large ancilla lifetime | More qubits, slower sim | Narrow scope |
| Unparameterized loops | Missed fusion opportunities | Use const generics |
| No type annotations on public API | Slower incremental | Add type signatures |
| Classical work in `quantum` fn | Prevents optimization | Move to `pure` helper |

## CI Performance Gates

```yaml
# .github/workflows/perf.yml
- name: Circuit quality regression
  run: |
    gala bench --compare-baseline --fail-on-regression 10%
```

```bash
# Local: check before push
gala bench --compare-baseline
```