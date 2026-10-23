# ADR 0005: Effect lattice and in-place gate semantics

- Status: Accepted
- Date: 2026-10-22

## Context

The effect system has three levels, `pure ⊑ quantum ⊑ prob` ([TYPE_SYSTEM.md](../TYPE_SYSTEM.md)
§3). Two questions came up in implementation: what a gate call does to the name of its qubit, and
whether a measured result may control classical branching.

## Decision

- **Lattice.** `Effect` in `crates/gala-types` is a three-point join semilattice with `join`. A
  function's inferred effect is the join of its operations. A declared effect must cover the body
  (E0301), a `pure` body may not call a quantum function (E0302), a `quantum` body may not call a
  probabilistic one (E0303), and `measure` is only allowed in a `prob` function (E0304).
- **In-place gates.** A gate takes its qubit and returns the same qubit. Two forms are accepted:
  a bare statement `h(q)` keeps the name `q` bound to the returned qubit, so `q` stays usable (and
  must still be consumed, E0402 otherwise); a binding `let r = h(q)` moves `q` into `r`, so `q` is
  dead afterwards and reuse is E0412. Two-qubit gates return a pair, so `let (a, b) = cx(a, b)`
  rebinds both names. Gates that act on register elements borrow them, so `h(q[0])` does not
  consume the register (`linearity/register_borrow.gala`, accepted).
- **Measured conditions.** `Measured<Bool>` may be used as an `if` condition. This is mid-circuit
  branching, and it selects the QIR adaptive profile (ADR 0003).

## Consequences

- The classical/quantum boundary is checked at each call, and the errors name both effects.
- Rebinding makes gate chains read like straight-line code, but authors coming from circuit
  libraries must learn that the qubit name changes.
- The `Measured<Bool>` condition rule is a language commitment. Changing it requires an RFC under
  CONTRIBUTING §4.
