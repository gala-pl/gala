//! AST and CST types with visitors for Gala.

use gala_span::{ByteSpan, FileId, Span};
use std::fmt;

/// An identifier (interned string).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident(pub String);

impl Ident {
    pub fn new(s: impl Into<String>) -> Self {
        Ident(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A node in the AST with a span.
#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Spanned { node, span }
    }
}

/// Top-level AST node.
#[derive(Debug, Clone)]
pub enum Item {
    FnDef(FnDef),
    StructDef(StructDef),
    EnumDef(EnumDef),
    TraitDef(TraitDef),
    ImplBlock(ImplBlock),
    TypeAlias(TypeAlias),
    ConstDef(ConstDef),
    Import(Import),
}

/// Function definition.
#[derive(Debug, Clone)]
pub struct FnDef {
    pub ident: Ident,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub ret_ty: Option<Type>,
    pub effect: Option<Effect>,
    pub body: Block,
    pub span: Span,
}

/// Generic parameter (type or const).
#[derive(Debug, Clone)]
pub enum GenericParam {
    Type { ident: Ident, bound: Option<Type> },
    Const { ident: Ident, ty: Type },
}

/// Function parameter.
#[derive(Debug, Clone)]
pub struct Param {
    pub mutable: bool,
    pub pattern: Pattern,
    pub ty: Type,
    pub span: Span,
}

/// Effect annotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Pure,
    Quantum,
    Prob,
}

/// Statement.
#[derive(Debug, Clone)]
pub enum Stmt {
    Let(LetStmt),
    Expr(Expr),
    Return(Option<Expr>),
    Item(Item),
}

/// Let statement.
#[derive(Debug, Clone)]
pub struct LetStmt {
    pub mutable: bool,
    pub pattern: Pattern,
    pub ty: Option<Type>,
    pub init: Option<Expr>,
    pub span: Span,
}

/// Expression.
#[derive(Debug, Clone)]
pub enum Expr {
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

/// Binary expression.
#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub lhs: Box<Expr>,
    pub op: BinOp,
    pub rhs: Box<Expr>,
    pub span: Span,
}

/// Binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
    Range,
}

/// Unary expression.
#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: UnOp,
    pub expr: Box<Expr>,
    pub span: Span,
}

/// Unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

/// Function call.
#[derive(Debug, Clone)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    pub span: Span,
}

/// Method call.
#[derive(Debug, Clone)]
pub struct MethodCallExpr {
    pub receiver: Box<Expr>,
    pub method: Ident,
    pub args: Vec<Expr>,
    pub span: Span,
}

/// If expression.
#[derive(Debug, Clone)]
pub struct IfExpr {
    pub cond: Box<Expr>,
    pub then_branch: Block,
    pub else_branch: Option<Box<Expr>>,
    pub span: Span,
}

/// Match expression.
#[derive(Debug, Clone)]
pub struct MatchExpr {
    pub scrutinee: Box<Expr>,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: Span,
}

/// For expression.
#[derive(Debug, Clone)]
pub struct ForExpr {
    pub pattern: Pattern,
    pub iterable: Box<Expr>,
    pub body: Block,
    pub span: Span,
}

/// While expression.
#[derive(Debug, Clone)]
pub struct WhileExpr {
    pub cond: Box<Expr>,
    pub body: Block,
    pub span: Span,
}

/// Block expression.
#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub tail: Option<Box<Expr>>,
    pub span: Span,
}

/// Lambda expression.
#[derive(Debug, Clone)]
pub struct LambdaExpr {
    pub params: Vec<Param>,
    pub ret_ty: Option<Type>,
    pub effect: Option<Effect>,
    pub body: Box<Expr>,
    pub span: Span,
}

/// Field access.
#[derive(Debug, Clone)]
pub struct FieldExpr {
    pub base: Box<Expr>,
    pub field: Ident,
    pub span: Span,
}

/// Index expression.
#[derive(Debug, Clone)]
pub struct IndexExpr {
    pub base: Box<Expr>,
    pub index: Box<Expr>,
    pub span: Span,
}

