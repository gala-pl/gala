# 07 — Standard Library

The standard library is layered so beginners meet high-level constructs first and drop down only
when needed. It is written in Gala where possible, with a small set of compiler **intrinsics** for
primitives that cannot be expressed in-language (qubit allocation, gate application, measurement).

## Layering

```
gala.vqa        (encoders, ansätze, optimizers, grad utilities)   ← highest level
gala.algorithms(QFT, phase estimation, Grover, Trotter)
gala.gates     (standard gate set + derived adjoint/controlled)
gala.core      (qubit, measure, registers, effects)              ← intrinsics live here
gala.classical (collections, numerics, IO)                       ← ordinary classical stdlib
gala.noise     (noise models as typed values)
gala.hardware  (backend capabilities, topology descriptors)
```

## `gala.core`
The primitive layer. Qubit allocation (`qubit`, `qubits<N>`), `measure`, `drop`, register
indexing/slicing, and the effect primitives. Mostly compiler intrinsics with thin Gala wrappers.

## `gala.gates`
The standard universal gate set: Pauli `x/y/z`, `h`, `s`, `t`, phase, rotations `rx/ry/rz`,
two-qubit `cx/cz/swap`, and parametrized multi-qubit gates. Because gates are reversible, their
**adjoint and controlled forms are compiler-derived** — the library exposes them without
hand-written inverses.

## `gala.algorithms`
Batteries-included, well-tested building blocks:
- Quantum Fourier Transform (`qft`, `iqft`)
- Quantum phase estimation
- Amplitude amplification / Grover search
- Trotter–Suzuki Hamiltonian evolution
- Quantum arithmetic (adders, comparators) using auto-uncomputed ancillae

These double as the flagship correctness tests for uncomputation and reversibility.

## `gala.vqa` (the AI layer — the beachhead)
The reason Gala exists for its primary persona:
- **Encoders:** `angle_encode`, `amplitude_encode`, `basis_encode`.
- **Ansatz zoo:** `layered_ansatz`, hardware-efficient ansatz, problem-specific templates.
- **Templates:** `qaoa`, `vqe`, quantum kernels.
- **Optimizers:** gradient descent, Adam, SPSA — plain classical `pure` code operating on
  `Params`.
- **`grad` utilities:** helpers around the native `grad` operator; batching hints; parameter
  initialization strategies.
- **Interop:** conversion helpers for classical ML frameworks (via the future Python bridge).

## `gala.noise`
Noise models as first-class typed values (depolarizing, amplitude/phase damping, readout error,
custom Kraus channels) for error-aware development and testing before hardware.

## `gala.hardware`
Backend capability descriptors and topology (connectivity graphs, native gate sets) — used by the
target-lowering pass and by capability checking.

## `gala.classical`
Because Gala is a complete language, not a DSL: collections (`Vec`, `Map`, `Set`), numerics
(`Float`, `Complex`, linear algebra basics), iterators, strings, IO, error handling. Keeps hybrid
programs entirely in-language.

## Design rules for the stdlib
- **Prefer Gala source over intrinsics** so the stdlib doubles as documentation and dogfoods the
  language.
- **Every public item has doc comments** (`///`) and at least one property test.
- **No item ships without conformance coverage** ([TESTING_QA.md](./TESTING_QA.md)).
- **Stability tiers:** `stable`, `unstable(feature)`, `experimental` — the ML/hardware layers move
  fastest and are explicitly marked.
