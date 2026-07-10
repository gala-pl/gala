---
name: debug-compiler
description: Debug a compiler error or unexpected behavior in Gala source code
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Bash(cargo run --bin gala *) Read Grep Glob
---

## Task

Debug the following Gala compiler issue and find the root cause:

$ARGUMENTS

## Approach

1. Reproduce the issue by compiling the relevant `.gala` source:
   `cargo run --bin gala -- <path-to-file>`

2. Check existing tests to understand expected behavior:
   `cargo test --workspace -- <test-name>`

3. Trace through the compiler pipeline to find where the issue originates:
   - Lexer (`compiler/src/lexer.rs`)
   - Parser (`compiler/src/parser.rs`)  
   - AST/IR (`compiler/src/ast.rs`, `compiler/src/ir.rs`)
   - Codegen (`compiler/src/codegen.rs`)

4. Look at existing tests for similar features to understand the pattern

5. Suggest a fix with the minimal change needed

6. Verify the fix: run `cargo test --workspace` and `cargo clippy --workspace -- -D warnings`
