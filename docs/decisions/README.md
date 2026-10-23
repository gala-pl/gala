# Decision log (ADRs)

This directory is the decision log described in [CONTRIBUTING.md](../CONTRIBUTING.md) §6. Every
RFC outcome and every deviation from the spec lands here as a numbered, dated entry, so that the
design stays coherent across many contributors and many months.

## Format

- **File name:** `NNNN-short-slug.md`, with `NNNN` a zero-padded sequence number. Numbers are never
  reused. A superseded entry keeps its file and gets a `Superseded by` line.
- **Header:** title, `Status` (`Proposed`, `Accepted`, or `Superseded by NNNN`), and `Date`
  (`YYYY-MM-DD`, the day the decision was recorded).
- **Sections:**
  - `Context`: the problem and the constraints that were true at the time, with pointers to code
    or spec sections.
  - `Decision`: what was decided, stated so that a reader can check the code against it.
  - `Consequences`: what this makes easier, what it makes harder, and what follow-up work it
    implies.

## When to write one

- An RFC (see [RFC_TEMPLATE.md](../RFC_TEMPLATE.md)) is accepted or rejected.
- A spec deviation is found or chosen, including one made during implementation.
- A dependency, backend, or crate boundary is chosen, or a previously chosen one is dropped.

An ADR records a decision that the code has made or will make. It does not replace the
conformance suite, which remains the executable definition of behavior.

## Index

| No. | Title | Status |
|-----|-------|--------|
| [0001](0001-hand-written-lexer-and-parser.md) | Hand-written lexer and parser | Accepted |
| [0002](0002-pure-rust-state-vector-simulator.md) | Pure-Rust state-vector simulator as the default backend | Accepted |
| [0003](0003-qir-as-textual-llvm-ir.md) | QIR emitted as textual LLVM IR | Accepted |
| [0004](0004-uncomputation-by-recorded-history.md) | Uncomputation: static liftability check, inverse from recorded history | Accepted |
| [0005](0005-effect-lattice-and-in-place-gates.md) | Effect lattice and in-place gate semantics | Accepted |
| [0006](0006-error-code-scheme.md) | Error code scheme | Accepted |