/// Let expression (for in-place sugar).
#[derive(Debug, Clone)]
pub struct LetExpr {
    pub mutable: bool,
    pub pattern: Pattern,
    pub ty: Option<Type>,
    pub init: Box<Expr>,
    pub span: Span,
}

/// Literal value.
#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Complex { re: f64, im: f64 },
    Bool(bool),
    String(String),
    Unit,
}

/// Pattern.
#[derive(Debug, Clone)]
pub enum Pattern {
    Ident(Ident),
    Wildcard,
    Tuple(Vec<Pattern>),
    Struct { path: Path, fields: Vec<(Ident, Pattern)> },
    Literal(Literal),
}

/// Type.
#[derive(Debug, Clone)]
pub enum Type {
    Path(Path),
    Qubits(Box<ConstExpr>),
    Qubit,
    Measured(Box<Type>),
    Tuple(Vec<Type>),
    Array(Box<Type>, Box<ConstExpr>),
    Fn { params: Vec<Type>, ret: Box<Type>, effect: Option<Effect> },
    Named(String, Vec<Type>),
}

/// Path (qualified name).
#[derive(Debug, Clone)]
pub struct Path {
    pub segments: Vec<PathSegment>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct PathSegment {
    pub ident: Ident,
    pub type_args: Vec<TypeArg>,
}

#[derive(Debug, Clone)]
pub enum TypeArg {
    Type(Type),
    Const(ConstExpr),
}

/// Constant expression (for const generics).
#[derive(Debug, Clone)]
pub enum ConstExpr {
    Int(i64),
    Ident(Ident),
    Binary { lhs: Box<ConstExpr>, op: BinOp, rhs: Box<ConstExpr> },
}

/// Struct definition.
#[derive(Debug, Clone)]
pub struct StructDef {
    pub ident: Ident,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<StructField>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub ident: Ident,
    pub ty: Type,
    pub span: Span,
}

/// Enum definition.
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub ident: Ident,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub ident: Ident,
    pub fields: Vec<Type>,
    pub span: Span,
}

/// Trait definition.
#[derive(Debug, Clone)]
pub struct TraitDef {
    pub ident: Ident,
    pub generics: Vec<GenericParam>,
    pub items: Vec<TraitItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TraitItem {
    Fn(FnSig),
}

/// Function signature (for traits).
#[derive(Debug, Clone)]
pub struct FnSig {
    pub ident: Ident,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub ret_ty: Option<Type>,
    pub effect: Option<Effect>,
    pub span: Span,
}

/// Impl block.
#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub ty: Type,
    pub effect: Option<Effect>,
    pub items: Vec<Item>,
    pub span: Span,
}

/// Type alias.
#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub ident: Ident,
    pub generics: Vec<GenericParam>,
    pub ty: Type,
    pub span: Span,
}

/// Const definition.
#[derive(Debug, Clone)]
pub struct ConstDef {
    pub ident: Ident,
    pub ty: Type,
    pub value: Expr,
    pub span: Span,
}

/// Import statement.
#[derive(Debug, Clone)]
pub struct Import {
    pub path: Path,
    pub alias: Option<Ident>,
    pub span: Span,
}

// Visitor and walk functions removed for now to simplify compilation
// They can be added back later

#[cfg(test)]
mod tests {
    use super::*;
    use gala_span::{FileId, Span};

    #[test]
    fn test_ast_construction() {
        let span = Span::new(
            FileId(0),
            ByteSpan::new(0, 10),
        );

        let expr = Expr::Literal(Literal::Int(42));
        let item = Item::FnDef(FnDef {
            ident: Ident::new("main"),
            generics: vec![],
            params: vec![],
            ret_ty: None,
            effect: None,
            body: Block {
                stmts: vec![Stmt::Expr(Expr::Literal(Literal::Int(0)))],
                tail: None,
                span,
            },
            span,
        });

        assert!(matches!(item, Item::FnDef(_)));
    }
}