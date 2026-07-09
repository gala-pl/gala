# 11 — Contributing & Agent Operating Rules

Conventions for humans and AI agents building Gala. The build backlog is in
[10-agentic-build-plan.md](./10-agentic-build-plan.md); this doc is the "how we work" contract.

## 1. Definition of Done (non-negotiable)

A change (or work package) is Done only when **all** hold:

1. Builds warning-free: `cargo build` and `cargo clippy -- -D warnings`.
2. Formatted: `cargo fmt --check` (Rust) and `gala fmt --check` (any `.gala`).
3. Tests pass: unit + relevant `proptest` properties + `insta` snapshots.
4. Conformance updated: cases added/updated for new or changed behavior, **including
   failure/refusal paths** (e.g. `E0530`).
5. Docs updated: `///` on public items, and this suite if behavior/architecture changed.
6. Diagnostics: any new code has an `explain` long-form entry and matches its documented `help:`.
7. Independent review approved (Builder ≠ Reviewer) and CI green.

Never mark Done with failing tests, partial implementation, `todo!()`/`unimplemented!()` on a
shipped path, or a weakened test.

## 2. Branching, commits, PRs

- Feature branches named `wp-XXX-short-slug`. Small, focused commits.
- Conventional-commits style (`feat:`, `fix:`, `refactor:`, `test:`, `docs:`).
- PR title references the WP id; PR body fills the DoD checklist (§1) and links the spec docs read.
- Squash-merge to `main`; `main` is always green (branch protection enforces CI + review).

## 3. Code conventions (Rust)

- Edition pinned in `rust-toolchain.toml`; `Cargo.lock` committed.
- Crate boundaries per [03-architecture.md](./03-architecture.md); **nothing above `gala-gir`
  imports a backend crate** (`gala-qir`, `gala-sim`, `gala-codegen-classical`). CI enforces this
  with a dependency lint.
- Prefer the designated crate over hand-rolling: parsing `chumsky`, lexing `logos`, diagnostics
  `ariadne`, incremental `salsa`, LLVM `inkwell`, simulation `roqoqo`/`roqoqo-quest`, LSP
  `tower-lsp`, CLI `clap`.
- Errors are values (`Result`, diagnostics); no `unwrap()` on user-reachable paths.
- Public APIs documented; internal invariants asserted in debug builds.

## 4. Language-change process (RFCs)

Any change to observable language behavior — syntax, types, effects, diagnostics semantics —
requires:
1. An RFC (problem, proposal, alternatives, migration).
2. Conformance cases demonstrating the new behavior.
3. A decision-log entry.
4. Architect approval before implementation.

WPs marked `[spec-sensitive]` in the build plan follow this even if they seem like "just
implementation," because their choices define behavior.

## 5. Agent operating rules (in addition to the above)

- **Stay in scope.** Touch only the crates your WP names. Cross-cutting changes need their own WP.
- **Plan before building.** Post approach + file list + test list; wait for Architect ack on
  `[spec-sensitive]` WPs.
- **Escalate ambiguity.** If the spec is unclear or you must deviate, stop and open a question with
  a proposed resolution. Do not guess.
- **No silent scope creep.** Discovered work becomes a new WP/issue, not a surprise in an unrelated
  PR.
- **Reproduce before fixing.** For bug WPs, add a failing test first, then fix.
- **Never weaken verification** to make CI pass.
- **Leave the campsite cleaner:** update docs and conformance in the same PR as the change.

## 6. Decision log

`docs/decisions/` holds dated, numbered ADR-style entries: context, decision, consequences. Every
RFC outcome and every spec deviation lands here. This is how the design stays coherent across many
agents and months.

## 7. Security & supply chain

- `cargo deny`/audit in CI for advisories and license policy.
- Pin and review new dependencies; prefer well-maintained crates.
- Hardware credentials/tokens never committed; backend adapters read from environment/secrets.

## 8. Releases

- Semantic versioning for the toolchain; std versioned with it.
- Changelog per release; conformance suite is the compatibility gate.
- Reproducible builds from a pinned toolchain + lockfile.