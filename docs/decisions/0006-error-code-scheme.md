# ADR 0006: Error code scheme

- Status: Accepted
- Date: 2026-10-22

## Context

Diagnostics need stable identifiers so that `gala explain`, the conformance suite, and editor
integrations can refer to them. Codes must survive rewording of the message text.

## Decision

- A code is `E` followed by four digits, for example `E0412`. `DiagnosticCode` in
  `crates/gala-diagnostics` stores the numeric part.
- The category is the hundreds digit:

| Range | Category | Examples |
|-------|----------|----------|
| E01xx | Syntax (lexer and parser) | E0101 unexpected token, E0110 unexpected end of file |
| E02xx | Types and name resolution | E0201 type mismatch, E0202 unresolved name |
| E03xx | Effects | E0301 declared effect too weak, E0304 measurement outside `prob` |
| E04xx | Linearity | E0401 no-cloning, E0402 implicit drop, E0403 same qubit twice, E0412 use after consume |
| E05xx | Uncomputation | E0530 cannot uncompute automatically |
| E06xx | Backend and target | E0601 capability mismatch, E0602 QIR emission failed, E0603 hardware unavailable |
| E07xx | Runtime | E0701 runtime error, E0702 simulator capacity exceeded |

- Every code in the registry must have a `gala explain` entry in
  `crates/gala-diagnostics/src/explain.rs`, and the conformance suite names codes in its
  `// expect:` headers.
- The registry test `registry_codes_are_unique` in `explain.rs` enforces that no two registry
  entries share a number.

## Consequences

- A new diagnostic takes its category from the hundreds digit, which makes the code predictable
  for readers and for tooling.
- Some categories are not yet full. E0207 (feature not supported yet) sits in the types range,
  and E0405 (branch linearity) sits in the linearity range. Both are kept where they are.
- Changing a code's meaning is a language change and requires an RFC (CONTRIBUTING §4).
