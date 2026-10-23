---
name: gala-python-interop
description: Call Gala from Python — PyO3 bridge, Jupyter kernel, PyTorch integration, NumPy arrays
disable-model-invocation: true
allowed-tools: Bash(cargo run --bin gala *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Overview

Gala provides **Python interoperability** via PyO3. Call Gala kernels from Python for incremental adoption in ML pipelines.

## Installation

```bash
# Install Gala Python package (when available)
pip install gala-lang

# Or build from source
cd tools/gala-python
maturin develop --release
```

## Basic Usage

```python
import gala
import numpy as np

# Compile Gala code from string
kernel = gala.compile("""
fn bell() -> Qubits<2> quantum {
    let a = qubit()
    let b = qubit()
    h(a)
    cx(a, b)
    (a, b)
}
""")

# Run on simulator
result = kernel.run(backend="simulator", shots=1000)
print(result)  # Measured distribution
```

## PyTorch Integration

```python
import torch
import gala

# Gala variational layer as PyTorch module
class GalaVQLayer(torch.nn.Module):
    def __init__(self, n_qubits: int, depth: int):
        super().__init__()
        self.n_qubits = n_qubits
        self.depth = depth
        
        # Gala parameters as PyTorch parameters
        self.theta = torch.nn.Parameter(
            torch.randn(depth, n_qubits) * 0.1
        )
        
        # Compile Gala circuit
        self.circuit = gala.compile(f"""
        import gala.vqa.{{angle_encode, layered_ansatz}}
        
        fn circuit(x: Vec<Float>, θ: Params) -> Measured<Float> prob {{
            let q = angle_encode(qubits<{n_qubits}>(), x)
            layered_ansatz(q, θ, depth: {depth})
            // Measure expectation of Z on first qubit
            let m = measure(q[0])
            // Convert to expectation: P(0) - P(1)
            Measured<Float>::new(if m {{ -1.0 }} else {{ 1.0 }})
        }}
        """)
    
    def forward(self, x: torch.Tensor) -> torch.Tensor:
        # x: [batch, n_qubits]
        # theta: [depth, n_qubits]
        results = []
        for i in range(x.shape[0]):
            result = self.circuit.run(
                backend="simulator",
                shots=1000,
                x=x[i].detach().numpy(),
                theta=self.theta.detach().numpy()
            )
            results.append(result.expectation())
        return torch.tensor(results, dtype=torch.float32)

# Use in PyTorch model
class HybridModel(torch.nn.Module):
    def __init__(self):
        super().__init__()
        self.classical = torch.nn.Sequential(
            torch.nn.Linear(784, 256),
            torch.nn.ReLU(),
            torch.nn.Linear(256, 4)  # 4 qubits
        )
        self.quantum = GalaVQLayer(4, 3)
        self.output = torch.nn.Linear(1, 10)
    
    def forward(self, x):
        x = self.classical(x)
        x = self.quantum(x)  # Quantum layer
        x = self.output(x)
        return x

# Train end-to-end!
model = HybridModel()
optimizer = torch.optim.Adam(model.parameters(), lr=0.01)

for epoch in range(100):
    for batch_x, batch_y in dataloader:
        optimizer.zero_grad()
        pred = model(batch_x)
        loss = torch.nn.functional.cross_entropy(pred, batch_y)
        loss.backward()
        optimizer.step()
```

## NumPy Arrays

```python
import gala
import numpy as np

# Gala functions accept/return NumPy arrays
kernel = gala.compile("""
fn process(data: Vec<Float>) -> Measured<Float> prob {
    let q = qubits<4>()
    angle_encode(q, data)
    // ... circuit ...
    measure(q[0])
""")

# NumPy in, NumPy out
x = np.array([0.1, 0.2, 0.3, 0.4], dtype=np.float32)
result = kernel.run(x=x, shots=10000)

# result is a gala.Measured object
print(result.expectation())      # float
print(result.distribution())     # dict: {0: 0.48, 1: 0.52}
print(result.samples())          # np.array of 0/1 samples
```

## Gradient from Python

```python
import gala
import numpy as np

# Compile with gradient
kernel = gala.compile("""
fn loss(x: Vec<Float>, θ: Params) -> Measured<Float> prob {
    // ... VQA circuit ...
    let expectation = measure(q[0])  # Returns Measured<Bool>
    Measured<Float>::new(if expectation {{ 1.0 }} else {{ -1.0 }})
}
""", with_grad=True)

# Compute gradient wrt θ
x = np.random.randn(4).astype(np.float32)
θ = np.random.randn(3, 4).astype(np.float32)

loss_val, grad_θ = kernel.grad(x=x, theta=θ, wrt="theta")

print(f"Loss: {loss_val}")
print(f"Gradient shape: {grad_θ.shape}")  # (3, 4)

# Use in custom optimizer
θ -= 0.01 * grad_θ
```

## Jupyter Kernel

```bash
# Install Gala Jupyter kernel
gala kernel install

# In Jupyter notebook:
%%gala
fn bell() -> Qubits<2> quantum {
    let a = qubit()
    let b = qubit()
    h(a)
    cx(a, b)
    (a, b)
}

# Run cell shows circuit diagram + simulation results
```

### Jupyter Features
- **Inline circuit diagrams** (matplotlib/SVG)
- **State vector visualization** (Bloch sphere, histogram)
- **Progressive execution** — run cell-by-cell
- **Variable inspection** — hover for types/effects

## Async Hardware Execution

```python
import gala
import asyncio

kernel = gala.compile("...")

async def run_on_hardware():
    # Submit job (non-blocking)
    job = await kernel.submit_async(
        backend="ionq.aria",
        shots=1000,
        x=data
    )
    
    # Poll for completion
    while not job.done():
        await asyncio.sleep(5)
        status = await job.status()
        print(f"Status: {status}")
    
    # Get results
    result = await job.result()
    return result

# Run multiple in parallel
async def main():
    jobs = [run_on_hardware() for _ in range(10)]
    results = await asyncio.gather(*jobs)

asyncio.run(main())
```

## Callbacks & Streaming

```python
import gala

def progress_callback(job_id, progress, partial_result):
    print(f"Job {job_id}: {progress*100:.1f}% complete")

kernel = gala.compile("...")

# Stream results as they arrive
for partial in kernel.stream(
    backend="simulator",
    shots=10000,
    batch_size=1000,
    callback=progress_callback
):
    print(f"Partial expectation: {partial.expectation()}")
```

## Type Mapping

| Gala Type | Python Type |
|-----------|-------------|
| `Bool` | `bool` / `np.bool_` |
| `Int` | `int` / `np.int64` |
| `Float` | `float` / `np.float32` / `np.float64` |
| `Complex` | `complex` / `np.complex64` |
| `Vec<Float>` | `np.ndarray` (1D, float32) |
| `Params` | `np.ndarray` (2D, float32) |
| `Measured<Bool>` | `gala.MeasuredBool` |
| `Measured<Float>` | `gala.MeasuredFloat` |
| `Qubits<N>` | `gala.QubitRegister` (opaque) |

## Measured Object API

```python
result = kernel.run(shots=1000)

# Expectation value
print(result.expectation())  # float

# Full probability distribution
print(result.distribution())  # {0: 0.49, 1: 0.51}

# Raw samples (for custom statistics)
samples = result.samples()  # np.array([0, 1, 1, 0, ...])

# Statistical methods
print(result.std())         # Standard deviation
print(result.confidence_interval(0.95))  # (low, high)
```

## Configuration

```python
import gala

# Global config
gala.config.set("default_backend", "simulator")
gala.config.set("default_shots", 1000)
gala.config.set("default_seed", 42)

# Per-kernel config
kernel = gala.compile(code, config={
    "backend": "simulator",
    "shots": 10000,
    "seed": 123,
    "noise": {"depolarizing": 0.01}
})
```

## Error Handling

```python
import gala
from gala import GalaError, CompileError, RuntimeError

try:
    kernel = gala.compile(invalid_code)
except CompileError as e:
    print(f"Compile error: {e.code}")  # E0412, etc.
    print(f"Message: {e.message}")
    print(f"Help: {e.help}")
    print(f"Span: {e.span}")  # (file, line, col)

try:
    result = kernel.run(backend="ionq.aria", shots=1000)
except RuntimeError as e:
    print(f"Runtime error: {e}")
    # Job ID available for retry: e.job_id
```

## Performance Tips

1. **Reuse compiled kernels** — compile once, run many times
2. **Batch shots** — single run with `shots=10000` faster than 10×`shots=1000`
3. **Use gradient batching** — `kernel.grad()` uses runtime batching
4. **Choose right backend** — simulator for dev, hardware for final
5. **Profile with `gala.run --bench-grad`** from CLI

## Roadmap

- **Full PyTorch autograd integration** — `torch.autograd.Function` for Gala
- **JAX integration** — `jax.pure_callback` for Gala kernels
- **TensorFlow integration** — `tf.py_function` wrapper
- **Distributed execution** — multi-device, multi-shot parallelism