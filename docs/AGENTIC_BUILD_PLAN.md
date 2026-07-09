# 10 — Agentic Build Plan

This is the executable plan: how to build Gala with a team of AI coding agents (e.g. Claude Code
agents) plus human review. It defines agent **roles**, the **operating loop**, a **work-package
(WP) backlog** with dependencies and acceptance criteria, and **ready-to-use agent prompts**.

> **How to read this:** each WP is a self-contained unit an agent can pick up. It lists its goal,
> the crate(s) it touches, dependencies (other WPs that must finish first), deliverables, and a
> concrete **Definition of Done (DoD)**. Ship WPs in dependency order; parallelize siblings.

---

## 1. Agent roles

| Role | Responsibility | Tools/scope |
|------|----------------|-------------|
| **Architect** (human-led, agent-assisted) | Owns specs, resolves RFCs, approves IR/type changes | This doc suite; decision log |
| **Builder** | Implements a WP end to end (code + tests + docs) | One crate cluster per WP |
| **Reviewer** | Independent verification of a WP against its DoD | Read + test; cannot self-approve own WP |
| **Test/QA** | Owns conformance suite, property tests, CI health | `tests/`, CI config |
| **Docs** | Keeps this suite and `///` docs in sync with reality | `docs/`, doc comments |
| **Integrator** | Merges, keeps `main` green, manages releases | CI, branch protection |

Rule: **the Builder of a WP never reviews their own WP.** Every WP passes an independent Reviewer
and green CI before merge.

## 2. The operating loop (per WP)

```
1. CLAIM      Builder claims a WP (marks in-progress), reads linked spec docs.
2. PLAN       Builder writes a short plan comment: approach, files, test list. Architect ack for
              WPs marked [spec-sensitive].
3. BUILD      Implement on a feature branch. Small commits. Follow 11-contributing.md.
4. SELF-CHECK Run fmt, clippy -D warnings, tests, relevant conformance cases locally.
5. PR         Open PR referencing the WP id; fill the PR checklist (DoD).
6. REVIEW     Reviewer verifies against DoD; runs the branch; checks tests actually test the thing.
7. MERGE      Integrator merges when CI green + Reviewer approves. Docs updated in the same PR.
8. LOG        Any deviation from spec is recorded in the decision log; specs updated if needed.
```

**Guardrails for agents (also in [CONTRIBUTING.md](./CONTRIBUTING.md)):**
- Never weaken a test to make CI pass; fix the code or escalate.
- Never mark a WP done with failing tests, partial implementation, or `todo!()` in shipped paths.
- If blocked or the spec is ambiguous, stop and open a question rather than guessing.
- Prefer reusing the designated crate over hand-rolling (parser → `chumsky`, diagnostics →
  `ariadne`, LLVM → `inkwell`, sim → `roqoqo`, LSP → `tower-lsp`).
- Keep the frontend backend-agnostic: nothing above `gala-gir` may import a backend crate.

## 3. Milestone → WP map

- **M0 Bootstrap:** WP-000..003
- **M1 Frontend checks (no runtime):** WP-010..017
- **M2 It runs on the simulator:** WP-020..027
- **M3 Hybrid & differentiable:** WP-030..036
- **M4 Hardware & scale:** WP-040..047
- **M5 Ecosystem & FT readiness:** WP-050..056

Dependency graph (high level):

```
WP-000 ─▶ WP-001 ─▶ WP-010 ─▶ WP-011 ─▶ WP-012 ─▶ WP-013 ─▶ WP-014 ─▶ WP-016(GIR)
   │         │                                   │            │            │
   └▶WP-002  └▶WP-003(CI)                        └▶WP-015(uncompute)       ├▶ WP-020(sim)
                                                                           ├▶ WP-030(diff)
                                                                           └▶ WP-040(QIR)
```

---

## 4. Work-package backlog

### M0 — Bootstrap

**WP-000 — Workspace skeleton**
- Goal: Cargo workspace with all crate stubs from [ARCHITECTURE.md](./ARCHITECTURE.md); each
  crate compiles empty.
