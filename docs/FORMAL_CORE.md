# Formal core (WP-054): operational semantics sketch

Status: **sketch, not mechanized.** This document states a small-step operational semantics for
the subset of Gala that the compiler implements today, and gives proof sketches for progress,
preservation, and linearity soundness. Nothing here is checked by a proof assistant. The claims
are backed by the reasoning written below and by the conformance suite
([TESTING_QA.md](./TESTING_QA.md) §3), which is evidence and not proof.

The design intent is in [TYPE_SYSTEM.md](./TYPE_SYSTEM.md) §7. This file says what the
implementation actually checks, which is narrower in places.

## 1. Scope of the core

The core is the fragment of the language that the front end (`gala-parser`, `gala-hir`),
`gala-types`, `gala-uncompute`, and `gala-sim` handle end to end:

- **Classical expressions:** `Int`, `Float`, `Bool`, tuples, literals, arithmetic, comparison,
  `let` bindings, `if`/`else`, function calls, and `return`.
- **Qubit resources:** `qubit()` and `qubits<N>()` allocation; gates `h`, `x`, `ry`, `cx`, and so
  on, each consuming and returning its qubit(s); `measure(q)`, which consumes `q` and returns
  `Measured<Bool>`; and `drop(q)`, which releases a qubit after the uncomputation check.
- **Effects:** the lattice `pure ⊑ quantum ⊑ prob`, with a declared effect checked against the
  inferred one.

Outside the core, and therefore not covered by any claim here: effect polymorphism, `grad`
(rejected with E0207 today), `Qubits<N>` arithmetic beyond the size check, imports and modules,
and the QIR and hardware backends. Those are described by the conformance suite only.

## 2. Syntax (core)

```
Types      τ ::= Int | Float | Bool | (τ, …, τ) | Qubit | Qubits<n> | Measured<τ>
                 | τ →ε τ
Effects    ε ::= pure | quantum | prob               (pure ⊑ quantum ⊑ prob)
Values     v ::= n | b | (v, …, v) | q | ⟨ids⟩ | closure
Expr       e ::= v | x | let x = e; e | if e { e } else { e } | f(e, …, e)
                 | qubit() | qubits<n>() | g(e)      (g a gate)
                 | measure(e) | drop(e) | return e
Stmt-level: a sequence of let bindings ending in an expression.
```

Gates are the builtins of `library/std/gates.gala`. Each has type `Qubit →quantum Qubit` (or the
pair form for two-qubit gates), and each is a unitary on the state.

## 3. Runtime configuration

A configuration is `⟨σ, Σ, e⟩`, where:

- `σ` is the classical store (a finite map from variables to classical values);
- `Σ` is the quantum state: a finite set of allocated qubit identifiers together with a state
  vector over them (`gala-sim` representation), with a count of live qubits;
- `e` is the expression under evaluation.

A qubit identifier is **live** when it is allocated and has not been consumed by `measure` or
released by `drop`.

## 4. Typing judgment

`Γ ⊢ e : τ ! ε`, where `Γ` is split into an unrestricted part `Γ_u` (classical bindings, which may
be used any number of times) and a linear part `Γ_l` (bindings of types containing `Qubit`, which
must be used exactly once).

Key rules, stated informally:

- **Qubit variables are linear.** A variable of linear type is removed from `Γ_l` at each use, and
  the rule for a use requires it to be present. Using it a second time is a context miss, which
  is what the linearity checker reports as E0412.
- **Gates take and return.** `g : Qubit →quantum Qubit`. In `let r = g(q)`, `q` is removed from
  `Γ_l` and `r` is added to it. A bare statement `g(q)` writes the returned qubit back to `q`'s
  binding, so `q` stays in `Γ_l`; if it is never consumed, the scope-end rule (E0402) applies.
  Register elements are borrowed (ADR 0005), so `g(r[i])` leaves `r` in `Γ_l`.
- **Measurement is destructive and effectful.** `measure(q) : Measured<Bool> ! prob`, consuming `q`.
- **Drop requires liftability.** `drop(q)` is typed only if the uncomputation analysis accepts
  `q`'s causal cone (§6).
- **No implicit drop.** At the end of a scope, every linear binding must have been consumed. A
  violation is E0402.
- **Branches agree.** For `if e₁ { e₂ } else { e₃ }`, the linear contexts after `e₂` and `e₃` must
  be equal. A mismatch is E0405.
- **Effects.** `Γ ⊢ e : τ ! ε` with `ε` the join of the effects of its parts. A function declared
  with `ε_decl` must satisfy `ε ⊑ ε_decl`, else E0301; a call to a callee of effect `ε_c` from a
  caller of effect `ε` needs `ε_c ⊑ ε`, else E0302 or E0303; `measure` needs `prob`, else E0304.

## 5. Reduction (selected rules)

- **Allocation.** `⟨σ, Σ, qubit()⟩ → ⟨σ, Σ + {q ↦ |0⟩}, q⟩` for a fresh `q`.
- **Gate.** `⟨σ, Σ, g(q)⟩ → ⟨σ, U_g · Σ, q⟩`, where `U_g` acts on the live qubit `q` and the
  returned qubit is still live (under the same or a new name, per the typing rules above).
- **Measurement.** `⟨σ, Σ, measure(q)⟩ → ⟨σ, Π_b Σ / √p_b, b⟩` with probability `p_b`. The qubit
  `q` is removed from the live set.
- **Drop.** `⟨σ, Σ, drop(q)⟩ → ⟨σ, Σ', ()⟩`, where `Σ'` is `Σ` with `q` removed. The rule
  **requires** that `q` is in the reset state after the synthesized inverse, which is the content
  of the uncomputation condition in §6. Without that precondition, the reduction is undefined and
  the program is ill-typed.

