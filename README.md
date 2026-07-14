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
├── compiler/          # Gala compiler (lexer, parser, AST, IR, codegen)
├── library/           # Gala language libraries:
│   └── std/           # Standard library (intrinsics, gates, algorithms, ML, noise, hardware)
├── tools/
│   ├── gala-lsp/      # Language server protocol implementation
│   └── gala-fmt/      # Code formatter
├── docs/              # Language specification and guides
├── cookbook/          # Task-oriented recipe collection
├── tests/             # Integration tests and test fixtures
└── examples/          # Example Gala programs
```

## Cookbook

Browse the [cookbook](./cookbook/) for task-oriented recipes — runnable `.gala` programs with step-by-step walkthroughs covering classical, quantum, and hybrid patterns.

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
