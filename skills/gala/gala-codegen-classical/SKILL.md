---
name: gala-codegen-classical
description: Work with Gala's classical codegen — Cranelift (debug) and LLVM (release) for orchestration layer
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's classical codegen (`crates/gala-codegen-classical`) compiles the **classical orchestration layer** (loops, optimizers, data handling, gradient descent) to native code. Uses **Cranelift** for fast debug builds, **LLVM (via inkwell)** for optimized releases.

Key files:
- `crates/gala-codegen-classical/src/cranelift.rs` — Cranelift backend
- `crates/gala-codegen-classical/src/llvm.rs` — LLVM backend (inkwell)
- `crates/gala-codegen-classical/src/lower.rs` — GIR classical ops → backend IR
- `crates/gala-codegen-classical/src/runtime.rs` — Runtime interface (memory, calls)

## Common Tasks

### Add classical op lowering
1. Edit `lower.rs` — map GIR classical `OpKind` (Add, Mul, Call, etc.) to backend IR
2. Implement for both Cranelift and LLVM backends
3. Handle calling convention for Gala → Rust FFI
4. Test: compile and run classical-only function

### Optimize codegen for release
1. Edit `llvm.rs` — add LLVM optimization passes (inline, vectorize, etc.)
2. Ensure `pure` functions get optimized aggressively
3. Benchmark: orchestration binary perf vs interpreted baseline

### Add runtime intrinsic
1. Edit `runtime.rs` — define host function signature
2. Implement in `gala-runtime` (Rust side)
3. Lower call in `lower.rs` to runtime function
4. Test: end-to-end hybrid program

### Switch default backend
- Debug: `cargo build --features codegen-cranelift` (default)
- Release: `cargo build --release --features codegen-llvm`
- Both features can coexist; CLI flag selects

## Testing

```bash
cargo test -p gala-codegen-classical        # Unit tests
cargo test --workspace codegen-classical    # All codegen tests
cargo build --release --features codegen-llvm  # Release build
cargo run --bin gala -- build examples/classical_only.gala --target native  # Debug
```

## Conventions

- **Orchestration layer only** — quantum ops go to QIR/sim, not here
- **Cranelift for speed** — fast incremental debug builds
- **LLVM for perf** — optimized release artifacts
- **Shared lowering** — `lower.rs` feeds both backends
- **FFI-compatible** — Gala calls Rust runtime, Rust calls Gala entry points
- No `unwrap()` on user paths

## Key Lowering

```rust
// Classical GIR ops → backend IR
fn lower_op(builder: &mut Builder, op: &Op) {
    match op.kind {
        OpKind::Add => builder.ins().fadd(op.inputs[0], op.inputs[1]),
        OpKind::Mul => builder.ins().fmul(op.inputs[0], op.inputs[1]),
        OpKind::Call(func_id) => lower_call(builder, func_id, op.inputs),
        OpKind::Return => builder.ins().return_(&op.inputs),
        // ... control flow, memory, etc.
    }
}
```

## Debugging

```bash
# Emit Cranelift IR (debug)
cargo run --bin gala -- --emit clif examples/classical_only.gala

# Emit LLVM IR (release)
cargo run --bin gala -- --emit llvm examples/classical_only.gala --features codegen-llvm

# Run compiled binary
cargo run --bin gala -- build examples/classical_only.gala --target native
./target/gala/examples/classical_only
```