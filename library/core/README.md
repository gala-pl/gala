# gala-core

The `no_std` core runtime library for the Gala programming language — analogous to Rust's `core` crate. Provides the primitive type aliases and fundamental operations that the Gala compiler lowers to.

## Overview

`gala-core` defines the runtime representation of Gala's built-in types and their basic operations. It is designed to be minimal, portable, and dependency-free. The compiler emits calls to these functions rather than inlining them, keeping the codegen simple and the runtime behavior well-defined.

## Modules

### `int` — Integer Primitives

```rust
pub type Int = i64;

pub const ZERO: Int = 0;
pub const ONE: Int = 1;

pub fn add(a: Int, b: Int) -> Int;
pub fn sub(a: Int, b: Int) -> Int;
pub fn mul(a: Int, b: Int) -> Int;
pub fn div(a: Int, b: Int) -> Int;
```

Gala's integer type maps to `i64`. Provides arithmetic operations that the compiler emits calls to.

### `float` — Float Primitives

```rust
pub type Float = f64;

pub fn add(a: Float, b: Float) -> Float;
pub fn sub(a: Float, b: Float) -> Float;
pub fn mul(a: Float, b: Float) -> Float;
pub fn div(a: Float, b: Float) -> Float;
```

Gala's float type maps to `f64` (IEEE 754 double-precision). Provides standard arithmetic operations.

### `bool` — Boolean Primitives

```rust
pub type Bool = bool;

pub const TRUE: Bool = true;
pub const FALSE: Bool = false;

pub fn and(a: Bool, b: Bool) -> Bool;
pub fn or(a: Bool, b: Bool) -> Bool;
pub fn not(a: Bool) -> Bool;
```

Gala's boolean type maps to Rust's `bool`. Provides logical operations.

### `tuple` — Tuple Primitives

```rust
#[repr(C)]
pub struct Tuple2<T, U>(pub T, pub U);

#[repr(C)]
pub struct Tuple3<T, U, V>(pub T, pub U, pub V);
```

`#[repr(C)]` tuple structs for 2- and 3-element tuples, ensuring a stable ABI for compiler interop.

## Design

- **`#![no_std]`** — usable in embedded and WASM environments without the standard library
- **Zero dependencies** — keeps the runtime footprint minimal
- **Type aliases over newtypes** — the compiler distinguishes Gala types at the IR level, so Rust-level type aliases are sufficient for the runtime
- **Direct Rust primitive mapping** — `Int = i64`, `Float = f64`, `Bool = bool` — no overhead from wrapper types

## Planned Additions

- [ ] `Complex` type for quantum amplitudes
- [ ] `Params` type for variational parameters
- [ ] Quantum measurement result types
- [ ] Array/slice primitives
- [ ] `Qubit` runtime representation
