# Gala

A hybrid quantum-classical programming language whose compiler enforces the laws of quantum mechanics.

## Quick Start

```bash
cargo build --release
echo 'fn main() -> Int { print("Hello, Gala!"); return 0; }' > hello.gala
cargo run --release --bin gala -- hello.gala
```

## Project Structure

```
├── compiler/          # Gala compiler (lexer, parser, IR, codegen)
├── library/
│   ├── core/          # Core runtime (no_std primitives)
│   └── std/           # Standard library (I/O, collections, strings)
├── tools/
│   ├── gala-lsp/      # Language server protocol implementation
│   └── gala-fmt/      # Code formatter
├── docs/              # Language specification and guides
├── tests/             # Integration tests and test fixtures
└── examples/          # Example Gala programs
```

## Features

- **Strong static typing** with type inference
- **Expression-oriented** — blocks, if/else, and function bodies are expressions
- **Pattern matching** on algebraic data types
- **Zero-cost abstractions** via monomorphization
- **Safe concurrency** with ownership and borrowing
- **WASM target** support via `gala-core`

## Building

Requires Rust 1.75+.

```bash
# Build everything
cargo build --workspace

# Run tests
cargo test --workspace

# Compile a Gala program
cargo run --bin gala -- examples/hello_world.gala
```
