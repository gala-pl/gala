---
name: gala-contributing
description: Follow Gala's contribution workflow — conventional commits, feature branches, PR process, code style, guardrails
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Bash(git *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala follows a strict **agentic build plan** ([PLAN.md](../docs/PLAN.md)) with defined roles (Architect, Builder, Reviewer, Test/QA, Docs, Integrator) and an operating loop. All contributions must pass CI gates and independent review.

## Workflow

### 1. Claim a Work Package (WP)
- WPs defined in `docs/PLAN.md` §4
- Feature branch: `wp-XXX-short-slug` (e.g., `wp-013-type-inference`)
- Mark WP in-progress on tracking board

### 2. Plan (Architect ack for `[spec-sensitive]` WPs)
- Read WP entry + linked spec docs (`TYPE_SYSTEM.md`, `COMPILER_PIPELINE.md`, etc.)
- Write short plan: approach, files to change, test list
- For `[spec-sensitive]` WPs (013, 014, 015, 030, 040): wait for Architect ack

### 3. Build
- Small commits, conventional messages: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`
- Follow code style: Rust 2021, max_width 100, `reorder_imports`, documented public APIs
- Use designated crates: `chumsky`, `logos`, `ariadne`, `salsa`, `inkwell`, `roqoqo`, `tower-lsp`, `clap`
- **Never import backend crate above `gala-gir`**

### 4. Self-Check (must pass before PR)
```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
# Run relevant conformance cases
cargo test --workspace conformity/<category>
# Verify docs build
cargo doc --workspace --no-deps
```

### 5. PR
- Reference WP ID in title: `wp-013: Implement bidirectional type inference`
- Fill PR checklist (DoD from WP):
  - [ ] Builds warning-free
  - [ ] `cargo fmt --check` clean
  - [ ] Conformance tests added/updated
  - [ ] Property tests added (if applicable)
  - [ ] Public items documented
  - [ ] `explain` entries for new diagnostic codes
  - [ ] No `todo!()` in shipped paths
  - [ ] No backend imports above `gala-gir`

### 6. Review
- **Builder never reviews own WP**
- Reviewer verifies against DoD, runs branch, checks tests exercise behavior
- Approve only if every DoD item met

### 7. Merge
- Integrator merges when CI green + Reviewer approves
- Squash merge to `main`
- Docs updated in same PR

## Guardrails (from CONTRIBUTING.md)

- ❌ Never weaken a test to make CI pass — fix the code or escalate
- ❌ Never mark WP done with failing tests, partial impl, or `todo!()` in shipped paths
- ❌ If blocked or spec ambiguous — STOP and open question, don't guess
- ✅ Prefer designated crates over hand-rolling
- ✅ Keep frontend backend-agnostic (nothing above `gala-gir` imports backend)
- ✅ All errors are values — no `unwrap()` on user-reachable paths

## Code Style

```bash
# Format
cargo fmt --all

# Lint (deny warnings)
cargo clippy --workspace -- -D warnings

# Check docs
cargo doc --workspace --no-deps

# Dependency audit
cargo deny check
```

## Conventional Commits

| Prefix | Use |
|--------|-----|
| `feat:` | New feature |
| `fix:` | Bug fix |
| `refactor:` | Code restructuring |
| `test:` | Test additions/changes |
| `docs:` | Documentation |
| `chore:` | Maintenance, deps, CI |

## Branch Naming

```
wp-XXX-short-slug
# Examples:
wp-013-type-inference
wp-020-simulator-backend
wp-030-native-differentiation
wp-060-structured-diagnostics
```

## Pre-commit Hooks

```bash
# Install
pre-commit install

# Run manually
pre-commit run --all-files
```

## Getting Help

- Read `docs/PLAN.md` for full WP details
- Read `docs/CONTRIBUTING.md` for contribution guidelines
- Ask Architect for `[spec-sensitive]` clarifications
- Ask Reviewer for DoD interpretation