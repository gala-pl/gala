# 03 — Architecture & Workspace Layout

Gala is a standalone compiler and toolchain implemented as a **Rust Cargo workspace**. This doc
defines the crate boundaries and how they fit together. The pass-by-pass flow is in
[COMPILER_PIPELINE.md](./COMPILER_PIPELINE.md); backends in
[BACKENDS_RUNTIME.md](./BACKENDS_RUNTIME.md).

## 1. Principles for the crate split

- **Thin, well-typed boundaries.** Each crate owns one concern and exposes a small API.
- **Query-based core.** The semantic layer is built on `salsa` for incremental, demand-driven
  computation — the same architecture rust-analyzer uses, so batch compilation and the LSP share
  one engine.
- **The IR is the contract.** Everything above lowering targets **GIR** (Gala IR); everything
  below consumes it. GIR is inspectable and versioned.
- **Novel core, reused periphery.** We write the type system, GIR, and differentiation ourselves;
  we reuse crates for lexing, parsing, diagnostics, LLVM, LSP scaffolding, and simulation.

## 2. Workspace layout

```
gala/
├── Cargo.toml                  # workspace manifest
├── crates/
│   ├── gala-cli/               # `gala` binary: run/build/test/fmt/repl/add
│   ├── gala-driver/            # orchestrates a compilation; wires salsa db
│   ├── gala-lexer/             # logos-based tokenizer
│   ├── gala-parser/            # chumsky parser -> CST/AST; error recovery
│   ├── gala-ast/               # AST types, spans, visitor
│   ├── gala-hir/               # desugared HIR; name resolution; module graph
│   ├── gala-types/             # type & effect checker, inference, linearity
│   ├── gala-uncompute/         # uncomputation analysis & synthesis
│   ├── gala-gir/               # Gala IR: typed, effect-annotated op-graph
│   ├── gala-diff/              # differentiation lowering (param-shift + autodiff)
│   ├── gala-opt/               # GIR optimization passes
│   ├── gala-qir/               # GIR -> QIR (LLVM IR) via inkwell
│   ├── gala-codegen-classical/ # classical lowering (cranelift/LLVM)
│   ├── gala-sim/               # simulator backends (roqoqo/QuEST bridge)
│   ├── gala-runtime/           # hybrid runtime: dispatch, mid-circuit measurement
│   ├── gala-diagnostics/       # ariadne-based diagnostic rendering + error codes
│   ├── gala-span/              # source spans, file db, interner (lasso)
│   ├── gala-lsp/               # tower-lsp language server
│   ├── gala-fmt/               # canonical formatter
│   ├── gala-pkg/               # manifest (gala.toml), resolver, registry client
│   └── gala-std/               # standard library (written in Gala + intrinsics)
├── std/                        # Gala-source standard library modules
├── tests/                      # conformance suite, integration tests
└── docs/                       # this suite
```

## 3. Crate responsibilities & key dependencies

| Crate | Responsibility | Primary external crates |
|-------|----------------|-------------------------|
| `gala-lexer` | Bytes → tokens | `logos` |
| `gala-parser` | Tokens → AST, error recovery | `chumsky` |
| `gala-span` | Spans, source map, string interning | `lasso` |
| `gala-diagnostics` | Rich diagnostics + error-code registry | `ariadne` |
| `gala-hir` | Desugar, name resolution, modules | `salsa` |
| `gala-types` | Type/effect inference, linearity check | `salsa`, `ena` (union-find) |
| `gala-uncompute` | Liftability analysis, uncompute synthesis | `salsa` |
| `gala-gir` | The IR data model + builders | `petgraph` (op-graph) |
| `gala-diff` | Gradient lowering | — |
| `gala-opt` | Circuit/classical optimization | — |
| `gala-qir` | Emit QIR (LLVM IR) | `inkwell` |
| `gala-codegen-classical` | Classical machine code / JIT | `cranelift` and/or `inkwell` |
| `gala-sim` | State-vector & noisy simulation | `roqoqo`, `roqoqo-quest` |
| `gala-qir` interop out | Vendor backends via QIR | `qoqo_qir`, vendor SDKs |
| `gala-lsp` | Editor integration | `tower-lsp`, `salsa` |
| `gala-driver` | Salsa DB, compilation orchestration | `salsa` |
| `gala-cli` | User-facing commands | `clap` |
| `gala-pkg` | Packages, versions, registry | `serde`, `toml`, `semver` |

## 4. Data-flow overview

```
                    ┌─────────────────────────── salsa query DB ───────────────────────────┐
 source (.gala) ─▶  │ lex ─▶ parse ─▶ HIR (resolve) ─▶ types+effects ─▶ linearity ─▶ uncompute │
                    └───────────────────────────────────┬──────────────────────────────────┘
                                                         ▼
                                                       GIR  (typed, effect-annotated)
                                                         │
                          ┌──────────────────────────────┼───────────────────────────┐
                          ▼                               ▼                           ▼
                    gala-diff (grad)                  gala-opt                   (inspection/LSP)
                          │                               │
                          └───────────────┬───────────────┘
                                          ▼
                        ┌─────────────────┼──────────────────┐
                        ▼                 ▼                  ▼
                   gala-qir (QIR)   gala-sim (run now)   gala-codegen-classical
                        │                 │                  │
                        ▼                 ▼                  ▼
                 vendor hardware     results/state     native orchestration binary
```

## 5. Boundaries that matter

- **AST vs HIR:** the AST mirrors syntax (for `fmt` and the LSP); HIR is desugared and
  name-resolved (for type checking). Keep them separate so the formatter never sees desugaring.
- **Types/linearity/uncompute are three passes, one crate cluster:** they share the salsa DB and
  run in order, each emitting diagnostics, none mutating source.
- **GIR is the only thing backends see.** No backend imports the frontend. This is what makes
  adding a backend (a new simulator, a new vendor) a contained task.
- **The runtime is separate from codegen.** `gala-runtime` coordinates the hybrid loop at
  execution time (dispatching kernels, handling mid-circuit measurement and feedback); codegen
  crates only *produce* artifacts.

## 6. Why these crates (grounding)

- `chumsky` + `ariadne` are designed together for compiler-grade parsing and diagnostics.
- `salsa` is the incremental-computation engine behind rust-analyzer — it gives us fast rebuilds
  and a responsive LSP from one query graph.
- `inkwell` provides safe LLVM bindings, and **QIR is LLVM IR**, so QIR emission and classical
  LLVM codegen share a toolchain.
- `roqoqo` + `roqoqo-quest` provide a QuEST-backed state-vector and noisy simulator (GPU/
  distributed capable), and `qoqo_qir` bridges to QIR and vendor backends — so simulation and
  cross-vendor execution are off-the-shelf.
- `tower-lsp` is the standard Rust LSP framework.

## 7. Build & release shape

- Single primary binary `gala` (from `gala-cli`), plus `gala-lsp` (may be folded into `gala lsp`).
- LLVM is the one heavy native dependency (via `inkwell`); the simulator pulls QuEST via
  `roqoqo-quest`. Both are gated behind cargo features so a "frontend-only" build (checker + LSP)
  compiles without them — important for fast CI and editor installs.
- Reproducible builds via a pinned `rust-toolchain.toml` and `Cargo.lock`.
