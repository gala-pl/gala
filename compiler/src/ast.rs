#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Ident(String),
    BinOp(Box<Expr>, BinOp, Box<Expr>),
    UnOp(UnOp, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    Let(String, Box<Expr>),
    Block(Vec<Stmt>),
    FnDef(Box<FnDef>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Let(String, Box<Expr>),
    Return(Option<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDef {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub ret: Type,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Unit,
    Fn(Vec<Type>, Box<Type>),
    Named(String),
}
