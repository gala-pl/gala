# 06 — Toolchain & Developer Experience

DX is a first-class deliverable, not a follow-on. Everything the broader ecosystem learned over
two decades ships from the start.

## 1. The `gala` CLI (`gala-cli`)

One entry point (built with `clap`):

```
gala new <name>        # scaffold a project (gala.toml, src/main.gala)
gala run [file]        # compile + run on the built-in simulator (zero config)
gala build             # compile; --emit gir|qir|llvm|native; --target <backend>
gala test              # run tests incl. quantum property tests
gala fmt               # canonical formatter (check with --check)
gala repl              # interactive REPL with live visualization
gala add <pkg>         # add a dependency
gala explain <code>    # long-form explanation of a diagnostic code (e.g. E0530)
gala lsp               # run the language server (usually launched by the editor)
gala doc               # generate API docs from /// comments
```

`gala run` works with no configuration because the simulator is built in — the critical
first-five-minutes experience.

## 2. Language server (`gala-lsp`)

Built on `tower-lsp`, sharing the `salsa` query graph with the compiler so results are consistent
and incremental. Day-one features:

- Diagnostics on type/effect/linearity/uncomputation errors, live as you type (error recovery in
  the parser keeps it working in broken files).
- Hover: types **and effects** (`fn classify(...) -> Measured<Bool> prob`).
- Go-to-definition, find-references, rename, autocomplete.
- **Inline circuit diagrams:** a code lens / hover renders the circuit a `quantum` function builds.
- Inlay hints for inferred qubit-register sizes (`Qubits<N>`) and effects.

## 3. Diagnostics (`gala-diagnostics`)

Rendered with `ariadne` (the natural pair for a `chumsky` frontend). Standards:

- **Every diagnostic has a stable code** and a one-line summary + a labeled span + a `help:`.
- **Teach the rule.** When a diagnostic encodes a physical law (no-cloning, uncomputation), the
  message explains the physics and the fix.
- **`gala explain <code>`** gives the long-form writeup with examples of the mistake and the fix.

Error-code ranges: `E01xx` syntax, `E02xx` types, `E03xx` effects, `E04xx` linearity,
`E05xx` uncomputation, `E06xx` backend/capability.

## 4. REPL (`gala repl`)

Evaluate expressions incrementally and **see quantum state update live**: state-vector amplitudes,
Bloch-sphere for single qubits, and measurement histograms. Backed by `gala-sim`. Great for
teaching and exploration.

## 5. Formatter (`gala-fmt`)

One canonical style, no options (Go/gofmt philosophy). `gala fmt --check` for CI. Operates on the
lossless CST so comments and layout intent are preserved.

## 6. Package manager (`gala-pkg`)

- **Manifest:** `gala.toml` (name, version, deps, target defaults) — `serde`/`toml`.
- **Resolver:** `semver`-based dependency resolution with a lockfile (`gala.lock`) for
  reproducible builds.
- **Registry:** a curated central registry (crates.io-style) plus git and path dependencies.
- **Std is versioned** with the toolchain.

## 7. Project layout (convention over configuration)

```
myproject/
├── gala.toml
├── gala.lock
├── src/
│   ├── main.gala          # or lib.gala
│   └── ...
└── tests/
    └── ...
```

## 8. Testing (summary; full detail in 08)

`gala test` runs example-based tests **and** quantum-aware property tests that check invariants
(unitarity, reversibility round-trips, uncomputation correctness) automatically. This "tests that
understand quantum mechanics" capability is a signature DX feature. See
[TESTING_QA.md](./TESTING_QA.md).

## 9. Editor & ecosystem integrations (phased)

- VS Code extension wrapping `gala lsp` (syntax highlighting via a TextMate/tree-sitter grammar +
  the LSP).
- Neovim/Zed via the same LSP.
- Jupyter-style kernel (later) for notebook workflows, since the ML audience lives there.
- Python bridge (later) so Gala kernels can be called from existing PyTorch/JAX pipelines for
  incremental adoption (this is Option B's on-ramp, not a change to the language).

## 10. The DX north star

A newcomer runs `gala new demo && cd demo && gala run`, sees a Bell pair simulate and its circuit
render, then introduces a no-cloning bug and gets a diagnostic that *teaches them the physics and
points at the fix*. That loop — instant, visual, educational — is the adoption engine.
