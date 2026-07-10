# gala-std

The standard library for the Gala programming language — analogous to Rust's `std` crate.

## Status

**Not yet implemented.** This directory is a placeholder for the future standard library.

## Planned Scope

The standard library will provide:

### I/O
- `print` / `println` — console output
- File reading and writing
- Standard input

### Collections
- `Vec<T>` — dynamic array
- `String` — UTF-8 string type
- `HashMap<K, V>` — hash map
- `Result<T, E>` — fallible result type
- `Option<T>` — optional value type

### Quantum Runtime
- `measure` — measurement operation
- Circuit construction helpers
- Simulation interfaces

### Math
- Trigonometric functions
- Random number generation
- Complex number operations

## Relationship to gala-core

`gala-core` provides the `no_std` primitive types and operations. `gala-std` builds on top of `gala-core` to provide richer abstractions that may require allocation, I/O, or OS support. The compiler can target either runtime depending on the target environment.
