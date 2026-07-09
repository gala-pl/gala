# Gala

**A modern, modular programming language for building AI-driven quantum systems.**
*Classical control. Quantum kernels. Differentiable everywhere.*

Gala is a standalone, statically-typed language whose compiler enforces the laws of quantum
mechanics — no-cloning, reversibility, and safe uncomputation — through modern type theory,
while making hybrid quantum-classical machine learning ergonomic end to end. The compiler and
toolchain are implemented in **Rust**, and Gala programs lower to **QIR** (the LLVM-based
Quantum Intermediate Representation) to run on simulators and real hardware across vendors.

> **Implementation track: Option B — standalone language.** Gala has its own `.gala` source
> files, its own parser, type checker, IR, optimizer, and `gala` CLI. Rust is the language the
> compiler is *written in*; it is not a Rust eDSL. This gives full control over syntax,
> semantics, and error-message quality.

---

## Documentation suite

Read in order for a full picture; each doc is self-contained enough to reference on its own.

| # | Document | What it covers |
|---|----------|----------------|
| — | [README.md](./README.md) | This index + project overview |
| 00 | [VISION.md](./VISION.md) | Thesis, why now, goals & non-goals, principles |
| 01 | [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) | Surface syntax, grammar (EBNF), semantics, examples |
| 02 | [TYPE_SYSTEM.md](./TYPE_SYSTEM.md) | Linear types, effect system, uncomputation, reversibility, differentiation |
| 03 | [ARCHITECTURE.md](./ARCHITECTURE.md) | Cargo workspace layout, crate boundaries, the compiler as a whole |
| 04 | [COMPILER_PIPELINE.md](./COMPILER_PIPELINE.md) | Lexer → parser → HIR → GIR → optimizer → backends; salsa query graph |
| 05 | [BACKENDS_RUNTIME.md](./BACKENDS_RUNTIME.md) | QIR emission, simulators (roqoqo/QuEST), hardware, the hybrid runtime |
| 06 | [TOOLCHAIN_DX.md](./TOOLCHAIN_DX.md) | `gala` CLI, LSP, REPL, formatter, package manager, diagnostics |
| 07 | [STANDARD_LIBRARY.md](./STANDARD_LIBRARY.md) | Standard library layering: core, gates, algorithms, ml, noise, hardware |
| 08 | [TESTING_QA.md](./TESTING_QA.md) | Test strategy, quantum-aware property testing, CI, conformance suite |
| 09 | [ROADMAP.md](./ROADMAP.md) | Phased milestones from bootstrap to fault-tolerant readiness |
| 10 | [AGENTIC_BUILD_PLAN.md](./AGENTIC_BUILD_PLAN.md) | **The agentic build plan** — epics, work packages, agent prompts, acceptance criteria, dependency graph |
| 11 | [CONTRIBUTING.md](./CONTRIBUTING.md) | Dev workflow, conventions, definition of done, agent operating rules |
| 12 | [GLOSSARY.md](./GLOSSARY.md) | Quantum + PL + compiler terminology |

---

## The one-paragraph pitch

Quantum software today is split between physics libraries bolted onto Python (Qiskit, Cirq,
PennyLane) that offer no language-level correctness guarantees, and academic languages (Silq,
Q#, Quipper) that have the semantics but not the tooling or the hybrid-AI focus. Gala occupies
the gap: the semantic rigor of the research languages (linear types for no-cloning, safe
automatic uncomputation, reversibility), the differentiable-programming ergonomics that quantum
ML needs (`grad` as a native operator), and production-grade DX (a real LSP, teaching-grade
errors, a built-in simulator, cross-vendor QIR lowering) — in one coherent, Rust-built toolchain.

## Why Rust is load-bearing (not incidental)

The single hardest guarantee in the design — the no-cloning theorem — is almost exactly Rust's
ownership model: a value that is moved on use, is neither `Copy` nor `Clone`, and must be
consumed. Implementing the compiler in Rust means the host language's own discipline mirrors the
object language's semantics, the ecosystem (`chumsky`, `logos`, `ariadne`, `salsa`, `inkwell`,
`tower-lsp`, `roqoqo`) covers most of the non-novel machinery, and the result is a fast,
memory-safe, single-binary toolchain.

## Status

Concept / pre-implementation. This suite is the design-of-record and the executable plan. Start
with [VISION.md](./VISION.md), then jump to
[AGENTIC_BUILD_PLAN.md](./AGENTIC_BUILD_PLAN.md) to begin building.

## License & governance (intended)

Open-source core (Apache-2.0 or MIT/Apache dual), public RFC process, transparent roadmap.
See [CONTRIBUTING.md](./CONTRIBUTING.md).