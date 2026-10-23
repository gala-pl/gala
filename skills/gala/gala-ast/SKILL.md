---
name: gala-ast
description: Work with Gala's AST — add node types, implement visitors, maintain CST/AST separation
disable-model-invocation: true
allowed-tools: Bash(cargo build *) Bash(cargo test *) Read Write Edit Glob Grep
---

## Task

$ARGUMENTS

## Context

Gala's AST (`crates/gala-ast`) defines the **abstract syntax tree** nodes used after parsing. It is separate from the CST (concrete syntax tree) used by the formatter. The AST is the input to HIR lowering.

Key files:
- `crates/gala-ast/src/node.rs` — Core AST node definitions (Expr, Stmt, Item, Type, Pattern)
- `crates/gala-ast/src/visit.rs` — Visitor trait and walk functions
- `crates/gala-ast/src/span.rs` — Span/SourceMap integration (re-exports from gala-span)
- `crates/gala-ast/src/token.rs` — Token wrapper for CST interop

## Common Tasks

### Add new AST node type
1. Add variant to appropriate enum in `node.rs` (e.g., `ExprKind`, `StmtKind`, `ItemKind`)
2. Derive `Clone`, `Debug`, `PartialEq`, `Hash`, `Serialize`, `Deserialize` as needed
3. Add visitor method in `visit.rs` (or use `#[derive(Visit)]` if using derive macro)
4. Update parser in `gala-parser` to produce the new node
5. Update HIR lowering in `gala-hir` to handle the new node
6. Update formatter in `gala-fmt` if syntax-affecting
7. Add tests in `gala-ast/tests/` and conformance tests

### Implement visitor for analysis/transformation
```rust
use gala_ast::visit::{Visit, VisitMut};

struct MyVisitor;
impl Visit for MyVisitor {
    fn visit_expr(&mut self, expr: &Expr) { walk_expr(self, expr); }
    // Override specific visit_* methods as needed
}
```

### Maintain AST vs CST separation
- **AST** (`gala-ast`): Semantic structure, no whitespace/comments, used for type checking
- **CST** (in `gala-parser`): Lossless syntax tree with trivia, used by `gala-fmt` and LSP
- Never put formatting info in AST; never put semantic info only in CST

## Testing

```bash
cargo test -p gala-ast                      # Unit tests
cargo test --workspace ast                  # All AST-related tests
```

## Conventions

- Node IDs: `NodeId` (u32) from `gala-span` for stable identity across passes
- Spans: `Span` (byte offset + length) from `gala-span` for diagnostics
- All public nodes documented with `///`
- No `unwrap()` on user paths
- Follow Rust edition 2021, max_width 100

## Key Types

```rust
// Core expressions
enum ExprKind { Literal, Path, Call, MethodCall, Binary, Unary, If, Match, For, While, Block, Lambda, Tuple, Array }

// Core statements
enum StmtKind { Let, Expr, Item }

// Top-level items
enum ItemKind { Fn, Struct, Enum, Trait, Impl, Import, TypeAlias, Const }

// Types
enum TypeKind { Path, Qubits, Qubit, Measured, Tuple, Array, Borrow, Fn }

// Patterns
enum PatKind { Ident, Literal, Tuple, Array, Struct, Wildcard, Or }
```