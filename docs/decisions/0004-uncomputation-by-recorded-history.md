# ADR 0004: Uncomputation by static liftability check and recorded history

- Status: Accepted
- Date: 2026-10-22

## Context

[TYPE_SYSTEM.md](../TYPE_SYSTEM.md) §2 describes safe automatic uncomputation: when an ancilla
is dropped, the compiler synthesizes its adjoint and returns the qubit to |0⟩. One design builds a
static adjoint circuit at compile time. That needs every gate's inverse, and every path through
the program, to be known statically, including branches and angles that depend on runtime values.

## Decision

- The compiler checks **liftability** statically in `crates/gala-uncompute`. At `drop(q)` it
  collects the causal cone of lineages entangled with `q` since its allocation and checks that
  (1) every gate in the cone is unitary, (2) every qubit in the cone is still live, and (3) the cone
  does not pass through a call to a user function. If any check fails, the compiler reports
  **E0530** (cannot uncompute automatically) and names the blocking qubit.
- The inverse gates are not emitted as a static sequence. They are applied at run time from the
  recorded gate history (see `gala-gir`), so angles and branch paths are exact for the run that
  happened.

## Consequences

- No static adjoint of a whole program is built. The check is cheap and its refusals are precise.
- Correctness of the inverse depends on the recorded history being complete. The conformance
  cases `uncompute/auto_uncompute.gala` (accepted) and `uncompute/cannot_uncompute_measure.gala`
  (E0530) pin both sides of the rule.
- Programs that pass the static check but whose inverse is only available at run time cannot be
  inspected as a fixed circuit. This is a cost for the QIR backend and needs its own treatment
  when the profile is extended.
