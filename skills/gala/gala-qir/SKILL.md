---
name: gala-qir
description: Work with Gala's QIR emission — GIR to QIR (LLVM IR) via inkwell, base/adaptive profiles, vendor interop
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's QIR backend (`crates/gala-qir`) emits **QIR (Quantum Intermediate Representation)** — an LLVM-based vendor-neutral IR. Uses **inkwell** (safe LLVM bindings). QIR *is* LLVM IR, so classical control flow lowers naturally alongside quantum intrinsics.

> **Note on file layout:** The crate currently lives in a single `crates/gala-qir/src/lib.rs`. As it matures it may be split into `emit.rs`, `profile.rs`, `intrinsics.rs`, `validation.rs`, `target.rs` (see `docs/ARCHITECTURE.md`). When in doubt, search for the relevant code in `lib.rs`.

Key files:
- `crates/gala-qir/src/lib.rs` — Main GIR → QIR emission, profile selection, intrinsic mapping

## Common Tasks

### Add new quantum intrinsic mapping
1. Edit `lib.rs` — map GIR `NodeKind` to QIR intrinsic call
2. Use `inkwell` to build `CallInst` to `__quantum__qis__<gate>__body`
3. Handle qubit array indexing, control qubits
4. Test: emit QIR, validate with `llvm-dis`, run on simulator via `qoqo_qir`

### Modify profile selection (Base vs Adaptive)
1. Edit `lib.rs` — Adaptive profile required for:
   - Mid-circuit measurement (`measure` not at end)
   - Measurement-conditioned control flow (`if measure(q) { ... }`)
2. Base profile for straight-line circuits with terminal measurement
3. Validation emits compile error on mismatch

### Add vendor-specific extension
1. QIR is vendor-neutral; vendor extensions via `qoqo_qir` or vendor SDKs
2. Add capability descriptor in `lib.rs` (qubits, gates, connectivity, mid-circuit)
3. Register in `gala-runtime` backend registry
4. Test on target hardware (nightly CI)

### Debug QIR emission
1. `--emit qir` outputs `.ll` (LLVM IR text)
2. Use `llvm-dis` / `opt` to inspect
3. Compare with golden QIR in `tests/conformance/qir/`

## Testing

```bash
cargo test -p gala-qir                      # Unit tests (requires LLVM)
cargo test --workspace qir                  # All QIR tests
cargo test --workspace conformity/qir       # Conformance: golden QIR
cargo run --bin gala -- --emit qir examples/hello_world.gala  # Debug QIR
```

## Conventions

- **QIR profiles** — Base (straight-line) vs Adaptive (mid-circuit + feedback)
- **Capabilities checked at compile time** — mismatch = error `E06xx`
- **Classical control flow in LLVM** — natural lowering since QIR = LLVM IR
- **inkwell for safety** — no raw LLVM C API
- **Deterministic emission** — same source = same QIR
- No `unwrap()` on user paths

## Key QIR Intrinsics

```llvm
; Single-qubit gates
declare void @__quantum__qis__h__body(%Qubit*)
declare void @__quantum__qis__rx__body(%Qubit*, double)
declare void @__quantum__qis__ry__body(%Qubit*, double)
declare void @__quantum__qis__rz__body(%Qubit*, double)

; Two-qubit gates
declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)
declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)

; Measurement
declare i1 @__quantum__qis__mz__body(%Qubit*)

; Controlled / Adjoint (Adaptive profile)
declare void @__quantum__rt__controlled__apply(...)
declare void @__quantum__rt__adjoint__apply(...)
```

## Debugging

```bash
# Emit QIR as LLVM IR text
cargo run --bin gala -- --emit qir examples/hello_world.gala

# Emit QIR as bitcode
cargo run --bin gala -- --emit qir=bc examples/hello_world.gala

# Validate profile
cargo run --bin gala -- --validate-qir-profile examples/hello_world.gala

# Disassemble bitcode
llvm-dis output.bc -o output.ll
```