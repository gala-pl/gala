# gala-compiler

The Gala programming language compiler — a hybrid quantum-classical language whose compiler enforces the laws of quantum mechanics at compile time.

## Overview

`gala-compiler` is the core of the Gala project. It compiles `.gala` source files through a multi-stage pipeline: lexing, parsing, intermediate representation, and code generation. The architecture is designed to expand into quantum-aware compilation passes including linear type checking, effect system analysis, uncomputation verification, and multiple backends (LLVM, QIR, simulation).

The compiler is both a library (`gala_compiler`) for programmatic use and a binary (`gala`) for command-line invocation.

## Compiler Pipeline

```
.gala source
  → Lexer  (hand-written tokenizer)
    → Parser  (recursive-descent Pratt parser)
      → AST  (expression-oriented, spans preserved)
        → IR  (instruction-based intermediate representation)
          → Codegen  (formatted output/LLVM)
```

### Current Passes

| Pass | Module | Description |
|------|--------|-------------|
| **Lexer** | `lexer.rs` | Tokenizes source into `Token` enum: literals (Int, Float, String, Bool), keywords (let, fn, if, else, return), operators, symbols. Handles string escapes and line comments. |
| **Parser** | `parser.rs` | Recursive-descent Pratt parser with binding power for operator precedence. Produces `Vec<Stmt>` AST. Supports infix/prefix operators, function definitions, if/else expressions, blocks. |
| **AST** | `ast.rs` | Expression-oriented AST: `Expr` (Int, Float, String, Bool, Ident, BinOp, UnOp, Call, If, Let, Block, FnDef), `Stmt` (Expr, Let, Return), `Type` (Int, Float, Bool, String, Unit, Fn, Named). |
| **IR** | `ir.rs` | Instruction-based intermediate representation: `IrValue` (constant values), `IrInst` (LoadConst, Add, Sub, Mul, Div, Eq, Ne, Neg, Not, And, Or, Call, Return, JmpIf, Jmp, Phi, Alloca, Store, Load). `IrBuilder` for instruction emission. |
| **Codegen** | `codegen.rs` | Recursive emitter taking `&[Stmt]` and producing formatted output with proper indentation and parenthesization. |
| **Diagnostics** | `diagnostics.rs` | Structured error reporting: `Diagnostic` with level (Error, Warning, Note), message, optional span, and notes. `Diagnostics` collection with `Display` implementation for rich terminal output. |

## Usage

```bash
# Build the compiler
cargo build --workspace

# Compile a .gala file (prints output to stdout)
cargo run --bin gala -- --input examples/hello_world.gala

# Write output to a file
cargo run --bin gala -- --input examples/hello_world.gala --output out.txt

# Specify emit mode
cargo run --bin gala -- --input examples/hello_world.gala --emit ir
```

### CLI Options

| Flag | Description |
|------|-------------|
| `-i`, `--input` | Path to input `.gala` source file (required) |
| `-o`, `--output` | Path to write compiled output (prints to stdout if omitted) |
| `-e`, `--emit` | Emit mode (e.g., `ir`, `ast`) |

## Library API

```rust
use gala_compiler::compile;

fn main() {
    let source = "fn main() -> Int { return 42; }";
    match compile(source) {
        Ok(output) => println!("{}", output),
        Err(diags) => {
            for diag in diags.diagnostics {
                eprintln!("{}", diag);
            }
        }
    }
}
```

### Public Modules

| Module | Description |
|--------|-------------|
| `ast` | AST type definitions (`Expr`, `Stmt`, `Type`, `FnDef`, `BinOp`, `UnOp`) |
| `codegen` | Code generation from parsed AST |
| `diagnostics` | Error and warning reporting infrastructure |
| `ir` | Intermediate representation types and builder |
| `lexer` | Tokenizer producing `Token` stream |
| `parser` | Recursive-descent parser producing `Vec<Stmt>` |

## Dependencies

- `clap` — CLI argument parsing (via workspace)
- `thiserror` — Error type derivation (via workspace)

## Planned Features

- [ ] Quantum type system: `Qubit`, `Qubits<N>`, `Measured<T>`
- [ ] Linear type checking (no-cloning enforcement)
- [ ] Effect system: `pure`, `quantum`, `prob`
- [ ] Uncomputation analysis pass
- [ ] LLVM backend via `inkwell`
- [ ] QIR backend for quantum circuit generation
- [ ] `roqoqo`/`roqoqo-quest` simulation backend
- [ ] Salsa-based incremental computation
- [ ] Rich `ariadne` diagnostics
