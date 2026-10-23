# RFC template

Use this for any change to observable language behavior: syntax, types, effects, or diagnostic
semantics. The process is in [CONTRIBUTING.md](./CONTRIBUTING.md) §4. Copy this file to
`docs/rfcs/NNNN-short-slug.md` (or the public RFC repository, once WP-055 creates it) and fill in
each section. Delete the guidance text in italics.

## Metadata

- **RFC number and title:**
- **Author:**
- **Status:** Draft | Under review | Accepted | Rejected | Superseded
- **Related WP:** (for example WP-054)
- **Date opened:**

## 1. Problem

*What does the current language or toolchain get wrong, or fail to express? Give a minimal
program that shows it, and the current output if there is one.*

## 2. Proposal

*The behavior after the change, stated precisely. Include the syntax, the typing and effect
rules, and the exact diagnostic codes that a refusal produces. Reuse existing codes where the
meaning is the same; a new code needs a `gala explain` entry (ADR 0006).*

## 3. Alternatives

*At least two alternatives, including "do nothing". For each, say why it was not chosen.*

## 4. Migration

*Which existing programs change behavior, and how. Name the conformance cases that will change,
and whether the change is a warning first or an error at once. If nothing existing changes, say
so and say how that was checked.*

## 5. Conformance cases

*The cases to add under `tests/conformance/`, one line each, with their `// expect:` header.
Include the failure and refusal paths (for example E0530), not only the accepted program.*

| File | Expectation | What it pins |
|------|-------------|--------------|
|      |             |              |

## 6. Decision-log entry

*The ADR that records the outcome, in `docs/decisions/` (see
[docs/decisions/README.md](./decisions/README.md)). Give its number and title here once it is
written.*

- **ADR:** NNNN, title

---

## Process (CONTRIBUTING §4)

1. **Write the RFC.** Problem, proposal, alternatives, and migration, in this template.
2. **Add conformance cases** that demonstrate the new behavior, including refusals. They are
   written before the implementation and fail until it lands.
3. **Record the decision** as an ADR in `docs/decisions/`, with the date it was accepted.
4. **Get architect approval** before implementation starts. `[spec-sensitive]` work packages in
   [AGENTIC_BUILD_PLAN.md](./AGENTIC_BUILD_PLAN.md) follow this even when the change looks small.
5. **Implement** against the accepted RFC, keeping the conformance suite as the gate
   ([TESTING_QA.md](./TESTING_QA.md) §3).
6. **Update the docs** in the same change: the language spec, the diagnostics registry, and
   [TYPE_SYSTEM.md](./TYPE_SYSTEM.md) if the type or effect rules moved.
