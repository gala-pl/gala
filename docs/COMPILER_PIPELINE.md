# 04 — Compiler Pipeline

This doc walks the compilation from source to artifact, pass by pass, and defines the `salsa`
query graph. Crate names refer to [ARCHITECTURE.md](./ARCHITECTURE.md).

## 1. Pipeline overview

```
.gala source
   │  gala-lexer (logos)
   ▼
Tokens ── gala-parser (chumsky) ──▶ AST (+CST for fmt/LSP)
   │
   ▼  gala-hir
HIR: desugared, name-resolved, module graph built
   │
   ▼  gala-types
Typed HIR: types + effects inferred and checked
   │
   ▼  gala-types (linearity) ──▶ gala-uncompute
Linearity verified; uncompute steps synthesized & inserted
   │
   ▼  gala-gir
GIR: typed, effect-annotated op-graph (classical + quantum in one graph)
   │
   ├─ gala-diff:  grad(...) lowered (param-shift for quantum, autodiff for classical)
   ├─ gala-opt:   circuit + classical optimization
   ▼
Lowered GIR
   │
   ├─▶ gala-qir  ──▶ QIR (LLVM IR) ──▶ vendor hardware (via qoqo_qir / SDKs)
   ├─▶ gala-sim  ──▶ simulator execution (roqoqo/QuEST)
   └─▶ gala-codegen-classical ──▶ native orchestration binary
```

## 2. Passes in detail

### 2.1 Lexing (`gala-lexer`)
`logos` tokenizer. Produces a token stream with byte spans. Handles nested block comments,
numeric/complex literals, and Unicode identifiers (Greek letters). Lossless enough that trivia
(comments, whitespace) is recoverable for the formatter.

### 2.2 Parsing (`gala-parser`)
`chumsky` parser with **error recovery** — a syntax error does not abort; it produces a partial
AST plus diagnostics, so the LSP stays useful in broken files. Emits both an AST (semantic) and a
concrete syntax view (for `fmt`/LSP).

### 2.3 HIR construction & name resolution (`gala-hir`)
Desugars surface sugar (e.g., in-place gate statements → explicit rebinds, `for` over ranges),
resolves names against the module graph, and assigns stable `DefId`s. This is the first salsa
layer: `hir(file)` is a query.

### 2.4 Type & effect inference (`gala-types`)
Bidirectional type checking with Hindley–Milner-style inference for the classical fragment,
extended with:
- **Effect inference** over the `pure ⊑ quantum ⊑ prob` lattice.
- **Const-generic** solving for `Qubits<N>` sizes.
Union-find (`ena`) backs unification. Produces a fully-typed HIR and type diagnostics.

### 2.5 Linearity checking (`gala-types` / dedicated sub-pass)
A liveness/ownership analysis over typed HIR verifying the linear discipline (§1 of
[TYPE_SYSTEM.md](./TYPE_SYSTEM.md)): every qubit used exactly once, no use-after-consume, no
aliasing, no implicit drop. Emits the teaching-grade `E04xx`/`E05xx` diagnostics.

### 2.6 Uncomputation synthesis (`gala-uncompute`)
Computes per-value **provenance** (the reversible op-graph that produced it). At each scope exit /
`drop` of a liftable ancilla, synthesizes `adjoint(provenance)` and inserts it. If a value is not
liftable (depends on measurement or irreversible input), emits `E0530` with the blocking reason.

### 2.7 GIR lowering (`gala-gir`)
Lowers typed HIR to **GIR**: a graph where nodes are classical or quantum operations, edges carry
linear-value ownership and classical data dependencies, and every node is typed and
effect-annotated. GIR is the stable contract for everything downstream and is dumpable
(`gala build --emit gir`).

### 2.8 Differentiation (`gala-diff`)
For each `grad(f, wrt: p)`, walks GIR: quantum-parameter dependencies become parameter-shift
evaluation nodes; classical dependencies become reverse-mode autodiff; the two compose by the
chain rule at the boundary. Output is more GIR (gradients are first-class, visible to the
optimizer).

### 2.9 Optimization (`gala-opt`)
GIR → GIR rewrites:
- **Quantum:** gate fusion, adjacent-gate cancellation, rotation merging, ancilla scheduling,
  gate-count/depth reduction (hardware-agnostic).
- **Classical:** constant folding, DCE, inlining of small `pure` functions.
- Optimizations preserve effect annotations and linearity invariants (checked in debug builds).

### 2.10 Backends
- **`gala-qir`:** GIR → QIR (LLVM IR) via `inkwell`. Chooses a QIR *profile* (base vs adaptive)
  based on whether the program uses mid-circuit measurement / conditioned control flow. See
  [BACKENDS_RUNTIME.md](./BACKENDS_RUNTIME.md).
- **`gala-sim`:** GIR executed on `roqoqo`/QuEST (state-vector or noisy).
- **`gala-codegen-classical`:** the classical orchestration layer to native code via `cranelift`
  (fast debug) or LLVM (release).

## 3. Hardware lowering (separate, inspectable pass)

Between optimized GIR and a specific device, a **target-lowering** pass maps the logical qubit
model to physical constraints: qubit routing/SWAP insertion for limited connectivity,
decomposition into the device's native gate set, and scheduling. Kept distinct and dumpable so the
"logical model up top, physical constraints at the bottom" separation is real and debuggable.

## 4. Salsa query graph (incremental core)

Queries (memoized, demand-driven, invalidated on input change):

```
source_text(FileId)                      -> String                [input]
tokens(FileId)                           -> Vec<Token>
ast(FileId)                              -> Ast
module_graph(Crate)                      -> ModuleGraph
hir(FileId)                              -> Hir
resolve(DefId)                           -> Resolution
type_of(DefId)                           -> Ty
effect_of(DefId)                         -> Effect
linearity_check(DefId)                   -> Vec<Diagnostic>
uncompute(DefId)                         -> UncomputePlan | Vec<Diagnostic>
gir(DefId)                               -> Gir
gir_diff(DefId)                          -> Gir           (after grad lowering)
gir_opt(DefId)                           -> Gir
diagnostics(FileId)                      -> Vec<Diagnostic>   (aggregate)
```

Because batch compilation and the LSP consume the *same* queries, an editor keystroke
recomputes only the affected sub-graph, and `gala build` reuses cached results across runs.

## 5. Error philosophy in the pipeline

- **No early abort on recoverable errors.** Each stage produces a best-effort result + diagnostics
  so later stages (and the LSP) still add value.
- **Every diagnostic has a code** (`E04xx` linearity, `E05xx` uncomputation, `E03xx` effects,
  `E02xx` types, `E01xx` syntax) and a `--explain E0530` long-form entry.
- **Diagnostics speak physics.** Where a rule encodes a physical law, the message says so.

## 6. Determinism & reproducibility

Given the same source, toolchain version, and target, compilation is deterministic (stable GIR,
stable QIR). Simulation with a fixed RNG seed is reproducible; hardware runs are inherently
stochastic and surfaced as `Measured<T>` distributions.