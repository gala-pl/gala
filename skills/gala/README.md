# Gala Skills Index

This directory contains specialized skills for working with the **Gala quantum-classical programming language** compiler and toolchain.

## Compiler Pipeline Skills

| Skill | Description | Crate |
|-------|-------------|-------|
| [`gala-core`](gala-core/SKILL.md) | Core primitive types shared across compiler crates | `gala-core` |
| [`gala-lexer`](gala-lexer/SKILL.md) | Logos-based tokenizer — tokens, keywords, literals, Unicode | `gala-lexer` |
| [`gala-span`](gala-span/SKILL.md) | Source spans, file database, string interning | `gala-span` |
| [`gala-parser`](gala-parser/SKILL.md) | Chumsky parser with error recovery — grammar, AST/CST | `gala-parser` |
| [`gala-ast`](gala-ast/SKILL.md) | AST node definitions, visitor pattern, CST/AST separation | `gala-ast` |
| [`gala-hir`](gala-hir/SKILL.md) | HIR construction, desugaring, name resolution, salsa queries | `gala-hir` |
| [`gala-types`](gala-types/SKILL.md) | Type & effect inference, linearity, const generics, unification | `gala-types` |
| [`gala-uncompute`](gala-uncompute/SKILL.md) | Provenance tracking, liftability, adjoint synthesis, E0530 | `gala-uncompute` |
| [`gala-gir`](gala-gir/SKILL.md) | GIR data model, lowering, validation, JSON serialization | `gala-gir` |
| [`gala-diff`](gala-diff/SKILL.md) | Native differentiation — parameter-shift, autodiff, chain rule | `gala-diff` |
| [`gala-opt`](gala-opt/SKILL.md) | GIR optimizer — quantum/classical passes, gate fusion, DCE | `gala-opt` |

## Backend & Runtime Skills

| Skill | Description | Crate |
|-------|-------------|-------|
| [`gala-qir`](gala-qir/SKILL.md) | QIR emission via inkwell — base/adaptive profiles, vendor interop | `gala-qir` |
| [`gala-sim`](gala-sim/SKILL.md) | Simulator backends — QuEST state-vector, noisy simulation | `gala-sim` |
| [`gala-codegen-classical`](gala-codegen-classical/SKILL.md) | Classical codegen — Cranelift (debug), LLVM (release) | `gala-codegen-classical` |
| [`gala-runtime`](gala-runtime/SKILL.md) | Hybrid runtime — dispatch, mid-circuit, batching, caching | `gala-runtime` |

## Tooling & DX Skills

| Skill | Description | Crate |
|-------|-------------|-------|
| [`gala-diagnostics`](gala-diagnostics/SKILL.md) | Diagnostic rendering, error codes (E01xx–E06xx), explain, JSON, fixes | `gala-diagnostics` |
| [`gala-fuzz`](gala-fuzz/SKILL.md) | Shared fuzzing infrastructure — proptest/libfuzzer harnesses | `gala-fuzz` |
| [`gala-lint`](gala-lint/SKILL.md) | Linter for Gala source files — rules, CLI binary | `gala-lint` |
| [`gala-lsp`](gala-lsp/SKILL.md) | Language Server — tower-lsp, salsa sharing, circuit diagrams, agent methods | `gala-lsp` |
| [`gala-fmt`](gala-fmt/SKILL.md) | Canonical formatter — CST-based, idempotent, --check/--diff/--stdin | `gala-fmt` |
| [`gala-pkg`](gala-pkg/SKILL.md) | Package manager — gala.toml, resolver, registry, lockfile | `gala-pkg` |
| [`gala-cli`](gala-cli/SKILL.md) | CLI commands — build/check/run/test/fmt/repl/add/pkg, JSON modes | `gala-cli` |
| [`gala-driver`](gala-driver/SKILL.md) | Compilation driver — salsa DB orchestration, incremental queries | `gala-driver` |

## Library & Testing Skills

| Skill | Description | Location |
|-------|-------------|----------|
| [`gala-std`](gala-std/SKILL.md) | Standard library — intrinsics, gates, algorithms, VQA, noise | `gala-std`, `library/std/` |
| [`gala-testing`](gala-testing/SKILL.md) | Testing infra — conformance, property tests, snapshots, CI | `tests/`, `crates/*/tests/` |
| [`gala-contributing`](gala-contributing/SKILL.md) | Contribution workflow — WPs, conventional commits, PR process | `docs/PLAN.md` |
| [`gala-extensions`](gala-extensions/SKILL.md) | Editor extensions — 12+ editors, tree-sitter source of truth | `extensions/` |

## Quick Reference

### Build & Test Commands
```bash
cargo build --workspace                    # Build all
cargo test --workspace                     # Test all
cargo clippy --workspace -- -D warnings   # Lint (deny warnings)
cargo fmt --check                          # Check formatting
cargo doc --workspace --no-deps            # Verify docs
```

### Key Documentation
- [`ARCHITECTURE.md`](../docs/ARCHITECTURE.md) — Crate map, boundaries
- [`COMPILER_PIPELINE.md`](../docs/COMPILER_PIPELINE.md) — Pass-by-pass flow
- [`TYPE_SYSTEM.md`](../docs/TYPE_SYSTEM.md) — Linear types, effects, uncomputation
- [`BACKENDS_RUNTIME.md`](../docs/BACKENDS_RUNTIME.md) — QIR, simulators, runtime
- [`LANGUAGE_SPEC.md`](../docs/LANGUAGE_SPEC.md) — Surface syntax, grammar
- [`TESTING_QA.md`](../docs/TESTING_QA.md) — Testing strategy, conformance
- [`PLAN.md`](../docs/PLAN.md) — WP backlog, agent roles

### Error Code Ranges
| Range | Category |
|-------|----------|
| E01xx | Syntax / Lexer |
| E02xx | Types |
| E03xx | Effects |
| E04xx | Linearity (no-cloning) |
| E05xx | Uncomputation |
| E06xx | Backend / Differentiation |

### Effect Lattice
```
pure ⊑ quantum ⊑ prob
```

### Quantum Guarantees (enforced by compiler)
1. **No-cloning** — Linear types (move semantics, no `Copy`)
2. **No leaked entanglement** — Auto-uncomputation on `drop`
3. **Boundary safety** — Effect system (measurement = `prob`)
4. **Correct inverses** — Compiler-derived `adjoint`/`control`
5. **Correct gradients** — Native `grad` (param-shift + autodiff)
6. **Size/topology fit** — `Qubits<N>` const generics