- Deps: none.
- Deliverables: workspace `Cargo.toml`, crate dirs, `rust-toolchain.toml`, `Cargo.lock`.
- DoD: `cargo build` succeeds; `gala --version` prints; README links resolve.

**WP-001 — Span, source DB, interning**
- Goal: `gala-span` — `FileId`, byte spans, source map, string interner (`lasso`).
- Deps: WP-000.
- DoD: unit tests for span arithmetic and interning; used by later crates.

**WP-002 — Diagnostics framework**
- Goal: `gala-diagnostics` — error-code registry, `Diagnostic` type, `ariadne` rendering, the
  `gala explain <code>` data model.
- Deps: WP-001.
- DoD: renders a sample multi-label diagnostic; `explain` returns long-form for a seeded code;
  snapshot tests via `insta`.

**WP-003 — CI + repo hygiene**
- Goal: CI pipeline per [TESTING_QA.md](./TESTING_QA.md) §4; `fmt`/`clippy` gates; PR
  template with the DoD checklist; branch protection.
- Deps: WP-000.
- DoD: CI green on an empty pipeline; a deliberately failing test blocks merge.

### M1 — Frontend that type-checks

**WP-010 — Lexer**
- Goal: `gala-lexer` with `logos`; all tokens in [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) §2,
  incl. nested block comments, complex literals, Unicode idents.
- Deps: WP-001.
- DoD: property test (roundtrip of token spans); conformance lexer cases.

**WP-011 — Parser + AST**
- Goal: `gala-parser` (`chumsky`) → `gala-ast`; grammar per spec §3; **error recovery**.
- Deps: WP-010, WP-002.
- DoD: parses all `tests/fixtures/*.gala`; a syntax error yields partial AST + `E01xx` diagnostic
  (not a panic); snapshot tests.

**WP-012 — HIR, desugar, name resolution, modules**
- Goal: `gala-hir` on `salsa`; desugar (spec §7.4), module graph, `DefId`s, resolution.
- Deps: WP-011.
- DoD: resolves multi-file fixtures; unresolved-name diagnostics; salsa incremental recompute test.

**WP-013 — Type & effect inference** `[spec-sensitive]`
- Goal: `gala-types` — bidirectional inference, HM classical core, effect lattice
  `pure ⊑ quantum ⊑ prob`, const-generic `Qubits<N>` solving (`ena`).
- Deps: WP-012.
- DoD: conformance `types/` and `effects/` pass; boundary-cross (`prob` in `quantum`) is `E03xx`.

**WP-014 — Linearity checker (no-cloning)** `[spec-sensitive]`
- Goal: ownership/liveness pass enforcing linear discipline
  ([TYPE_SYSTEM.md](./TYPE_SYSTEM.md) §1) with the flagship teaching diagnostics.
- Deps: WP-013.
- DoD: conformance `linearity/` passes: use-after-consume, duplication, implicit-drop, aliasing
  each produce the specified `E04xx` with the documented `help:` text.

**WP-015 — Uncomputation analysis + synthesis** `[spec-sensitive]`
- Goal: `gala-uncompute` — provenance tracking, liftability, `adjoint(provenance)` synthesis on
  `drop`/scope-exit; `E0530` refusal with reason.
- Deps: WP-014.
- DoD: conformance `uncompute/` passes (both success and refusal); synthesized plans validated
  once the simulator exists (WP-020) via the `uncomputes` property.

**WP-016 — GIR + lowering from typed HIR**
- Goal: `gala-gir` data model (typed, effect-annotated op-graph; `petgraph`) + lowering; dumpable
  via `gala build --emit gir`.
- Deps: WP-014 (and WP-015 for uncompute insertion).
- DoD: golden GIR snapshots for Bell/QFT; invariant checks (linearity preserved) in debug builds.

**WP-017 — `gala check` CLI + `explain`**
- Goal: wire `gala-cli` + `gala-driver` so `gala check` runs the whole frontend and prints
  diagnostics; `gala explain <code>` works.
