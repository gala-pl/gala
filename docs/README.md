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
| 00 | [docs/00-vision.md](./docs/00-vision.md) | Thesis, why now, goals & non-goals, principles |
| 01 | [docs/01-language-spec.md](./docs/01-language-spec.md) | Surface syntax, grammar (EBNF), semantics, examples |
| 02 | [docs/02-type-system.md](./docs/02-type-system.md) | Linear types, effect system, uncomputation, reversibility, differentiation |
| 03 | [docs/03-architecture.md](./docs/03-architecture.md) | Cargo workspace layout, crate boundaries, the compiler as a whole |
| 04 | [docs/04-compiler-pipeline.md](./docs/04-compiler-pipeline.md) | Lexer → parser → HIR → GIR → optimizer → backends; salsa query graph |
| 05 | [docs/05-backends-runtime.md](./docs/05-backends-runtime.md) | QIR emission, simulators (roqoqo/QuEST), hardware, the hybrid runtime |
| 06 | [docs/06-toolchain-dx.md](./docs/06-toolchain-dx.md) | `gala` CLI, LSP, REPL, formatter, package manager, diagnostics |
| 07 | [docs/07-stdlib.md](./docs/07-stdlib.md) | Standard library layering: core, gates, algorithms, ml, noise, hardware |
| 08 | [docs/08-testing-qa.md](./docs/08-testing-qa.md) | Test strategy, quantum-aware property testing, CI, conformance suite |
| 09 | [docs/09-roadmap.md](./docs/09-roadmap.md) | Phased milestones from bootstrap to fault-tolerant readiness |
| 10 | [docs/10-agentic-build-plan.md](./docs/10-agentic-build-plan.md) | **The agentic build plan** — epics, work packages, agent prompts, acceptance criteria, dependency graph |
| 11 | [docs/11-contributing.md](./docs/11-contributing.md) | Dev workflow, conventions, definition of done, agent operating rules |
| 12 | [docs/12-glossary.md](./docs/12-glossary.md) | Quantum + PL + compiler terminology |

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
with [docs/00-vision.md](./docs/00-vision.md), then jump to
[docs/10-agentic-build-plan.md](./docs/10-agentic-build-plan.md) to begin building.

## License & governance (intended)

Open-source core (Apache-2.0 or MIT/Apache dual), public RFC process, transparent roadmap.
See [docs/11-contributing.md](./docs/11-contributing.md).