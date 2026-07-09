# 00 — Vision, Goals & Principles

## Thesis

> The next generation of quantum software will be written by AI and software engineers, not
> physicists — and it will be hybrid by default. Gala is the language for that world.

Quantum programs are never purely quantum. They are classical programs that orchestrate quantum
kernels, read measurements, and feed parameters back in a loop. Gala is therefore designed as a
**classical language whose compiler enforces the laws of quantum mechanics** — no-cloning,
reversibility, and safe uncomputation — while making hybrid quantum-classical machine learning
ergonomic end to end.

## Why now

- **Hardware crossed the fault-tolerance threshold.** Below-threshold logical qubits have been
  demonstrated (logical error rate falling as the surface code scales), and credible roadmaps
  now target hundreds of logical qubits from ~10,000 physical qubits by the end of the decade,
  with mid-circuit measurement and measurement-conditioned control flow already shipping. Programs
  are becoming *real programs* (branching, loops, feedback), not flat circuits — which is exactly
  when a language beats a circuit-builder library.
- **The market is ML-led.** Independent forecasts put quantum computing on a 22–42% CAGR through
  2030 and repeatedly name machine learning as the largest application segment. The value follows
  hybrid quantum-AI workloads — the hardest thing to write well with today's tools.
- **The tooling gap is structural.** Python SDKs cannot express compile-time no-cloning, safe
  uncomputation, or reversibility, because Python's type system can't. The research languages can,
  but lack tooling, ecosystem, and hybrid-first design. No one has combined both.

## Goals (in priority order)

1. **Correctness you can't opt out of.** Physically meaningless programs are unrepresentable.
2. **Explicit classical/quantum boundary.** Measurement — where quantum becomes classical and
   determinism becomes probability — is visible in the types and never silent.
3. **Differentiability as a language feature.** `grad` is a native operator; the compiler
   synthesizes parameter-shift for quantum parameters and autodiff for classical ones and composes
   them.
4. **Great DX from day one.** LSP, teaching-grade errors, built-in simulator, canonical
   formatter, package manager, and test runner ship *with* the language.
5. **Modular & backend-polymorphic.** The same source runs on a state-vector simulator, a noisy
   simulator, or real hardware by swapping a capability, not rewriting code.
6. **Progressive disclosure.** Five-line Bell pair for beginners; explicit uncomputation control,
   dependent qubit counts, and custom effect handlers for experts.
7. **Eyes on the future, feet on the present.** Designed for logical-qubit programs with rich
   control flow, but useful today on simulators and NISQ devices.

## Non-goals

- Not a physics simulator or a replacement for classical linear-algebra libraries.
- Not a language that *assumes* fault-tolerant hardware exists today.
- Not a Rust eDSL (that was Option A; we chose the standalone track — see README).
- Not trying to be everything on day one: the beachhead is **hybrid quantum machine learning**.

## Design principles

When principles conflict, higher ones win.

1. **If the hardware forbids it, the type system forbids it.**
2. **Make the common case beautiful and the advanced case possible.**
3. **Errors teach.** A diagnostic should explain the physics or the rule it enforces, and suggest
   the fix.
4. **One obvious way.** Canonical formatter, canonical project layout, minimal ceremony.
5. **Inspectable at every layer.** A stable, typed IR you can dump and reason about — "LLVM for
   quantum," not a black box.
6. **Reuse proven machinery.** Don't reinvent lexers, LLVM, or simulators; build the novel core
   (type system + differentiation) and assemble the rest from mature Rust crates.

## Target personas (summary)

- **Maya**, the hybrid ML engineer — primary persona; wants `grad` to work across the boundary.
- **Dr. Chen**, the algorithm researcher — wants Silq-like semantics with real tooling.
- **Sam**, the platform engineer — wants vendor-neutral deploy, reproducible builds, CI.
- **Prof. Ada & Leo**, educator and student — want a language that teaches.

## What success looks like

A hybrid quantum-ML model — data encoding, parameterized ansatz, measurement, gradient, and
classical optimizer — expressed as one type-checked Gala program, developed on the built-in
simulator with zero setup, and deployed to real hardware by swapping one backend capability, with
a class of quantum bugs made impossible by the compiler.