- Deps: WP-013..016.
- DoD: end-to-end check on fixtures; M1 acceptance gate met.

### M2 — It runs on the simulator

**WP-020 — Simulator backend**
- Goal: `gala-sim` bridging `roqoqo` + `roqoqo-quest` (QuEST) state-vector; execute GIR.
- Deps: WP-016.
- DoD: Bell/QFT/Grover produce correct distributions; deterministic under fixed seed.

**WP-021 — `gala run`**
- Goal: `gala-cli run` → lower → simulate → print `Measured<T>` results; zero config.
- Deps: WP-020.
- DoD: `gala new demo && gala run` simulates a Bell pair out of the box (M2 gate).

**WP-022 — Basic optimizer**
- Goal: `gala-opt` — gate fusion, adjacent cancellation, rotation merge; invariant-preserving.
- Deps: WP-016.
- DoD: gate-count benchmark improves on the standard set; property tests confirm semantic
  equivalence on the simulator.

**WP-023 — `gala.core` + `gala.gates`**
- Goal: intrinsics + Gala-source standard gates with compiler-derived adjoint/controlled.
- Deps: WP-016.
- DoD: gate library property-tested (unitarity); adjoint/controlled derivation correct.

**WP-024 — `gala.algorithms` (first set)**
- Goal: QFT/iQFT, Grover, phase estimation, a Trotter step, quantum adder (exercises uncompute).
- Deps: WP-023, WP-020.
- DoD: `reversible`/`uncomputes` properties green; simulated outputs match references.

**WP-025 — Formatter**
- Goal: `gala-fmt` on the lossless CST; one canonical style; `--check`.
- Deps: WP-011.
- DoD: idempotent (`fmt(fmt(x)) == fmt(x)`); stdlib formatted; CI `fmt --check` gate on.

**WP-026 — REPL with visualization**
- Goal: `gala repl` — incremental eval + state-vector/Bloch/histogram rendering (terminal + optional
  web view).
- Deps: WP-020.
- DoD: interactive session demo; visualization updates per statement.

**WP-027 — Quantum property-testing harness**
- Goal: `#[property(...)]` attributes (`unitary`, `reversible`, `uncomputes`, `grad_matches`,
  effect-honesty) run by `gala test`.
- Deps: WP-020 (grad_matches deferred to WP-030).
- DoD: properties run over randomized inputs; failures shrink to minimal cases.

### M3 — Hybrid & differentiable (the beachhead)

**WP-030 — Native differentiation** `[spec-sensitive]`
- Goal: `gala-diff` — parameter-shift for quantum params, reverse-mode autodiff for classical,
  chain-rule composition; lowers to GIR.
- Deps: WP-016, WP-020.
- DoD: `grad_matches` vs finite-difference within tolerance on a suite of circuits; conformance
  `diff/` passes.

**WP-031 — `gala.ml`**
- Goal: encoders, ansatz zoo, QAOA/VQE templates, optimizers (GD/Adam/SPSA), grad utilities.
- Deps: WP-030.
- DoD: a variational classifier trains to target accuracy on the simulator (M3 gate).

**WP-032 — Noisy simulator + `gala.noise`**
- Goal: noise models as typed values; QuEST noisy execution.
- Deps: WP-020.
- DoD: noise-model property tests; a noisy vs noiseless demo.

**WP-033 — Gradient batching in runtime**
- Goal: `gala-runtime` batches parameter-shift evaluations into minimal submissions.
- Deps: WP-030.
- DoD: submissions-per-grad benchmark meets target; results unchanged vs unbatched.

**WP-034 — LSP v1**
- Goal: `gala-lsp` (`tower-lsp`) sharing the salsa DB: diagnostics, hover (types+effects),
  go-to-def, rename, completion, **inline circuit diagrams**.
- Deps: WP-013..016.
- DoD: works in VS Code (WP-035) on broken files (error recovery); latency budget met.

**WP-035 — VS Code extension**
- Goal: syntax grammar (tree-sitter/TextMate) + LSP client.
- Deps: WP-034.
- DoD: install-and-edit demo; highlighting + diagnostics + circuit lens.

