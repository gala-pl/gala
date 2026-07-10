---
paths:
  - "compiler/**/*.rs"
---

# Compiler Conventions

## Pipeline Architecture
- Each compiler pass is a separate function or module with a clear input/output type
- Passes are composed in `main.rs` / `lib.rs` as a pipeline
- Use `salsa` for incremental computation (HIR, name resolution, type checking)
- Span information must be preserved through all IR levels for good diagnostics

## Diagnostics
- Use `ariadne` for rich, colorful diagnostic output
- Every error must include source location (span)
- Suggest fixes where possible (e.g., "did you mean X?")
- Classify errors: type error, syntax error, linearity violation, quantum constraint, etc.

## Lexer (logos)
- Tokens carry span information
- Handle all Unicode characters gracefully
- Keywords are reserved; provide clear error messages for misuse

## Parser (chumsky)
- Error recovery: produce partial ASTs for IDE support
- Rich error messages with expected tokens and suggestions
- Support incremental reparsing

## Type System
- Strong static typing with Hindley-Milner-style inference
- Linear types for quantum values (no cloning, no implicit dropping)
- Custom effect system for quantum operations
- Uncomputation analysis is a separate compiler pass

## Codegen
- LLVM backend via `inkwell` for classical code
- QIR generation for quantum circuits
- `roqoqo` backend for simulation/testing
- WASM target support
