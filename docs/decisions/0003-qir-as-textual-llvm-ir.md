# ADR 0003: QIR emitted as textual LLVM IR

- Status: Accepted
- Date: 2026-10-22

## Context

The plan names `inkwell` for LLVM bindings (WP-040). `inkwell` needs a matching system LLVM
installation, and the version has to agree with the binding. The QIR output only needs to be
text that downstream QIR tooling can consume. It does not need LLVM's optimizer or code
generator.

## Decision

- `crates/gala-qir` emits QIR as textual LLVM IR from GIR (`emit(gir) -> QirModule`). It writes
  the module by hand, with no LLVM library.
- The profile is chosen from the program shape. If any block ends in a `Branch` on a measurement
  result (mid-circuit measurement), the module uses the **adaptive** profile. Otherwise it uses
  the **base** profile. The check is `adaptive = any block has Term::Branch`.
- `inkwell` remains an optional dependency only in `crates/gala-codegen-classical`, behind the
  `classical` feature. It is not used for QIR.

## Consequences

- Building the compiler needs no system LLVM for QIR. The output is inspectable as plain text, and
  tests can inspect it directly. The conformance case `tests/conformance/qir/bell_qir.gala` checks
  that the Bell program is accepted; golden-file comparison of the emitted text is not yet in place.
- Our hand-written emitter must track the QIR specification itself. Any profile feature we
  need (for example richer classical result handling) is our work to add.
- The base/adaptive choice is local to the emitter, so the rule in the consequence above is the
  only place to change if the profile policy moves.