**WP-036 — `gala doc`**
- Goal: API docs from `///`.
- Deps: WP-012.
- DoD: stdlib docs generate and publish.

### M4 — Hardware & scale

**WP-040 — QIR emission** `[spec-sensitive]`
- Goal: `gala-qir` — GIR → QIR (LLVM IR) via `inkwell`; base + adaptive profile selection.
- Deps: WP-016.
- DoD: golden QIR for reference programs; profile chosen correctly (mid-circuit ⇒ adaptive);
  QIR validated by re-simulation.

**WP-041 — Vendor backends via `qoqo_qir`**
- Goal: submit QIR to IBM/IonQ/Quantinuum/Braket; capability descriptors.
- Deps: WP-040.
- DoD: hardware smoke test on ≥1 vendor; capability mismatch is a compile error (`E06xx`).

**WP-042 — Backend-polymorphism + capability checking**
- Goal: backend-as-capability model ([BACKENDS_RUNTIME.md](./BACKENDS_RUNTIME.md) §1);
  `run(f, on: backend)`.
- Deps: WP-041.
- DoD: same program simulates and runs on hardware with only the backend line changed (M4 gate).

**WP-043 — Target lowering (routing/native gates/scheduling)**
- Goal: logical→physical pass: SWAP routing for connectivity, native-gate decomposition,
  scheduling; dumpable.
- Deps: WP-040.
- DoD: routed circuits valid on device topologies; depth/SWAP benchmarks tracked.

**WP-044 — Classical codegen release path**
- Goal: `gala-codegen-classical` — LLVM release (via `inkwell`), cranelift debug.
- Deps: WP-016.
- DoD: orchestration binary runs standalone; perf benchmark vs interpreted baseline.

**WP-045 — Tensor-network simulator**
- Goal: pluggable larger-scale simulator for low-entanglement circuits.
- Deps: WP-020.
- DoD: scales beyond state-vector on suitable circuits; agrees where both apply.

**WP-046 — Package manager + registry v1**
- Goal: `gala-pkg` — `gala.toml`, resolver, `gala.lock`, registry client; `gala add`.
- Deps: WP-000.
- DoD: publish/consume a sample package; reproducible build from lockfile.

**WP-047 — Optimizer v2 (hardware-aware)**
- Goal: cost models per backend; noise-aware gate selection.
- Deps: WP-022, WP-043.
- DoD: fidelity/depth improvement on hardware smoke tests.

### M5 — Ecosystem & FT readiness

**WP-050 — Mid-circuit / adaptive control-flow patterns**
- Goal: language + runtime patterns for logical-qubit programs (measure-and-branch, repeat-until).
- Deps: WP-040, WP-042.
- DoD: adaptive-profile programs run; conformance cases added.

**WP-051 — Python bridge**
- Goal: call Gala kernels from Python (PyO3) for incremental adoption.
- Deps: WP-021, WP-030.
- DoD: a PyTorch loop calls a Gala variational kernel; docs + example.

**WP-052 — Notebook kernel**
- Goal: Jupyter-style kernel using the REPL engine.
- Deps: WP-026.
- DoD: notebook demo with inline visualization.

**WP-053 — Expanded stdlib (ml/algorithms)**
- Goal: more encoders, ansätze, algorithms; stability tiering.
- Deps: WP-031, WP-024.
- DoD: coverage + property tests; changelog.

**WP-054 — Formal core writeup**
- Goal: operational semantics + soundness (progress/preservation, linearity, uncomputation) for
  the core subset.
- Deps: WP-013..016.
- DoD: document + mechanized checks where feasible; cross-linked from
  [TYPE_SYSTEM.md](./TYPE_SYSTEM.md) §7.

**WP-055 — Governance + RFC process at scale**
- Goal: public RFC repo/flow, decision log, contributor ladder.
- Deps: WP-003.
- DoD: an external RFC lands end to end.

