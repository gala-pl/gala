use crate::ast::*;
use crate::lexer::{Lexer, Token};

#[derive(Debug)]
pub struct Parser {
    lexer: Lexer,
    current: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current = lexer.next_token();
        Self { lexer, current }
    }

    fn advance(&mut self) -> Token {
        let prev = self.current.clone();
        self.current = self.lexer.next_token();
        prev
    }

    fn expect(&mut self, expected: &Token) -> Result<Token, String> {
        if std::mem::discriminant(&self.current) == std::mem::discriminant(expected) {
            Ok(self.advance())
        } else {
            Err(format!("Expected {expected}, got {}", self.current))
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        while self.current != Token::Eof {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match &self.current {
            Token::Let => {
                self.advance();
                let name = self.expect_ident()?;
                self.expect(&Token::Eq)?;
                let expr = self.parse_expr(0)?;
                self.expect(&Token::Semicolon)?;
                Ok(Stmt::Let(name, Box::new(expr)))
            }
            Token::Return => {
                self.advance();
                if self.current == Token::Semicolon {
                    self.advance();
                    Ok(Stmt::Return(None))
                } else {
                    let expr = self.parse_expr(0)?;
                    self.expect(&Token::Semicolon)?;
                    Ok(Stmt::Return(Some(expr)))
                }
            }
            Token::Fn => {
                self.advance();
                let expr = self.parse_fn_def()?;
                Ok(Stmt::Expr(expr))
            }
            Token::If => {
                self.advance();
                let expr = self.parse_if_expr()?;
                Ok(Stmt::Expr(expr))
            }
            _ => {
                let expr = self.parse_expr(0)?;
                self.expect(&Token::Semicolon)?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match &self.current {
            Token::Ident(s) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            }
            _ => Err(format!("Expected identifier, got {}", self.current)),
        }
    }

    fn parse_expr(&mut self, min_bp: u32) -> Result<Expr, String> {
        let mut lhs = match self.advance() {
            Token::Int(n) => Expr::Int(n),
            Token::Float(n) => Expr::Float(n),
            Token::String(s) => Expr::String(s),
            Token::True | Token::False => Expr::Bool(self.current == Token::False),
            Token::Ident(name) => Expr::Ident(name),
            Token::If => self.parse_if_expr()?,
            Token::Fn => self.parse_fn_def()?,
            Token::LBrace => self.parse_block()?,
            Token::LParen => {
                let expr = self.parse_expr(0)?;
                self.expect(&Token::RParen)?;
                expr
            }
            Token::Minus => {
                let rhs = self.parse_expr(prefix_bp(UnOp::Neg))?;
                Expr::UnOp(UnOp::Neg, Box::new(rhs))
            }
            Token::Bang => {
                let rhs = self.parse_expr(prefix_bp(UnOp::Not))?;
                Expr::UnOp(UnOp::Not, Box::new(rhs))
            }
            t => return Err(format!("Unexpected token: {t}")),
        };

        loop {
            let op = match &self.current {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                Token::EqEq => BinOp::Eq,
                Token::BangEq => BinOp::Ne,
                Token::Lt => BinOp::Lt,
                Token::Le => BinOp::Le,
                Token::Gt => BinOp::Gt,
                Token::Ge => BinOp::Ge,
                Token::AndAnd => BinOp::And,
                Token::OrOr => BinOp::Or,
                Token::LParen => {
                    self.advance();
                    let mut args = Vec::new();
                    if self.current != Token::RParen {
                        args.push(self.parse_expr(0)?);
                        while self.current == Token::Comma {
                            self.advance();
                            args.push(self.parse_expr(0)?);
                        }
                    }
                    self.expect(&Token::RParen)?;
                    lhs = Expr::Call(Box::new(lhs), args);
                    continue;
                }
                t if is_simple_stmt_start(t) => break,
                _ => break,
            };

            let (l_bp, r_bp) = infix_bp(op);
            if l_bp < min_bp {
                break;
            }
            self.advance();
            let rhs = self.parse_expr(r_bp)?;
            lhs = Expr::BinOp(Box::new(lhs), op, Box::new(rhs));
        }

        Ok(lhs)
    }

    fn parse_if_expr(&mut self) -> Result<Expr, String> {
        let cond = self.parse_expr(0)?;
        let then = self.parse_block_expr()?;
        let else_branch = if self.current == Token::Else {
            self.advance();
            Some(Box::new(if self.current == Token::If {
                self.parse_if_expr()?
            } else {
                self.parse_block_expr()?
            }))
        } else {
            None
        };
        Ok(Expr::If(Box::new(cond), Box::new(then), else_branch))
    }

    fn parse_block(&mut self) -> Result<Expr, String> {
        let mut stmts = Vec::new();
        while self.current != Token::RBrace && self.current != Token::Eof {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&Token::RBrace)?;
        Ok(Expr::Block(stmts))
    }

    fn parse_block_expr(&mut self) -> Result<Expr, String> {
        self.expect(&Token::LBrace)?;
        self.parse_block()
    }

    fn parse_fn_def(&mut self) -> Result<Expr, String> {
        let name = self.expect_ident()?;
        self.expect(&Token::LParen)?;
        let mut params = Vec::new();
        if self.current != Token::RParen {
            loop {
                let p_name = self.expect_ident()?;
                self.expect(&Token::Colon)?;
                let p_type = self.parse_type()?;
                params.push((p_name, p_type));
                if self.current == Token::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(&Token::RParen)?;
        let ret = if self.current == Token::Arrow {
            self.advance();
            self.parse_type()?
        } else {
            Type::Unit
        };
        self.expect(&Token::LBrace)?;
        let body = self.parse_block()?;
        Ok(Expr::FnDef(Box::new(FnDef { name, params, ret, body: Box::new(body) })))
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        let name = self.expect_ident()?;
        Ok(match name.as_str() {
            "Int" => Type::Int,
            "Float" => Type::Float,
            "Bool" => Type::Bool,
            "String" => Type::String,
            "Unit" => Type::Unit,
            _ => Type::Named(name),
        })
    }
}

fn is_simple_stmt_start(t: &Token) -> bool {
    matches!(t, Token::Semicolon | Token::RBrace | Token::Eof)
}

fn prefix_bp(_op: UnOp) -> u32 {
    100
}

fn infix_bp(op: BinOp) -> (u32, u32) {
    match op {
        BinOp::Or => (1, 2),
        BinOp::And => (3, 4),
        BinOp::Eq | BinOp::Ne => (5, 6),
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => (7, 8),
        BinOp::Add | BinOp::Sub => (9, 10),
        BinOp::Mul | BinOp::Div | BinOp::Mod => (11, 12),
    }
}
