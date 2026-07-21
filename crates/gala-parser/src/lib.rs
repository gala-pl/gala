//! Hand-written parser for the Gala programming language.

use gala_ast::{Ident, Pattern};
use gala_diagnostics::{codes, Diagnostic, Diagnostics};
use gala_lexer::{Lexer, Token};
use gala_span::{FileId, SourceMap, Span};

/// Parse a source file into an AST with error recovery.
pub fn parse_file(
    file_id: FileId,
    source: &str,
    _source_map: &mut SourceMap,
) -> Result<Vec<gala_ast::Item>, Diagnostics> {
    let lexer = Lexer::new(file_id, source);
    let tokens = lexer.collect_all();
    parse(tokens.as_slice())
}

/// Parse a token stream into an AST.
pub fn parse(tokens: &[(Token, Span)]) -> Result<Vec<gala_ast::Item>, Diagnostics> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}

struct Parser<'a> {
    tokens: &'a [(Token, Span)],
    pos: usize,
    diags: Diagnostics,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [(Token, Span)]) -> Self {
        Parser { tokens, pos: 0, diags: Diagnostics::new() }
    }

    fn parse(&mut self) -> Result<Vec<gala_ast::Item>, Diagnostics> {
        let mut items = Vec::new();
        while !self.is_at_end() {
            match self.parse_item() {
                Ok(item) => items.push(item),
                Err(()) => self.synchronize(),
            }
        }
        if self.diags.has_errors() {
            Err(self.diags.clone())
        } else {
            Ok(items)
        }
    }

    fn parse_item(&mut self) -> Result<gala_ast::Item, ()> {
        if self.match_token(&Token::KwFn) {
            self.parse_fn_def()
        } else if self.match_token(&Token::KwStruct) {
            self.parse_struct_def()
        } else if self.match_token(&Token::KwEnum) {
            self.parse_enum_def()
        } else if self.match_token(&Token::KwTrait) {
            self.parse_trait_def()
        } else if self.match_token(&Token::KwImpl) {
            self.parse_impl_block()
        } else if self.match_token(&Token::KwType) {
            self.parse_type_alias()
        } else if self.match_token(&Token::KwConst) {
            self.parse_const_def()
        } else if self.match_token(&Token::KwImport) {
            self.parse_import()
        } else {
            self.error("expected item");
            Err(())
        }
    }

    fn parse_fn_def(&mut self) -> Result<gala_ast::Item, ()> {
        let ident = self.parse_ident()?;
        let generics = self.parse_generics()?;
        self.expect(&Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(&Token::RParen)?;
        let ret_ty = if self.match_token(&Token::Arrow) { Some(self.parse_type()?) } else { None };
        let effect = self.parse_effect()?;
        let body = self.parse_block()?;
        Ok(gala_ast::Item::FnDef(gala_ast::FnDef {
            ident,
            generics,
            params,
            ret_ty,
            effect,
            body,
            span: Span::dummy(),
        }))
    }

    fn parse_generics(&mut self) -> Result<Vec<gala_ast::GenericParam>, ()> {
        if !self.match_token(&Token::LAngle) {
            return Ok(Vec::new());
        }
        let mut params = Vec::new();
        while !self.check(&Token::RAngle) && !self.is_at_end() {
            params.push(self.parse_generic_param()?);
            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        self.expect(&Token::RAngle)?;
        Ok(params)
    }

    fn parse_generic_param(&mut self) -> Result<gala_ast::GenericParam, ()> {
        if self.match_token(&Token::KwConst) {
            let ident = self.parse_ident()?;
            self.expect(&Token::Colon)?;
            let ty = self.parse_type()?;
            Ok(gala_ast::GenericParam::Const { ident, ty })
        } else {
            let ident = self.parse_ident()?;
            let bound =
                if self.match_token(&Token::Colon) { Some(self.parse_type()?) } else { None };
            Ok(gala_ast::GenericParam::Type { ident, bound })
        }
    }

    fn parse_params(&mut self) -> Result<Vec<gala_ast::Param>, ()> {
        let mut params = Vec::new();
        while !self.check(&Token::RParen) && !self.is_at_end() {
            let mutable = self.match_token(&Token::KwMut);
            let pattern = self.parse_pattern()?;
            self.expect(&Token::Colon)?;
            let ty = self.parse_type()?;
            params.push(gala_ast::Param { mutable, pattern, ty, span: Span::dummy() });
            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        Ok(params)
    }

    fn parse_effect(&mut self) -> Result<Option<gala_ast::Effect>, ()> {
        if self.match_token(&Token::KwPure) {
            Ok(Some(gala_ast::Effect::Pure))
        } else if self.match_token(&Token::KwQuantum) {
            Ok(Some(gala_ast::Effect::Quantum))
        } else if self.match_token(&Token::KwProb) {
            Ok(Some(gala_ast::Effect::Prob))
        } else {
            Ok(None)
        }
    }

    fn parse_block(&mut self) -> Result<gala_ast::Block, ()> {
        self.expect(&Token::LBrace)?;
        let mut stmts = Vec::new();
        let mut tail = None;
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            if self.check(&Token::KwLet) || self.check(&Token::KwReturn) {
                stmts.push(self.parse_stmt()?);
            } else {
                let expr = self.parse_expr()?;
                if self.match_token(&Token::Semicolon) {
                    stmts.push(gala_ast::Stmt::Expr(expr));
                } else {
                    tail = Some(Box::new(expr));
                    break;
                }
            }
        }
        self.expect(&Token::RBrace)?;
        Ok(gala_ast::Block { stmts, tail, span: Span::dummy() })
    }

    fn parse_stmt(&mut self) -> Result<gala_ast::Stmt, ()> {
        if self.match_token(&Token::KwLet) {
            let mutable = self.match_token(&Token::KwMut);
            let pattern = self.parse_pattern()?;
            let ty = if self.match_token(&Token::Colon) { Some(self.parse_type()?) } else { None };
            self.expect(&Token::Eq)?;
            let init = self.parse_expr()?;
            self.expect(&Token::Semicolon)?;
            Ok(gala_ast::Stmt::Let(gala_ast::LetStmt {
                mutable,
                pattern,
                ty,
                init: Some(init),
                span: Span::dummy(),
            }))
        } else if self.match_token(&Token::KwReturn) {
            let expr = if !self.check(&Token::Semicolon) { Some(self.parse_expr()?) } else { None };
            self.expect(&Token::Semicolon)?;
            Ok(gala_ast::Stmt::Return(expr))
        } else {
            let expr = self.parse_expr()?;
            self.expect(&Token::Semicolon)?;
            Ok(gala_ast::Stmt::Expr(expr))
        }
    }

    fn parse_expr(&mut self) -> Result<gala_ast::Expr, ()> {
        self.parse_binary(0)
    }

    fn parse_binary(&mut self, min_bp: u8) -> Result<gala_ast::Expr, ()> {
        let mut lhs = self.parse_unary()?;

        while !self.is_at_end() {
            let op = match self.current_token() {
                Some((Token::Plus, _)) => {
                    self.advance_token();
                    gala_ast::BinOp::Add
                }
                Some((Token::Minus, _)) => {
                    self.advance_token();
                    gala_ast::BinOp::Sub
                }
                Some((Token::Star, _)) => {
                    self.advance_token();
                    gala_ast::BinOp::Mul
                }
                Some((Token::Slash, _)) => {
                    self.advance_token();
                    gala_ast::BinOp::Div
                }
                Some((Token::Percent, _)) => {
                    self.advance_token();
                    gala_ast::BinOp::Mod
                }
                Some((Token::EqEq, _)) => {
                    self.advance_token();
                    gala_ast::BinOp::Eq
                }
                Some((Token::BangEq, _)) => {
                    self.advance_token();
                    gala_ast::BinOp::Ne
                }
                Some((Token::Lt, _)) => {
                    self.advance_token();
                    gala_ast::BinOp::Lt
                }
                Some((Token::Le, _)) => {
                    self.advance_token();
                    gala_ast::BinOp::Le
                }
                Some((Token::Gt, _)) => {
                    self.advance_token();
                    gala_ast::BinOp::Gt
                }
                Some((Token::Ge, _)) => {
                    self.advance_token();
                    gala_ast::BinOp::Ge
                }
                Some((Token::AndAnd, _)) => {
                    self.advance_token();
                    gala_ast::BinOp::And
                }
                Some((Token::OrOr, _)) => {
                    self.advance_token();
                    gala_ast::BinOp::Or
                }
                Some((Token::DotDot, _)) => {
                    self.advance_token();
                    gala_ast::BinOp::Range
                }
                _ => break,
            };

            let (l_bp, r_bp) = infix_bp(op);
            if l_bp < min_bp {
                break;
            }
            let rhs = self.parse_binary(r_bp)?;
            lhs = gala_ast::Expr::Binary(gala_ast::BinaryExpr {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
                span: Span::dummy(),
            });
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<gala_ast::Expr, ()> {
        if self.match_token(&Token::Minus) {
            let expr = self.parse_unary()?;
            Ok(gala_ast::Expr::Unary(gala_ast::UnaryExpr {
                op: gala_ast::UnOp::Neg,
                expr: Box::new(expr),
                span: Span::dummy(),
            }))
        } else if self.match_token(&Token::Bang) {
            let expr = self.parse_unary()?;
            Ok(gala_ast::Expr::Unary(gala_ast::UnaryExpr {
                op: gala_ast::UnOp::Not,
                expr: Box::new(expr),
                span: Span::dummy(),
            }))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<gala_ast::Expr, ()> {
        if self.match_token(&Token::True) {
            return Ok(gala_ast::Expr::Literal(gala_ast::Literal::Bool(true)));
        }
        if self.match_token(&Token::False) {
            return Ok(gala_ast::Expr::Literal(gala_ast::Literal::Bool(false)));
        }
        if self.check(&Token::Int(0)) {
            if let Some((Token::Int(i), _)) = self.advance_token() {
                return Ok(gala_ast::Expr::Literal(gala_ast::Literal::Int(i)));
            }
        }
        if self.check(&Token::Float(0.0)) {
            if let Some((Token::Float(f), _)) = self.advance_token() {
                return Ok(gala_ast::Expr::Literal(gala_ast::Literal::Float(f)));
            }
        }
        if self.check(&Token::Complex(0.0)) {
            if let Some((Token::Complex(c), _)) = self.advance_token() {
                return Ok(gala_ast::Expr::Literal(gala_ast::Literal::Complex { re: 0.0, im: c }));
            }
        }
        if self.check(&Token::String(String::new())) {
            if let Some((Token::String(s), _)) = self.advance_token() {
                return Ok(gala_ast::Expr::Literal(gala_ast::Literal::String(s)));
            }
        }
        if self.match_token(&Token::LParen) {
            let expr = self.parse_expr()?;
            self.expect(&Token::RParen)?;
            return Ok(expr);
        }
        if self.match_token(&Token::LBrace) {
            return Ok(gala_ast::Expr::Block(self.parse_block()?));
        }
        if self.match_token(&Token::KwIf) {
            let cond = self.parse_expr()?;
            let then_branch = self.parse_block()?;
            let else_branch = if self.match_token(&Token::KwElse) {
                // `else { ... }` is a block; `else if ...` is an expression.
                let branch = if self.check(&Token::LBrace) {
                    gala_ast::Expr::Block(self.parse_block()?)
                } else {
                    self.parse_expr()?
                };
                Some(Box::new(branch))
            } else {
                None
            };
            return Ok(gala_ast::Expr::If(gala_ast::IfExpr {
                cond: Box::new(cond),
                then_branch,
                else_branch,
                span: Span::dummy(),
            }));
        }
        if self.match_token(&Token::KwWhile) {
            let cond = self.parse_expr()?;
            let body = self.parse_block()?;
            return Ok(gala_ast::Expr::While(gala_ast::WhileExpr {
                cond: Box::new(cond),
                body,
                span: Span::dummy(),
            }));
        }
        if self.match_token(&Token::KwFor) {
            let pattern = self.parse_pattern()?;
            self.expect(&Token::KwIn)?;
            let iterable = self.parse_expr()?;
            let body = self.parse_block()?;
            return Ok(gala_ast::Expr::For(gala_ast::ForExpr {
                pattern,
                iterable: Box::new(iterable),
                body,
                span: Span::dummy(),
            }));
        }
        if self.match_token(&Token::Pipe) {
            let params = self.parse_params()?;
            self.expect(&Token::Pipe)?;
            let ret_ty =
                if self.match_token(&Token::Arrow) { Some(self.parse_type()?) } else { None };
            let effect = self.parse_effect()?;
            let body = self.parse_expr()?;
            return Ok(gala_ast::Expr::Lambda(gala_ast::LambdaExpr {
                params,
                ret_ty,
                effect,
                body: Box::new(body),
                span: Span::dummy(),
            }));
        }
        if self.match_token(&Token::KwMatch) {
            return self.parse_match_expr();
        }

        // Identifier / path, including quantum keywords usable as expressions
        // (e.g. `qubit()`, `measure q`, `h(q)`). These are lexed as keywords but
        // act as ordinary identifiers in expression position.
        let ident_name: Option<String> = match self.current_token() {
            Some((Token::Ident(s), _)) => Some(s.clone()),
            Some((Token::KwQubit, _)) => Some("qubit".to_string()),
            Some((Token::KwQubits, _)) => Some("qubits".to_string()),
            Some((Token::KwMeasure, _)) => Some("measure".to_string()),
            Some((Token::KwReverse, _)) => Some("reverse".to_string()),
            Some((Token::KwAdjoint, _)) => Some("adjoint".to_string()),
            Some((Token::KwControl, _)) => Some("control".to_string()),
            Some((Token::KwGrad, _)) => Some("grad".to_string()),
            Some((Token::KwDrop, _)) => Some("drop".to_string()),
            _ => None,
        };
        if let Some(name) = ident_name {
            self.advance_token();
            let ident = Ident::new(name);
            // Check for function call
            if self.check(&Token::LParen) {
                self.advance_token();
                let mut args = Vec::new();
                while !self.check(&Token::RParen) && !self.is_at_end() {
                    args.push(self.parse_expr()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                self.expect(&Token::RParen)?;
                return Ok(gala_ast::Expr::Call(gala_ast::CallExpr {
                    callee: Box::new(gala_ast::Expr::Ident(ident)),
                    args,
                    span: Span::dummy(),
                }));
            }
            return Ok(gala_ast::Expr::Ident(ident));
        }

        self.error("expected expression");
        Err(())
    }

    fn parse_match_expr(&mut self) -> Result<gala_ast::Expr, ()> {
        let scrutinee = self.parse_expr()?;
        self.expect(&Token::LBrace)?;
        let mut arms = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;
            let guard =
                if self.match_token(&Token::KwIf) { Some(self.parse_expr()?) } else { None };
            self.expect(&Token::FatArrow)?;
            let body = self.parse_expr()?;
            arms.push(gala_ast::MatchArm { pattern, guard, body, span: Span::dummy() });
            // Optional trailing comma between arms.
            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        self.expect(&Token::RBrace)?;
        Ok(gala_ast::Expr::Match(gala_ast::MatchExpr {
            scrutinee: Box::new(scrutinee),
            arms,
            span: Span::dummy(),
        }))
    }

    fn parse_type(&mut self) -> Result<gala_ast::Type, ()> {
        if self.match_token(&Token::TyQubit) {
            return Ok(gala_ast::Type::Qubit);
        }
        if self.match_token(&Token::TyQubits) {
            self.expect(&Token::LAngle)?;
            let size = self.parse_const_expr()?;
            self.expect(&Token::RAngle)?;
            return Ok(gala_ast::Type::Qubits(Box::new(size)));
        }
        if self.match_token(&Token::TyMeasured) {
            self.expect(&Token::LAngle)?;
            let ty = self.parse_type()?;
            self.expect(&Token::RAngle)?;
            return Ok(gala_ast::Type::Measured(Box::new(ty)));
        }
        if self.match_token(&Token::TyBool) {
            return Ok(gala_ast::Type::Path(gala_ast::Path {
                segments: vec![gala_ast::PathSegment {
                    ident: Ident::new("Bool"),
                    type_args: Vec::new(),
                }],
                span: Span::dummy(),
            }));
        }
        if self.match_token(&Token::TyInt) {
            return Ok(gala_ast::Type::Path(gala_ast::Path {
                segments: vec![gala_ast::PathSegment {
                    ident: Ident::new("Int"),
                    type_args: Vec::new(),
                }],
                span: Span::dummy(),
            }));
        }
        if self.match_token(&Token::TyFloat) {
            return Ok(gala_ast::Type::Path(gala_ast::Path {
                segments: vec![gala_ast::PathSegment {
                    ident: Ident::new("Float"),
                    type_args: Vec::new(),
                }],
                span: Span::dummy(),
            }));
        }
        if self.match_token(&Token::TyComplex) {
            return Ok(gala_ast::Type::Path(gala_ast::Path {
                segments: vec![gala_ast::PathSegment {
                    ident: Ident::new("Complex"),
                    type_args: Vec::new(),
                }],
                span: Span::dummy(),
            }));
        }
        if self.match_token(&Token::TyString) {
            return Ok(gala_ast::Type::Path(gala_ast::Path {
                segments: vec![gala_ast::PathSegment {
                    ident: Ident::new("String"),
                    type_args: Vec::new(),
                }],
                span: Span::dummy(),
            }));
        }
        if self.match_token(&Token::TyUnit) {
            return Ok(gala_ast::Type::Path(gala_ast::Path {
                segments: vec![gala_ast::PathSegment {
                    ident: Ident::new("Unit"),
                    type_args: Vec::new(),
                }],
                span: Span::dummy(),
            }));
        }
        if self.match_token(&Token::TyParams) {
            return Ok(gala_ast::Type::Path(gala_ast::Path {
                segments: vec![gala_ast::PathSegment {
                    ident: Ident::new("Params"),
                    type_args: Vec::new(),
                }],
                span: Span::dummy(),
            }));
        }
        if self.match_token(&Token::TyMeasured) {
            return Ok(gala_ast::Type::Path(gala_ast::Path {
                segments: vec![gala_ast::PathSegment {
                    ident: Ident::new("Measured"),
                    type_args: Vec::new(),
                }],
                span: Span::dummy(),
            }));
        }
        if self.match_token(&Token::TyQubit) {
            return Ok(gala_ast::Type::Path(gala_ast::Path {
                segments: vec![gala_ast::PathSegment {
                    ident: Ident::new("Qubit"),
                    type_args: Vec::new(),
                }],
                span: Span::dummy(),
            }));
        }
        if self.match_token(&Token::TyQubits) {
            return Ok(gala_ast::Type::Path(gala_ast::Path {
                segments: vec![gala_ast::PathSegment {
                    ident: Ident::new("Qubits"),
                    type_args: Vec::new(),
                }],
                span: Span::dummy(),
            }));
        }
        if self.match_token(&Token::TyVec) {
            return Ok(gala_ast::Type::Path(gala_ast::Path {
                segments: vec![gala_ast::PathSegment {
                    ident: Ident::new("Vec"),
                    type_args: Vec::new(),
                }],
                span: Span::dummy(),
            }));
        }

        if let Some((Token::Ident(s), _)) = self.current_token() {
            let s = s.clone();
            self.advance_token();
            let ident = Ident::new(s);
            let path = gala_ast::Path {
                segments: vec![gala_ast::PathSegment { ident, type_args: Vec::new() }],
                span: Span::dummy(),
            };
            // Check for generic args
            if self.check(&Token::LAngle) {
                self.advance_token();
                let mut args = Vec::new();
                while !self.check(&Token::RAngle) && !self.is_at_end() {
                    if let Some((Token::Int(i), _)) = self.current_token() {
                        let i = *i;
                        self.advance_token();
                        args.push(gala_ast::TypeArg::Const(gala_ast::ConstExpr::Int(i)));
                    } else {
                        args.push(gala_ast::TypeArg::Type(self.parse_type()?));
                    }
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                self.expect(&Token::RAngle)?;
                return Ok(gala_ast::Type::Named(
                    path.segments[0].ident.as_str().to_string(),
                    args.iter()
                        .map(|a| match a {
                            gala_ast::TypeArg::Type(t) => t.clone(),
                            gala_ast::TypeArg::Const(_) => gala_ast::Type::Path(gala_ast::Path {
                                segments: vec![gala_ast::PathSegment {
                                    ident: Ident::new("Int"),
                                    type_args: Vec::new(),
                                }],
                                span: Span::dummy(),
                            }),
                        })
                        .collect(),
                ));
            } else {
                return Ok(gala_ast::Type::Path(self.parse_path()?));
            }
        }

        if self.match_token(&Token::LParen) {
            let mut tys = Vec::new();
            while !self.check(&Token::RParen) && !self.is_at_end() {
                tys.push(self.parse_type()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
            self.expect(&Token::RParen)?;
            if tys.len() == 1 {
                Ok(tys.into_iter().next().unwrap())
            } else {
                Ok(gala_ast::Type::Tuple(tys))
            }
        } else {
            self.error("expected type");
            Err(())
        }
    }

    fn parse_const_expr(&mut self) -> Result<gala_ast::ConstExpr, ()> {
        if let Some((Token::Int(i), _)) = self.advance_token() {
            Ok(gala_ast::ConstExpr::Int(i))
        } else if let Some((Token::Ident(s), _)) = self.advance_token() {
            Ok(gala_ast::ConstExpr::Ident(Ident::new(s)))
        } else {
            self.error("expected const expression");
            Err(())
        }
    }

    fn parse_path(&mut self) -> Result<gala_ast::Path, ()> {
        let ident = self.parse_ident()?;
        let mut segments = vec![gala_ast::PathSegment { ident, type_args: Vec::new() }];
        while self.match_token(&Token::Dot) {
            let ident = self.parse_ident()?;
            segments.push(gala_ast::PathSegment { ident, type_args: Vec::new() });
        }
        Ok(gala_ast::Path { segments, span: Span::dummy() })
    }

    fn parse_ident(&mut self) -> Result<Ident, ()> {
        if let Some((Token::Ident(s), _)) = self.advance_token() {
            Ok(Ident::new(s))
        } else {
            self.error("expected identifier");
            Err(())
        }
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ()> {
        if self.match_token(&Token::Underscore) {
            Ok(Pattern::Wildcard)
        } else if let Some((Token::Int(i), _)) = self.current_token().cloned() {
            self.advance_token();
            Ok(Pattern::Literal(gala_ast::Literal::Int(i)))
        } else if let Some((Token::Float(f), _)) = self.current_token().cloned() {
            self.advance_token();
            Ok(Pattern::Literal(gala_ast::Literal::Float(f)))
        } else if let Some((Token::String(s), _)) = self.current_token().cloned() {
            self.advance_token();
            Ok(Pattern::Literal(gala_ast::Literal::String(s)))
        } else if self.match_token(&Token::True) {
            Ok(Pattern::Literal(gala_ast::Literal::Bool(true)))
        } else if self.match_token(&Token::False) {
            Ok(Pattern::Literal(gala_ast::Literal::Bool(false)))
        } else {
            let ident = self.parse_ident()?;
            Ok(Pattern::Ident(ident))
        }
    }

    fn parse_struct_def(&mut self) -> Result<gala_ast::Item, ()> {
        let ident = self.parse_ident()?;
        let generics = self.parse_generics()?;
        self.expect(&Token::LBrace)?;
        let mut fields = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let ident = self.parse_ident()?;
            self.expect(&Token::Colon)?;
            let ty = self.parse_type()?;
            fields.push(gala_ast::StructField { ident, ty, span: Span::dummy() });
            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        self.expect(&Token::RBrace)?;
        Ok(gala_ast::Item::StructDef(gala_ast::StructDef {
            ident,
            generics,
            fields,
            span: Span::dummy(),
        }))
    }

    fn parse_enum_def(&mut self) -> Result<gala_ast::Item, ()> {
        let ident = self.parse_ident()?;
        let generics = self.parse_generics()?;
        self.expect(&Token::LBrace)?;
        let mut variants = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let ident = self.parse_ident()?;
            let fields = if self.match_token(&Token::LParen) {
                let mut tys = Vec::new();
                while !self.check(&Token::RParen) && !self.is_at_end() {
                    tys.push(self.parse_type()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                self.expect(&Token::RParen)?;
                tys
            } else {
                Vec::new()
            };
            variants.push(gala_ast::EnumVariant { ident, fields, span: Span::dummy() });
            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        self.expect(&Token::RBrace)?;
        Ok(gala_ast::Item::EnumDef(gala_ast::EnumDef {
            ident,
            generics,
            variants,
            span: Span::dummy(),
        }))
    }

    fn parse_trait_def(&mut self) -> Result<gala_ast::Item, ()> {
        let ident = self.parse_ident()?;
        let generics = self.parse_generics()?;
        self.expect(&Token::LBrace)?;
        let mut items = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            if self.match_token(&Token::KwFn) {
                let ident = self.parse_ident()?;
                let generics = self.parse_generics()?;
                self.expect(&Token::LParen)?;
                let params = self.parse_params()?;
                self.expect(&Token::RParen)?;
                let ret_ty =
                    if self.match_token(&Token::Arrow) { Some(self.parse_type()?) } else { None };
                let effect = self.parse_effect()?;
                self.expect(&Token::Semicolon)?;
                items.push(gala_ast::TraitItem::Fn(gala_ast::FnSig {
                    ident,
                    generics,
                    params,
                    ret_ty,
                    effect,
                    span: Span::dummy(),
                }));
            } else {
                self.advance_token();
            }
        }
        self.expect(&Token::RBrace)?;
        Ok(gala_ast::Item::TraitDef(gala_ast::TraitDef {
            ident,
            generics,
            items,
            span: Span::dummy(),
        }))
    }

    fn parse_impl_block(&mut self) -> Result<gala_ast::Item, ()> {
        let ty = self.parse_type()?;
        let effect = self.parse_effect()?;
        self.expect(&Token::LBrace)?;
        let mut items = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            if self.match_token(&Token::KwFn) {
                items.push(self.parse_fn_def()?);
            } else {
                self.advance_token();
            }
        }
        self.expect(&Token::RBrace)?;
        Ok(gala_ast::Item::ImplBlock(gala_ast::ImplBlock {
            ty,
            effect,
            items,
            span: Span::dummy(),
        }))
    }

    fn parse_type_alias(&mut self) -> Result<gala_ast::Item, ()> {
        let ident = self.parse_ident()?;
        let generics = self.parse_generics()?;
        self.expect(&Token::Eq)?;
        let ty = self.parse_type()?;
        self.expect(&Token::Semicolon)?;
        Ok(gala_ast::Item::TypeAlias(gala_ast::TypeAlias {
            ident,
            generics,
            ty,
            span: Span::dummy(),
        }))
    }

    fn parse_const_def(&mut self) -> Result<gala_ast::Item, ()> {
        let ident = self.parse_ident()?;
        self.expect(&Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(&Token::Eq)?;
        let value = self.parse_expr()?;
        self.expect(&Token::Semicolon)?;
        Ok(gala_ast::Item::ConstDef(gala_ast::ConstDef { ident, ty, value, span: Span::dummy() }))
    }

    fn parse_import(&mut self) -> Result<gala_ast::Item, ()> {
        let path = self.parse_path()?;

        // Check for glob import: .*
        let mut items = None;
        let mut glob = false;
        if self.match_token(&Token::Dot) {
            if self.match_token(&Token::Star) {
                glob = true;
                self.expect(&Token::Semicolon)?;
                return Ok(gala_ast::Item::Import(gala_ast::Import {
                    path,
                    alias: None,
                    items,
                    glob,
                    span: Span::dummy(),
                }));
            }

            // Check for braced list: .{ ... }
            if self.match_token(&Token::LBrace) {
                let mut idents = Vec::new();

                // Parse first identifier (if any)
                if !self.check(&Token::RBrace) {
                    loop {
                        let ident = self.parse_ident()?;
                        idents.push(ident);

                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }

                self.expect(&Token::RBrace)?;
                items = Some(idents);
            }
        }

        // Check for alias: as ident
        let alias = if self.match_token(&Token::KwAs) { Some(self.parse_ident()?) } else { None };

        self.expect(&Token::Semicolon)?;
        Ok(gala_ast::Item::Import(gala_ast::Import {
            path,
            alias,
            items,
            glob,
            span: Span::dummy(),
        }))
    }

    fn current_span(&self) -> Span {
        if let Some((_, span)) = self.current_token() {
            *span
        } else {
            Span::dummy()
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || matches!(self.current_token(), Some((Token::Eof, _)))
    }

    fn current_token(&self) -> Option<&(Token, Span)> {
        self.tokens.get(self.pos)
    }

    fn advance_token(&mut self) -> Option<(Token, Span)> {
        let tok = self.current_token().cloned();
        self.pos += 1;
        tok
    }

    fn check(&self, expected: &Token) -> bool {
        if let Some((tok, _)) = self.current_token() {
            std::mem::discriminant(tok) == std::mem::discriminant(expected)
        } else {
            false
        }
    }

    fn match_token(&mut self, expected: &Token) -> bool {
        if let Some((tok, _)) = self.current_token() {
            if std::mem::discriminant(tok) == std::mem::discriminant(expected) {
                self.pos += 1;
                return true;
            }
        }
        false
    }

    fn expect(&mut self, expected: &Token) -> Result<(), ()> {
        if self.match_token(expected) {
            Ok(())
        } else {
            self.error(&format!("expected {:?}", expected));
            Err(())
        }
    }

    fn error(&mut self, msg: &str) {
        let span = self.current_span();
        self.diags.push(
            Diagnostic::error(codes::EXPECTED_EXPRESSION, msg).with_primary_label(span, "here"),
        );
    }

    fn synchronize(&mut self) {
        while !self.is_at_end() {
            if self.match_token(&Token::Semicolon) {
                return;
            }
            self.advance_token();
        }
    }
}

fn infix_bp(op: gala_ast::BinOp) -> (u8, u8) {
    match op {
        gala_ast::BinOp::Or => (1, 2),
        gala_ast::BinOp::And => (3, 4),
        gala_ast::BinOp::Eq | gala_ast::BinOp::Ne => (5, 6),
        gala_ast::BinOp::Lt | gala_ast::BinOp::Le | gala_ast::BinOp::Gt | gala_ast::BinOp::Ge => {
            (7, 8)
        }
        gala_ast::BinOp::Add | gala_ast::BinOp::Sub => (9, 10),
        gala_ast::BinOp::Mul | gala_ast::BinOp::Div | gala_ast::BinOp::Mod => (11, 12),
        gala_ast::BinOp::Range => (13, 14),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gala_span::SourceMap;

    #[test]
    fn test_parse_simple() {
        let source = "fn main() -> Int { return 42; }";
        let mut map = SourceMap::new();
        let fid = map.add_file("test.gala".into(), source.to_string());
        let result = parse_file(fid, source, &mut map);
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_parse_empty_source() {
        let source = "";
        let mut map = SourceMap::new();
        let fid = map.add_file("test.gala".into(), source.to_string());
        let result = parse_file(fid, source, &mut map);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_unary_expr() {
        let source = "fn main() -> Int { return -42; }";
        let mut map = SourceMap::new();
        let fid = map.add_file("test.gala".into(), source.to_string());
        let result = parse_file(fid, source, &mut map);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_syntax_error() {
        let source = "fn main() -> Int { return }";
        let mut map = SourceMap::new();
        let fid = map.add_file("test.gala".into(), source.to_string());
        let result = parse_file(fid, source, &mut map);
        if let Err(diags) = result {
            assert!(diags.has_errors());
        }
    }
}