**WP-056 — Education program**
- Goal: tutorial, course modules, "learn Gala" path centered on the teaching diagnostics + REPL.
- Deps: M3 complete.
- DoD: a course teaches Gala end to end (M5 gate).

---

## 5. Parallelization guidance

- **Critical path:** WP-000 → 001 → 010 → 011 → 012 → 013 → 014 → 016. Staff this first and
  deepest; most other work unblocks from GIR (WP-016).
- **Fan out after WP-016** into three tracks that share GIR but not each other: **sim** (020…),
  **diff** (030…), **QIR** (040…). Different Builders, coordinated only through the GIR contract.
- **Always-parallel tracks:** Test/QA (conformance grows with every WP) and Docs (this suite stays
  current) run continuously alongside feature work.
- **`[spec-sensitive]` WPs** (013, 014, 015, 030, 040) require Architect sign-off on approach
  before BUILD, because they can change the language's observable behavior.

## 6. Ready-to-use agent prompts

Paste-ready seeds; fill `{WP-id}` and let the agent read the linked docs. The operating loop (§2)
and guardrails apply to all.

**Builder prompt (generic):**
> You are a Builder agent on the Gala compiler (Rust, standalone language — Option B). Implement
> **{WP-id}** from `docs/10-agentic-build-plan.md`. First read that WP entry and every doc it links
> (especially `02-type-system.md` and `04-compiler-pipeline.md` if relevant). Post a short PLAN
> (approach, files, test list) before coding; if the WP is `[spec-sensitive]`, wait for Architect
> ack. Implement on a feature branch with small commits per `docs/11-contributing.md`. Use the
> designated crates (chumsky/logos/ariadne/salsa/inkwell/roqoqo/tower-lsp) rather than hand-rolling.
> Do not touch code outside this WP's crates. Satisfy the WP's Definition of Done exactly: build
> warning-free (`clippy -D warnings`), `gala fmt --check` clean, add or update the conformance and
> property tests named in the WP, document public items, and add `explain` entries for any new
> diagnostic codes. If anything is ambiguous or you must deviate from spec, STOP and open a
> question with your proposed resolution — do not guess, and never weaken a test to go green.

**Reviewer prompt (generic):**
> You are a Reviewer agent. You did not write **{WP-id}**. Check it against its Definition of Done
> in `docs/10-agentic-build-plan.md`. Pull the branch, run `cargo test`, the named conformance
> cases, `clippy -D warnings`, and `gala fmt --check`. Verify the tests actually exercise the
> behavior (not vacuous), diagnostics match the documented codes and `help:` text, and no backend
> crate is imported above `gala-gir`. Confirm docs were updated in the same PR. Approve only if
> every DoD item is met; otherwise leave specific, actionable requested-changes.

**Architect prompt (spec-sensitive gate):**
> You are the Architect. WP **{WP-id}** is `[spec-sensitive]`. Review the Builder's PLAN against
> `docs/01-language-spec.md` and `docs/02-type-system.md`. Confirm the approach preserves the
> guarantees (no-cloning, safe uncomputation, effect-boundary, reversibility) and won't change
> observable language behavior outside what an RFC allows. If it implies a language change, require
> an RFC + conformance cases + a decision-log entry first. Ack or request changes to the PLAN.

**Test/QA prompt:**
> You are the Test/QA agent. For merged WP **{WP-id}**, ensure `tests/conformance/` and the
> property suite cover its behavior, including failure/refusal paths (e.g. `E0530`). Add missing
> cases, keep CI green, and track the compile-time and circuit-quality benchmarks from
> `docs/08-testing-qa.md`.

## 7. Progress tracking

Track WPs on a board (or the repo's issue tracker) with columns Backlog → Claimed → In-progress →
In-review → Done, one issue per WP labeled by milestone. The dependency edges in §3/§5 are the
"blocked-by" links. A WP is **Done** only when merged with green CI and Reviewer approval — see the
Definition of Done in [TESTING_QA.md](./TESTING_QA.md) §6 and
[CONTRIBUTING.md](./CONTRIBUTING.md).