Classical reductions are the usual call-by-value rules for `let`, `if`, and application.

## 6. Uncomputation condition

`drop(a)` is accepted by `gala-uncompute` when, at the drop point, the causal cone of `a` (the
lineages entangled with `a` since its allocation) satisfies:

1. every gate in the cone is unitary (all gates in the core are);
2. every qubit in the cone is still live (none has been measured or dropped);
3. the cone does not pass through a call to a user function.

Under these conditions the inverse of the recorded gate history maps the cone back to a state in
which `a` is `|0⟩` and separable from the rest. Otherwise the analysis refuses with E0530.

This is the condition the implementation checks. It is weaker than full "`a` is in `|0⟩` after
the inverse" as a statement about `Σ`, because the check is structural, not a computed fact about
the state vector. Section 8 says what is still open.

## 7. Claims

### 7.1 Progress (sketch)

*Claim:* if `∅ ; Γ ⊢ e : τ ! ε` and `e` is not a value, then `⟨σ, Σ, e⟩` reduces, or it is stuck
only on a `drop` whose precondition fails, which the type rules exclude.

*Sketch:* induction on the typing derivation. Classical cases are standard. A gate applied to a
live qubit reduces by the gate rule. `measure` and `qubit()` always reduce. `drop` is typed only
when §6 holds, so its precondition is available and it reduces. A variable of linear type that is
not in `Γ_l` cannot occur in a well-typed term, because that would require a second use.

*Status:* a hand-written argument at the level of the core. Not mechanized. Not checked against the
full rule set of the implementation (in particular the effect and branch rules).

### 7.2 Preservation (sketch)

*Claim:* if `⟨σ, Σ, e⟩ → ⟨σ', Σ', e'⟩` and `Γ ⊢ e : τ ! ε`, then there is `Γ'` with
`Γ' ⊢ e' : τ ! ε'` and `ε' ⊑ ε`, and the linear context is preserved up to the consumed and
produced qubits.

*Sketch:* case analysis on the reduction. Gate and allocation rules replace one live qubit by one
live qubit, so the linear context is unchanged up to renaming. Measurement moves one qubit from
live to classical, and the typing rule for `measure` produces `Measured<Bool>` with effect `prob`,
so the effect bound holds when the enclosing function is `prob`. Drop removes one live qubit,
matching the removal from `Γ_l`.

*Status:* sketch. The effect-bound step relies on the lattice join as implemented in
`gala-types`. Not mechanized.

### 7.3 Linearity soundness (sketch)

*Claim:* a well-typed core program never duplicates a qubit (no two live bindings name the same
qubit identifier), and never discards a live qubit except through `drop` or `measure`.

*Sketch:* the context rules move each linear binding, so two uses of one binding cannot both be
typed. The reduction rules never copy a qubit identifier. The only rules that remove a live qubit
are `measure` and `drop`, and both appear in the typing rules. Aliasing through a register is
handled by the borrow rule (ADR 0005): a register element used in a gate is not moved, and its
register is still counted once.

*Status:* sketch. The register-borrow case is the one that most needs a mechanized check.

### 7.4 Uncomputation correctness (sketch, partial)

*Claim:* if `gala-uncompute` accepts `drop(a)`, then after the recorded inverse is applied, `a` is
in `|0⟩` and unentangled from the live qubits.

*Sketch:* the cone condition (§6) ensures that every gate touching `a` since allocation is unitary,
and that its partners are still live, so the recorded gates are exactly the operations that
entangled `a`. Applying their inverses in reverse order restores the state before the first
entangling gate, which had `a` in `|0⟩`.

*Status:* **partial.** The argument relies on the recorded history being complete and on the
inverse being applied in the right order at run time. Both properties are tested by the
conformance cases `uncompute/auto_uncompute.gala`, `run/uncompute_runs.gala`, and
`uncompute/cannot_uncompute_measure.gala`, but not proven. There is no denotational density-matrix
model in the repository, so the claim that "`a` is `|0⟩`" is not checked against one.

## 8. What is not proven or mechanized

- No mechanized proof of progress, preservation, or linearity in any proof assistant.
- No density-matrix or denotational model, so the uncomputation claim is operational and
  tested, not proven. [TYPE_SYSTEM.md](./TYPE_SYSTEM.md) §7 mentions a denotational model as a
  goal; it does not exist yet.
- The effect-polymorphism rules, `grad`, imports, and the QIR and hardware backends are outside
  the core.
- The branch-linearity rule and the classical `if` on `Measured<Bool>` are described here from
  the checker's behavior, not from a separately written specification.
- Soundness of the simulator against physical hardware is not claimed. The simulator is the
  reference semantics for testing only.

## 9. Cross-links to add (for the lead)

These links are not added here; `TYPE_SYSTEM.md` is not edited by this work.

- In [TYPE_SYSTEM.md](./TYPE_SYSTEM.md) §7 (Formalization plan): link to this file as "current
  status of the formal core".
- In [TYPE_SYSTEM.md](./TYPE_SYSTEM.md) §2 (uncomputation): link §6 of this file for the
  liftability condition.
- In [TESTING_QA.md](./TESTING_QA.md) §3: link §8 of this file for the conformance suite's role as
  evidence rather than proof.
- In [AGENTIC_BUILD_PLAN.md](./AGENTIC_BUILD_PLAN.md) WP-054: note that the DoD item "mechanized
  checks where feasible" is not met, and link §8.
- In [decisions/0004-uncomputation-by-recorded-history.md](./decisions/0004-uncomputation-by-recorded-history.md):
  link §6 and §7.4.
