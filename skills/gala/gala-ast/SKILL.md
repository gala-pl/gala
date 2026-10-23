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

> **Note on file layout:** The crate currently lives in a single `crates/gala-ast/src/lib.rs`. As it matures it may be split into `node.rs`, `visit.rs`, `span.rs`, `token.rs` (see `docs/ARCHITECTURE.md`). When in doubt, search for the relevant code in `lib.rs`.

Key files:
- `crates/gala-ast/src/lib.rs` — Core AST node definitions (`Item`, `Expr`, `Stmt`, `Type`, `Pattern`), `Ident`, `Spanned`

## Common Tasks

### Add new AST node type
1. Add variant to appropriate enum in `lib.rs` (e.g., `ExprKind`, `StmtKind`, `ItemKind`)
2. Derive `Clone`, `Debug`, `PartialEq`, `Hash` as needed
3. Update parser in `gala-parser` to produce the new node
4. Update HIR lowering in `gala-hir` to handle the new node
5. Update formatter in `gala-fmt` if syntax-affecting
6. Add tests in `gala-ast/tests/` and conformance tests

### Implement visitor for analysis/transformation
```rust
// Visitors are typically implemented inline via pattern matching
// in the consuming crate (e.g., Desugarer in gala-hir)
fn walk_expr(expr: &Expr, f: &mut impl FnMut(&Expr)) {
    match expr {
        Expr::Binary(b) => { walk_expr(&b.lhs, f); walk_expr(&b.rhs, f); }
        Expr::Call(c) => { walk_expr(&c.callee, f); for a in &c.args { walk_expr(a, f); } }
        // ...
    }
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
enum Expr {
    Literal(Literal),
    Ident(Ident),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Call(CallExpr),
    MethodCall(MethodCallExpr),
    If(IfExpr),
    Match(MatchExpr),
    For(ForExpr),
    While(WhileExpr),
    Block(Block),
    Tuple(Vec<Expr>),
    Array(Vec<Expr>),
    Lambda(LambdaExpr),
    Field(FieldExpr),
    Index(IndexExpr),
    Let(LetExpr),
}

// Core statements
enum Stmt {
    Let(LetStmt),
    Expr(Expr),
    Item(Item),
    Return(Option<Expr>),
}

// Top-level items
enum Item {
    FnDef(FnDef),
    StructDef(StructDef),
    EnumDef(EnumDef),
    TraitDef(TraitDef),
    ImplBlock(ImplBlock),
    TypeAlias(TypeAlias),
    ConstDef(ConstDef),
    Import(Import),
}

// Types
enum Type {
    Path(Path),
    Qubit,
    Qubits(Box<ConstExpr>),
    Measured(Box<Type>),
    Tuple(Vec<Type>),
    Array(Box<Type>, Box<ConstExpr>),
    Named(Ident, Vec<TypeArg>),
    Fn { params: Vec<Type>, ret: Box<Type>, effect: Option<Effect> },
}

// Patterns
enum Pattern {
    Ident(Ident),
    Literal(Literal),
    Tuple(Vec<Pattern>),
    Array(Vec<Pattern>),
    Struct { path: Path, fields: Vec<(Ident, Pattern)>, rest: bool },
    Wildcard,
    Or(Vec<Pattern>),
}
```