//! Recursive-descent parser for Gala with error recovery.
//!
//! Statements end at `;`, at a newline, or at the closing `}` of their block. A syntax error is
//! reported as an `E01xx` diagnostic and parsing resumes at the next statement or item, so
//! [`parse_source`] always returns the items it could build alongside the diagnostics.

use gala_ast::*;
use gala_diagnostics::{codes, Diagnostic, DiagnosticCode, Diagnostics};
use gala_lexer::{Lexer, Token};
use gala_span::{ByteSpan, FileId, SourceMap, Span};

/// Result of parsing one file: whatever items could be recovered, plus all diagnostics.
#[derive(Debug, Clone)]
pub struct ParseOutput {
    /// Items that parsed (possibly partial when there were errors).
    pub items: Vec<Item>,
    /// Syntax diagnostics. Empty when the file is well formed.
    pub diagnostics: Diagnostics,
}

/// Parse a source file, returning items and diagnostics together.
pub fn parse_source(file_id: FileId, source: &str) -> ParseOutput {
    let tokens = Lexer::new(file_id, source).collect_all();
    let mut parser = Parser::new(file_id, source, tokens);
    let items = parser.parse_items();
    ParseOutput { items, diagnostics: parser.diags }
}

/// Parse a source file. Returns `Err` with the diagnostics if the file has any syntax error.
pub fn parse_file(
    file_id: FileId,
    source: &str,
    _source_map: &mut SourceMap,
) -> Result<Vec<Item>, Diagnostics> {
    let out = parse_source(file_id, source);
    if out.diagnostics.has_errors() {
        Err(out.diagnostics)
    } else {
        Ok(out.items)
    }
}

/// Marker for "a diagnostic has been recorded; unwind to the recovery point".
type PResult<T> = Result<T, ()>;

struct Parser<'a> {
    file: FileId,
    src: &'a str,
    toks: Vec<(Token, Span)>,
    pos: usize,
    diags: Diagnostics,
}

impl<'a> Parser<'a> {
    fn new(file: FileId, src: &'a str, toks: Vec<(Token, Span)>) -> Self {
        Parser { file, src, toks, pos: 0, diags: Diagnostics::new() }
    }

    // ───────────────────────── token access ─────────────────────────

    fn peek(&self) -> &Token {
        self.toks.get(self.pos).map(|(t, _)| t).unwrap_or(&Token::Eof)
    }

    fn peek_at(&self, n: usize) -> &Token {
        self.toks.get(self.pos + n).map(|(t, _)| t).unwrap_or(&Token::Eof)
    }

    fn span_here(&self) -> Span {
        match self.toks.get(self.pos) {
            Some((_, s)) => *s,
            None => self.eof_span(),
        }
    }

    fn eof_span(&self) -> Span {
        let end = self.src.len() as u32;
        Span::new(self.file, ByteSpan::new(end, end))
    }

    fn prev_end(&self) -> u32 {
        if self.pos == 0 {
            0
        } else {
            self.toks.get(self.pos - 1).map(|(_, s)| s.byte_span.end).unwrap_or(0)
        }
    }

    /// True when the source between the previous token and the current one contains a newline.
    fn newline_before(&self) -> bool {
        if self.pos == 0 {
            return false;
        }
        let start = self.prev_end() as usize;
        let end = self.span_here().byte_span.start as usize;
        match self.src.get(start..end) {
            Some(gap) => gap.contains('\n'),
            None => false,
        }
    }

    fn bump(&mut self) -> (Token, Span) {
        let out = match self.toks.get(self.pos) {
            Some((t, s)) => (t.clone(), *s),
            None => (Token::Eof, self.eof_span()),
        };
        if self.pos < self.toks.len() {
            self.pos += 1;
        }
        out
    }

    fn check(&self, t: &Token) -> bool {
        self.peek() == t
    }

    fn eat(&mut self, t: &Token) -> bool {
        if self.check(t) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek(), Token::Eof) || self.pos >= self.toks.len()
    }

    fn join(a: Span, b: Span) -> Span {
        Span::new(
            a.file_id,
            ByteSpan::new(
                a.byte_span.start.min(b.byte_span.start),
                a.byte_span.end.max(b.byte_span.end),
            ),
        )
    }

    // ───────────────────────── diagnostics ─────────────────────────

    fn fail<T>(&mut self, code: DiagnosticCode, msg: impl Into<String>, span: Span) -> PResult<T> {
        self.diags.push(Diagnostic::error(code, msg).with_primary_label(span, "here"));
        Err(())
    }

    fn expected<T>(&mut self, what: &str) -> PResult<T> {
        let span = self.span_here();
        if let Token::Invalid(c) = self.peek() {
            let c = *c;
            self.bump();
            return self.fail(
                codes::INVALID_CHARACTER,
                format!("invalid character `{c}` in source"),
                span,
            );
        }
        if self.at_eof() {
            return self.fail(
                codes::UNEXPECTED_EOF,
                format!("expected {what}, found end of file"),
                span,
            );
        }
        let found = describe(self.peek());
        let msg = format!("expected {what}, found {found}");
        self.fail(codes::UNEXPECTED_TOKEN, msg, span)
    }

    fn expect(&mut self, t: &Token, what: &str) -> PResult<Span> {
        if self.check(t) {
            Ok(self.bump().1)
        } else {
            self.expected(what)
        }
    }

    // ───────────────────────── items ─────────────────────────

    fn parse_items(&mut self) -> Vec<Item> {
        let mut items = Vec::new();
        while !self.at_eof() {
            let start = self.pos;
            match self.parse_item() {
                Ok(Some(item)) => items.push(item),
                Ok(None) => {}
                Err(()) => {
                    self.recover_item();
                    // Guarantee progress even when the failing token is itself an item keyword.
                    if self.pos == start && !self.at_eof() {
                        self.bump();
                    }
                }
            }
        }
        items
    }

    fn parse_item(&mut self) -> PResult<Option<Item>> {
        let attributes = self.parse_attributes()?;
        match self.peek().clone() {
            Token::KwFn => Ok(Some(Item::FnDef(self.parse_fn(attributes)?))),
            Token::KwImport => {
                let imp = self.parse_import()?;
                self.end_stmt()?;
                Ok(Some(Item::Import(imp)))
            }
            Token::KwLet => {
                let start = self.bump().1;
                let (name, _) = self.take_ident()?;
                let ty = if self.eat(&Token::Colon) { Some(self.parse_type()?) } else { None };
                self.expect(&Token::Eq, "`=`")?;
                let value = self.parse_expr()?;
                let span = Self::join(start, value.span);
                self.end_stmt()?;
                Ok(Some(Item::ConstDef(ConstDef { ident: name, ty, value, span })))
            }
            Token::KwConst => {
                let start = self.bump().1;
                let (name, _) = self.take_ident()?;
                self.expect(&Token::Colon, "`:`")?;
                let ty = self.parse_type()?;
                self.expect(&Token::Eq, "`=`")?;
                let value = self.parse_expr()?;
                let span = Self::join(start, value.span);
                self.end_stmt()?;
                Ok(Some(Item::ConstDef(ConstDef { ident: name, ty: Some(ty), value, span })))
            }
            Token::KwStruct | Token::KwEnum | Token::KwTrait | Token::KwImpl | Token::KwType => {
                let span = self.span_here();
                self.fail(
                    codes::UNEXPECTED_TOKEN,
                    "this item kind is not supported by the compiler yet; only `fn`, `import`, \
                     and top-level `let`/`const` are",
                    span,
                )
            }
            _ => {
                if !attributes.is_empty() {
                    let span = self.span_here();
                    return self.fail(
                        codes::UNEXPECTED_TOKEN,
                        "attributes must precede a `fn`",
                        span,
                    );
                }
                self.expected("an item (`fn`, `import`, `let`, or `const`)")
            }
        }
    }

    fn parse_attributes(&mut self) -> PResult<Vec<Attribute>> {
        let mut attrs = Vec::new();
        while self.check(&Token::Hash) {
            let start = self.bump().1;
            self.expect(&Token::LBracket, "`[`")?;
            let (name, _) = self.take_ident()?;
            let mut args = Vec::new();
            if self.eat(&Token::LParen) {
                while !self.check(&Token::RParen) {
                    let (arg, _) = self.take_ident()?;
                    args.push(arg);
                    if !self.eat(&Token::Comma) {
                        break;
                    }
                }
                self.expect(&Token::RParen, "`)`")?;
            }
            let end = self.expect(&Token::RBracket, "`]`")?;
            attrs.push(Attribute { name, args, span: Self::join(start, end) });
        }
        Ok(attrs)
    }

    fn parse_fn(&mut self, attributes: Vec<Attribute>) -> PResult<FnDef> {
        let start = self.expect(&Token::KwFn, "`fn`")?;
        let (ident, _) = self.take_ident()?;
        let generics = if self.check(&Token::Lt) { self.parse_generics()? } else { Vec::new() };
        self.expect(&Token::LParen, "`(`")?;
        let mut params = Vec::new();
        while !self.check(&Token::RParen) {
            params.push(self.parse_param()?);
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        self.expect(&Token::RParen, "`)`")?;
        let ret_ty = if self.eat(&Token::Arrow) { Some(self.parse_type()?) } else { None };
        let effect = self.parse_effect();
        let (body, span) = if self.check(&Token::Semicolon) {
            // Signature-only declaration: an intrinsic provided by the compiler.
            let end = self.bump().1;
            (None, Self::join(start, end))
        } else {
            let body = self.parse_block()?;
            let span = Self::join(start, body.span);
            (Some(body), span)
        };
        Ok(FnDef { attributes, ident, generics, params, ret_ty, effect, body, span })
    }

    fn parse_generics(&mut self) -> PResult<Vec<GenericParam>> {
        self.expect(&Token::Lt, "`<`")?;
        let mut out = Vec::new();
        while !self.check(&Token::Gt) {
            if self.eat(&Token::KwConst) {
                let (name, _) = self.take_ident()?;
                self.expect(&Token::Colon, "`:`")?;
                let ty = self.parse_type()?;
                out.push(GenericParam::Const { ident: name, ty });
            } else {
                let (name, _) = self.take_ident()?;
                let bound = if self.eat(&Token::Colon) { Some(self.parse_type()?) } else { None };
                out.push(GenericParam::Type { ident: name, bound });
            }
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        self.expect(&Token::Gt, "`>`")?;
        Ok(out)
    }

    fn parse_param(&mut self) -> PResult<Param> {
        let start = self.span_here();
        let mutable = self.eat(&Token::KwMut);
        let pattern = self.parse_pattern()?;
        self.expect(&Token::Colon, "`:`")?;
        let ty = self.parse_type()?;
        let span = Self::join(start, self.prev_span());
        Ok(Param { mutable, pattern, ty, span })
    }

    fn parse_effect(&mut self) -> Option<Effect> {
        match self.peek() {
            Token::KwPure => {
                self.bump();
                Some(Effect::Pure)
            }
            Token::KwQuantum => {
                self.bump();
                Some(Effect::Quantum)
            }
            Token::KwProb => {
                self.bump();
                Some(Effect::Prob)
            }
            _ => None,
        }
    }

    fn parse_import(&mut self) -> PResult<Import> {
        let start = self.expect(&Token::KwImport, "`import`")?;
        let mut segments = Vec::new();
        let (first, _) = self.take_ident()?;
        segments.push(PathSegment { ident: first, type_args: Vec::new() });
        let mut items = None;
        let mut glob = false;
        while self.check(&Token::Dot) {
            self.bump();
            match self.peek().clone() {
                Token::LBrace => {
                    self.bump();
                    let mut names = Vec::new();
                    while !self.check(&Token::RBrace) {
                        let (n, _) = self.take_ident()?;
                        names.push(n);
                        if !self.eat(&Token::Comma) {
                            break;
                        }
                    }
                    self.expect(&Token::RBrace, "`}`")?;
                    items = Some(names);
                    break;
                }
                Token::Star => {
                    self.bump();
                    glob = true;
                    break;
                }
                _ => {
                    let (seg, _) = self.take_ident()?;
                    segments.push(PathSegment { ident: seg, type_args: Vec::new() });
                }
            }
        }
        let alias = if self.eat(&Token::KwAs) { Some(self.take_ident()?.0) } else { None };
        let span = Self::join(start, self.prev_span());
        let path = Path { segments, span };
        Ok(Import { path, alias, items, glob, span })
    }

    /// Skip to a plausible item start after an error.
    fn recover_item(&mut self) {
        while !self.at_eof() {
            match self.peek() {
                Token::KwFn | Token::KwImport | Token::KwLet | Token::KwConst | Token::Hash => {
                    break
                }
                _ => {
                    self.bump();
                }
            }
        }
    }

    // ───────────────────────── blocks and statements ─────────────────────────

    fn parse_block(&mut self) -> PResult<Block> {
        let start = self.expect(&Token::LBrace, "`{`")?;
        let mut stmts = Vec::new();
        let mut tail: Option<Box<Expr>> = None;
        loop {
            if self.check(&Token::RBrace) {
                break;
            }
            if self.at_eof() {
                let span = self.eof_span();
                self.diags.push(
                    Diagnostic::error(codes::UNEXPECTED_EOF, "unclosed block: expected `}`")
                        .with_primary_label(span, "the file ends here")
                        .with_help("add a closing `}`"),
                );
                return Err(());
            }
            match self.parse_stmt() {
                Ok((stmt, ended_by_semi)) => {
                    if let Stmt::Expr(e) = stmt {
                        if !ended_by_semi && self.check(&Token::RBrace) {
                            tail = Some(Box::new(e));
                        } else {
                            stmts.push(Stmt::Expr(e));
                        }
                    } else {
                        stmts.push(stmt);
                    }
                }
                Err(()) => self.recover_stmt(),
            }
        }
        let end = self.expect(&Token::RBrace, "`}`")?;
        Ok(Block { stmts, tail, span: Self::join(start, end) })
    }

    /// Skip to the end of a broken statement.
    fn recover_stmt(&mut self) {
        let start = self.pos;
        loop {
            if self.at_eof() || self.check(&Token::RBrace) {
                break;
            }
            if self.check(&Token::Semicolon) {
                self.bump();
                break;
            }
            if self.pos > start && self.newline_before() {
                break;
            }
            self.bump();
        }
    }

    /// Returns the statement and whether it was terminated by an explicit `;`.
    fn parse_stmt(&mut self) -> PResult<(Stmt, bool)> {
        match self.peek() {
            Token::KwLet => {
                let let_stmt = self.parse_let()?;
                self.end_stmt()?;
                Ok((Stmt::Let(let_stmt), true))
            }
            Token::KwReturn => {
                self.bump();
                let value = if self.at_stmt_end() { None } else { Some(self.parse_expr()?) };
                self.end_stmt()?;
                Ok((Stmt::Return(value), true))
            }
            _ => {
                let expr = self.parse_expr()?;
                let block_like = matches!(
                    expr.kind,
                    ExprKind::If(_) | ExprKind::For(_) | ExprKind::While(_) | ExprKind::Block(_)
                );
                if self.eat(&Token::Semicolon) {
                    return Ok((Stmt::Expr(expr), true));
                }
                if block_like || self.check(&Token::RBrace) || self.at_eof() {
                    return Ok((Stmt::Expr(expr), block_like));
                }
                if self.newline_before() {
                    return Ok((Stmt::Expr(expr), false));
                }
                self.expected("`;` or a newline after the expression")
            }
        }
    }

    fn at_stmt_end(&self) -> bool {
        self.check(&Token::Semicolon)
            || self.check(&Token::RBrace)
            || self.at_eof()
            || self.newline_before()
    }

    fn end_stmt(&mut self) -> PResult<()> {
        if self.eat(&Token::Semicolon) || self.check(&Token::RBrace) || self.at_eof() {
            return Ok(());
        }
        if self.newline_before() {
            return Ok(());
        }
        self.expected("`;` or a newline")
    }

    fn parse_let(&mut self) -> PResult<LetStmt> {
        let start = self.expect(&Token::KwLet, "`let`")?;
        let mutable = self.eat(&Token::KwMut);
        let pattern = self.parse_pattern()?;
        let ty = if self.eat(&Token::Colon) { Some(self.parse_type()?) } else { None };
        self.expect(&Token::Eq, "`=`")?;
        let init = self.parse_expr()?;
        let span = Self::join(start, init.span);
        Ok(LetStmt { mutable, pattern, ty, init: Some(init), span })
    }

    fn prev_span(&self) -> Span {
        match self.pos.checked_sub(1).and_then(|i| self.toks.get(i)) {
            Some((_, s)) => *s,
            None => self.eof_span(),
        }
    }

    // ───────────────────────── expressions ─────────────────────────

    fn parse_expr(&mut self) -> PResult<Expr> {
        let lhs = self.parse_binary(1)?;
        let op = match self.peek() {
            Token::Eq => Some(None),
            Token::PlusEq => Some(Some(BinOp::Add)),
            Token::MinusEq => Some(Some(BinOp::Sub)),
            Token::StarEq => Some(Some(BinOp::Mul)),
            Token::SlashEq => Some(Some(BinOp::Div)),
            _ => None,
        };
        if let Some(op) = op {
            self.bump();
            let value = self.parse_expr()?;
            let span = Self::join(lhs.span, value.span);
            return Ok(Expr::new(
                ExprKind::Assign(AssignExpr { target: Box::new(lhs), op, value: Box::new(value) }),
                span,
            ));
        }
        Ok(lhs)
    }

    fn binary_info(tok: &Token) -> Option<(u8, BinOp)> {
        Some(match tok {
            Token::DotDot => (1, BinOp::Range),
            Token::OrOr => (2, BinOp::Or),
            Token::AndAnd => (3, BinOp::And),
            Token::EqEq => (4, BinOp::Eq),
            Token::BangEq => (4, BinOp::Ne),
            Token::Lt => (5, BinOp::Lt),
            Token::Le => (5, BinOp::Le),
            Token::Gt => (5, BinOp::Gt),
            Token::Ge => (5, BinOp::Ge),
            Token::Plus => (6, BinOp::Add),
            Token::Minus => (6, BinOp::Sub),
            Token::Star => (7, BinOp::Mul),
            Token::Slash => (7, BinOp::Div),
            Token::Percent => (7, BinOp::Mod),
            _ => return None,
        })
    }

    fn parse_binary(&mut self, min_prec: u8) -> PResult<Expr> {
        let mut lhs = self.parse_unary()?;
        while let Some((prec, op)) = Self::binary_info(self.peek()) {
            if prec < min_prec {
                break;
            }
            self.bump();
            let rhs = self.parse_binary(prec + 1)?;
            let span = Self::join(lhs.span, rhs.span);
            lhs = Expr::new(
                ExprKind::Binary(BinaryExpr { lhs: Box::new(lhs), op, rhs: Box::new(rhs), span }),
                span,
            );
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> PResult<Expr> {
        let op = match self.peek() {
            Token::Minus => Some(UnOp::Neg),
            Token::Bang => Some(UnOp::Not),
            _ => None,
        };
        if let Some(op) = op {
            let start = self.bump().1;
            let expr = self.parse_unary()?;
            let span = Self::join(start, expr.span);
            return Ok(Expr::new(
                ExprKind::Unary(UnaryExpr { op, expr: Box::new(expr), span }),
                span,
            ));
        }
        let primary = self.parse_primary()?;
        self.parse_postfix(primary)
    }

    fn parse_postfix(&mut self, mut expr: Expr) -> PResult<Expr> {
        loop {
            match self.peek() {
                Token::LParen if !self.newline_before() => {
                    let (args, names, end) = self.parse_call_args()?;
                    let span = Self::join(expr.span, end);
                    expr = Expr::new(
                        ExprKind::Call(CallExpr {
                            callee: Box::new(expr),
                            generics: Vec::new(),
                            args,
                            names,
                            span,
                        }),
                        span,
                    );
                }
                Token::LBracket if !self.newline_before() => {
                    self.bump();
                    let index = self.parse_expr()?;
                    let end = self.expect(&Token::RBracket, "`]`")?;
                    let span = Self::join(expr.span, end);
                    expr = Expr::new(
                        ExprKind::Index(IndexExpr {
                            base: Box::new(expr),
                            index: Box::new(index),
                            span,
                        }),
                        span,
                    );
                }
                Token::Dot => {
                    self.bump();
                    let (field, fspan) = self.take_ident()?;
                    if self.check(&Token::LParen) {
                        let (args, _names, end) = self.parse_call_args()?;
                        let span = Self::join(expr.span, end);
                        expr = Expr::new(
                            ExprKind::MethodCall(MethodCallExpr {
                                receiver: Box::new(expr),
                                method: field,
                                args,
                                span,
                            }),
                            span,
                        );
                    } else {
                        let span = Self::join(expr.span, fspan);
                        expr = Expr::new(
                            ExprKind::Field(FieldExpr { base: Box::new(expr), field, span }),
                            span,
                        );
                    }
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    /// Parses `( args )`, returning arguments, their optional names, and the closing span.
    fn parse_call_args(&mut self) -> PResult<(Vec<Expr>, Vec<Option<Ident>>, Span)> {
        self.expect(&Token::LParen, "`(`")?;
        let mut args = Vec::new();
        let mut names = Vec::new();
        while !self.check(&Token::RParen) {
            let named = matches!(self.peek(), Token::Ident(_)) && self.peek_at(1) == &Token::Colon;
            if named {
                let (name, _) = self.take_ident()?;
                self.bump(); // colon
                names.push(Some(name));
            } else {
                names.push(None);
            }
            args.push(self.parse_expr()?);
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        let end = self.expect(&Token::RParen, "`)`")?;
        Ok((args, names, end))
    }

    /// True when the tokens ahead look like `< <int or ident> >` followed by `(`, which is an
    /// explicit-argument call such as `qubits<4>()`.
    fn at_generic_call(&self) -> bool {
        matches!(self.peek(), Token::Lt)
            && matches!(self.peek_at(1), Token::Int(_) | Token::Ident(_))
            && self.peek_at(2) == &Token::Gt
            && self.peek_at(3) == &Token::LParen
    }

    fn parse_primary(&mut self) -> PResult<Expr> {
        let (tok, span) = self.toks.get(self.pos).cloned().unwrap_or((Token::Eof, self.eof_span()));
        match tok {
            Token::Int(v) => {
                self.bump();
                Ok(Expr::new(ExprKind::Literal(Literal::Int(v)), span))
            }
            Token::Float(v) => {
                self.bump();
                Ok(Expr::new(ExprKind::Literal(Literal::Float(v)), span))
            }
            Token::Complex(im) => {
                self.bump();
                Ok(Expr::new(ExprKind::Literal(Literal::Complex { re: 0.0, im }), span))
            }
            Token::String(s) => {
                self.bump();
                Ok(Expr::new(ExprKind::Literal(Literal::String(s)), span))
            }
            Token::True => {
                self.bump();
                Ok(Expr::new(ExprKind::Literal(Literal::Bool(true)), span))
            }
            Token::False => {
                self.bump();
                Ok(Expr::new(ExprKind::Literal(Literal::Bool(false)), span))
            }
            Token::LParen => {
                self.bump();
                if self.check(&Token::RParen) {
                    let end = self.bump().1;
                    return Ok(Expr::new(ExprKind::Literal(Literal::Unit), Self::join(span, end)));
                }
                let first = self.parse_expr()?;
                if self.check(&Token::Comma) {
                    let mut items = vec![first];
                    while self.eat(&Token::Comma) {
                        if self.check(&Token::RParen) {
                            break;
                        }
                        items.push(self.parse_expr()?);
                    }
                    let end = self.expect(&Token::RParen, "`)`")?;
                    return Ok(Expr::new(ExprKind::Tuple(items), Self::join(span, end)));
                }
                let end = self.expect(&Token::RParen, "`)`")?;
                // Parenthesized expression: keep the inner kind, widen the span.
                Ok(Expr { kind: first.kind, span: Self::join(span, end) })
            }
            Token::LBracket => {
                self.bump();
                let mut items = Vec::new();
                while !self.check(&Token::RBracket) {
                    items.push(self.parse_expr()?);
                    if !self.eat(&Token::Comma) {
                        break;
                    }
                }
                let end = self.expect(&Token::RBracket, "`]`")?;
                Ok(Expr::new(ExprKind::Array(items), Self::join(span, end)))
            }
            Token::LBrace => {
                let block = self.parse_block()?;
                let s = block.span;
                Ok(Expr::new(ExprKind::Block(block), s))
            }
            Token::KwIf => self.parse_if(),
            Token::KwFor => self.parse_for(),
            Token::KwWhile => self.parse_while(),
            Token::KwMatch => self.parse_match(),
            Token::Ident(_)
            | Token::KwQubit
            | Token::KwQubits
            | Token::KwMeasure
            | Token::KwDrop
            | Token::KwReverse
            | Token::KwAdjoint
            | Token::KwControl
            | Token::KwGrad => {
                let (name, nspan) = self.take_ident()?;
                if self.at_generic_call() {
                    self.bump(); // <
                    let arg = match self.bump().0 {
                        Token::Int(v) => TypeArg::Const(ConstExpr::Int(v)),
                        Token::Ident(s) => TypeArg::Const(ConstExpr::Ident(Ident::new(s))),
                        _ => return self.expected("a constant"),
                    };
                    self.bump(); // >
                    let (args, names, end) = self.parse_call_args()?;
                    let callee_span = nspan;
                    let span = Self::join(nspan, end);
                    return Ok(Expr::new(
                        ExprKind::Call(CallExpr {
                            callee: Box::new(Expr::new(ExprKind::Ident(name), callee_span)),
                            generics: vec![arg],
                            args,
                            names,
                            span,
                        }),
                        span,
                    ));
                }
                Ok(Expr::new(ExprKind::Ident(name), nspan))
            }
            Token::Pipe => {
                self.fail(codes::UNEXPECTED_TOKEN, "lambdas are not supported yet", span)
            }
            Token::Invalid(c) => {
                self.bump();
                self.fail(
                    codes::INVALID_CHARACTER,
                    format!("invalid character `{c}` in source"),
                    span,
                )
            }
            _ => {
                if self.at_eof() {
                    self.expected("an expression")
                } else {
                    let found = describe(&tok);
                    self.fail(
                        codes::EXPECTED_EXPRESSION,
                        format!("expected an expression, found {found}"),
                        span,
                    )
                }
            }
        }
    }

    fn parse_if(&mut self) -> PResult<Expr> {
        let start = self.expect(&Token::KwIf, "`if`")?;
        let cond = self.parse_expr()?;
        let then_branch = self.parse_block()?;
        let mut end = then_branch.span;
        let else_branch = if self.eat(&Token::KwElse) {
            let e = if self.check(&Token::KwIf) {
                self.parse_if()?
            } else {
                let b = self.parse_block()?;
                let s = b.span;
                Expr::new(ExprKind::Block(b), s)
            };
            end = e.span;
            Some(Box::new(e))
        } else {
            None
        };
        let span = Self::join(start, end);
        Ok(Expr::new(
            ExprKind::If(IfExpr { cond: Box::new(cond), then_branch, else_branch, span }),
            span,
        ))
    }

    fn parse_for(&mut self) -> PResult<Expr> {
        let start = self.expect(&Token::KwFor, "`for`")?;
        let pattern = self.parse_pattern()?;
        self.expect(&Token::KwIn, "`in`")?;
        let iterable = self.parse_expr()?;
        let body = self.parse_block()?;
        let span = Self::join(start, body.span);
        Ok(Expr::new(
            ExprKind::For(ForExpr { pattern, iterable: Box::new(iterable), body, span }),
            span,
        ))
    }

    fn parse_match(&mut self) -> PResult<Expr> {
        let start = self.expect(&Token::KwMatch, "`match`")?;
        let scrutinee = self.parse_expr()?;
        self.expect(&Token::LBrace, "`{`")?;
        let mut arms = Vec::new();
        while !self.check(&Token::RBrace) {
            let arm_start = self.span_here();
            let pattern = self.parse_pattern()?;
            let guard = if self.check(&Token::KwIf) {
                self.bump();
                Some(self.parse_expr()?)
            } else {
                None
            };
            self.expect(&Token::FatArrow, "`=>`")?;
            let body = self.parse_expr()?;
            let span = Self::join(arm_start, body.span);
            arms.push(MatchArm { pattern, guard, body, span });
            if !self.eat(&Token::Comma) && !self.check(&Token::RBrace) && !self.newline_before() {
                return self.expected("`,` or `}` after a match arm");
            }
        }
        let end = self.expect(&Token::RBrace, "`}`")?;
        let span = Self::join(start, end);
        Ok(Expr::new(
            ExprKind::Match(MatchExpr { scrutinee: Box::new(scrutinee), arms, span }),
            span,
        ))
    }

    fn parse_while(&mut self) -> PResult<Expr> {
        let start = self.expect(&Token::KwWhile, "`while`")?;
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        let span = Self::join(start, body.span);
        Ok(Expr::new(ExprKind::While(WhileExpr { cond: Box::new(cond), body, span }), span))
    }

    // ───────────────────────── patterns ─────────────────────────

    fn parse_pattern(&mut self) -> PResult<Pattern> {
        match self.peek().clone() {
            Token::Ident(s) if s == "_" => {
                self.bump();
                Ok(Pattern::Wildcard)
            }
            Token::LParen => {
                self.bump();
                let mut items = Vec::new();
                while !self.check(&Token::RParen) {
                    items.push(self.parse_pattern()?);
                    if !self.eat(&Token::Comma) {
                        break;
                    }
                }
                self.expect(&Token::RParen, "`)`")?;
                if items.is_empty() {
                    Ok(Pattern::Literal(Literal::Unit))
                } else {
                    Ok(Pattern::Tuple(items))
                }
            }
            Token::Int(_) | Token::True | Token::False => {
                let (tok, _) = self.bump();
                Ok(Pattern::Literal(match tok {
                    Token::Int(v) => Literal::Int(v),
                    Token::True => Literal::Bool(true),
                    _ => Literal::Bool(false),
                }))
            }
            _ => {
                let (name, _) = self.take_ident()?;
                Ok(Pattern::Ident(name))
            }
        }
    }

    // ───────────────────────── types ─────────────────────────

    fn parse_type(&mut self) -> PResult<Type> {
        let (tok, span) = self.toks.get(self.pos).cloned().unwrap_or((Token::Eof, self.eof_span()));
        match tok {
            Token::TyQubit => {
                self.bump();
                Ok(Type::Qubit)
            }
            Token::TyQubits => {
                self.bump();
                self.expect(&Token::Lt, "`<`")?;
                let size = self.parse_const_expr()?;
                self.expect(&Token::Gt, "`>`")?;
                Ok(Type::Qubits(Box::new(size)))
            }
            Token::TyMeasured => {
                self.bump();
                self.expect(&Token::Lt, "`<`")?;
                let inner = self.parse_type()?;
                self.expect(&Token::Gt, "`>`")?;
                Ok(Type::Measured(Box::new(inner)))
            }
            Token::TyVec => {
                self.bump();
                self.expect(&Token::Lt, "`<`")?;
                let inner = self.parse_type()?;
                self.expect(&Token::Gt, "`>`")?;
                Ok(Type::Named("Vec".to_string(), vec![inner]))
            }
            Token::TyBool
            | Token::TyInt
            | Token::TyFloat
            | Token::TyComplex
            | Token::TyString
            | Token::TyUnit
            | Token::TyParams => {
                self.bump();
                let name = match tok {
                    Token::TyBool => "Bool",
                    Token::TyInt => "Int",
                    Token::TyFloat => "Float",
                    Token::TyComplex => "Complex",
                    Token::TyString => "String",
                    Token::TyUnit => "Unit",
                    _ => "Params",
                };
                Ok(Type::Named(name.to_string(), Vec::new()))
            }
            Token::Ident(_) => {
                let (name, nspan) = self.take_ident()?;
                let mut type_args = Vec::new();
                if self.eat(&Token::Lt) {
                    while !self.check(&Token::Gt) {
                        type_args.push(self.parse_type_arg()?);
                        if !self.eat(&Token::Comma) {
                            break;
                        }
                    }
                    self.expect(&Token::Gt, "`>`")?;
                }
                let span = Self::join(nspan, self.prev_span());
                Ok(Type::Path(Path {
                    segments: vec![PathSegment { ident: name, type_args }],
                    span,
                }))
            }
            Token::LParen => {
                self.bump();
                let mut items = Vec::new();
                let mut trailing_comma = false;
                while !self.check(&Token::RParen) {
                    items.push(self.parse_type()?);
                    trailing_comma = self.eat(&Token::Comma);
                    if !trailing_comma {
                        break;
                    }
                }
                self.expect(&Token::RParen, "`)`")?;
                if items.len() == 1 && !trailing_comma {
                    Ok(items.remove(0))
                } else {
                    Ok(Type::Tuple(items))
                }
            }
            Token::LBracket => {
                self.bump();
                let elem = self.parse_type()?;
                self.expect(&Token::Semicolon, "`;`")?;
                let len = self.parse_const_expr()?;
                self.expect(&Token::RBracket, "`]`")?;
                Ok(Type::Array(Box::new(elem), Box::new(len)))
            }
            _ => {
                if self.at_eof() {
                    self.expected("a type")
                } else {
                    let found = describe(&tok);
                    self.fail(codes::EXPECTED_TYPE, format!("expected a type, found {found}"), span)
                }
            }
        }
    }

    fn parse_type_arg(&mut self) -> PResult<TypeArg> {
        match self.peek() {
            Token::Int(_) => Ok(TypeArg::Const(self.parse_const_expr()?)),
            _ => Ok(TypeArg::Type(self.parse_type()?)),
        }
    }

    fn parse_const_expr(&mut self) -> PResult<ConstExpr> {
        let mut lhs = match self.peek().clone() {
            Token::Int(v) => {
                self.bump();
                ConstExpr::Int(v)
            }
            Token::Ident(_) => {
                let (name, _) = self.take_ident()?;
                ConstExpr::Ident(name)
            }
            _ => return self.expected("a constant expression"),
        };
        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                Token::Star => BinOp::Mul,
                _ => break,
            };
            self.bump();
            let rhs = self.parse_const_expr_atom()?;
            lhs = ConstExpr::Binary { lhs: Box::new(lhs), op, rhs: Box::new(rhs) };
        }
        Ok(lhs)
    }

    fn parse_const_expr_atom(&mut self) -> PResult<ConstExpr> {
        match self.peek().clone() {
            Token::Int(v) => {
                self.bump();
                Ok(ConstExpr::Int(v))
            }
            Token::Ident(_) => {
                let (name, _) = self.take_ident()?;
                Ok(ConstExpr::Ident(name))
            }
            _ => self.expected("a constant expression"),
        }
    }

    // ───────────────────────── names ─────────────────────────

    /// Accept an identifier. Quantum builtins (`qubit`, `measure`, ...) are keywords to the
    /// lexer but ordinary names here.
    fn take_ident(&mut self) -> PResult<(Ident, Span)> {
        let (tok, span) = self.toks.get(self.pos).cloned().unwrap_or((Token::Eof, self.eof_span()));
        let name = match tok {
            Token::Ident(s) => s,
            Token::KwQubit => "qubit".to_string(),
            Token::KwQubits => "qubits".to_string(),
            Token::KwMeasure => "measure".to_string(),
            Token::KwDrop => "drop".to_string(),
            Token::KwReverse => "reverse".to_string(),
            Token::KwAdjoint => "adjoint".to_string(),
            Token::KwControl => "control".to_string(),
            Token::KwGrad => "grad".to_string(),
            Token::TyBool => "Bool".to_string(),
            Token::TyInt => "Int".to_string(),
            Token::TyFloat => "Float".to_string(),
            Token::TyComplex => "Complex".to_string(),
            Token::TyString => "String".to_string(),
            Token::TyUnit => "Unit".to_string(),
            Token::TyParams => "Params".to_string(),
            Token::TyMeasured => "Measured".to_string(),
            Token::TyQubit => "Qubit".to_string(),
            Token::TyQubits => "Qubits".to_string(),
            Token::TyVec => "Vec".to_string(),
            _ => return self.expected("an identifier"),
        };
        self.bump();
        Ok((Ident::new(name), span))
    }
}

/// A human-readable description of a token, used in diagnostics.
fn describe(tok: &Token) -> String {
    let text = match tok {
        Token::Ident(s) => return format!("identifier `{s}`"),
        Token::Int(v) => return format!("integer `{v}`"),
        Token::Float(v) => return format!("float `{v}`"),
        Token::Complex(v) => return format!("complex literal `{v}i`"),
        Token::String(_) => return "string literal".to_string(),
        Token::Eof => return "end of file".to_string(),
        Token::LParen => "`(`",
        Token::RParen => "`)`",
        Token::LBrace => "`{`",
        Token::RBrace => "`}`",
        Token::LBracket => "`[`",
        Token::RBracket => "`]`",
        Token::Comma => "`,`",
        Token::Semicolon => "`;`",
        Token::Colon => "`:`",
        Token::Arrow => "`->`",
        Token::Eq => "`=`",
        Token::EqEq => "`==`",
        Token::Lt => "`<`",
        Token::Gt => "`>`",
        Token::Dot => "`.`",
        Token::DotDot => "`..`",
        Token::Plus => "`+`",
        Token::Minus => "`-`",
        Token::Star => "`*`",
        Token::Slash => "`/`",
        Token::KwFn => "`fn`",
        Token::KwLet => "`let`",
        Token::KwIf => "`if`",
        Token::KwElse => "`else`",
        Token::KwFor => "`for`",
        Token::KwIn => "`in`",
        Token::KwReturn => "`return`",
        Token::KwImport => "`import`",
        Token::KwMut => "`mut`",
        Token::KwPure => "`pure`",
        Token::KwQuantum => "`quantum`",
        Token::KwProb => "`prob`",
        other => return format!("`{other:?}`"),
    };
    text.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> ParseOutput {
        parse_source(FileId(0), src)
    }

    fn error_codes(out: &ParseOutput) -> Vec<String> {
        out.diagnostics.errors().iter().map(|d| d.code.to_string()).collect()
    }

    #[test]
    fn parses_bell_pair_from_spec() {
        let src = "fn bell() -> Qubits<2> quantum {\n    let a = qubit()\n    let b = qubit()\n    h(a)\n    let (a, b) = cx(a, b)\n    (a, b)\n}\n";
        let out = parse(src);
        assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
        let Item::FnDef(f) = &out.items[0] else { panic!("expected fn") };
        assert_eq!(f.ident.as_str(), "bell");
        assert_eq!(f.effect, Some(Effect::Quantum));
        assert!(matches!(f.ret_ty, Some(Type::Qubits(_))));
        assert_eq!(f.body.as_ref().unwrap().stmts.len(), 4);
        assert!(f.body.as_ref().unwrap().tail.is_some(), "tuple on last line is the tail");
    }

    #[test]
    fn statements_end_at_newlines_without_semicolons() {
        let out = parse("fn f() -> Int {\n let x = 1\n let y = x + 1\n y\n}\n");
        assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
    }

    #[test]
    fn postfix_call_does_not_continue_across_newline() {
        // `cx(a, b)` then `(a, b)` on the next line must be two statements, not a call.
        let out = parse("fn f(a: Qubit, b: Qubit) quantum {\n cx(a, b)\n (a, b)\n}\n");
        assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
        let Item::FnDef(f) = &out.items[0] else { panic!() };
        assert_eq!(f.body.as_ref().unwrap().stmts.len(), 1);
        assert!(f.body.as_ref().unwrap().tail.is_some());
    }

    #[test]
    fn parses_generic_types_and_effects() {
        let out = parse("fn coin() -> Measured<Bool> prob { measure(qubit()) }");
        assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
        let Item::FnDef(f) = &out.items[0] else { panic!() };
        assert!(matches!(f.ret_ty, Some(Type::Measured(_))));
        assert_eq!(f.effect, Some(Effect::Prob));
    }

    #[test]
    fn parses_vec_tuple_and_for_loop() {
        let src = "fn g(x: Vec<Float>) -> (Int, Int) {\n for i in 0..3 { let z = i }\n (1, 2)\n}\n";
        let out = parse(src);
        assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
    }

    #[test]
    fn parses_tuple_pattern_and_qubits_generic_call() {
        let out = parse(
            "fn f() quantum { let (a, b) = cx(qubit(), qubit()); let q = qubits<4>(); drop(q) }",
        );
        assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
    }

    #[test]
    fn parses_named_arguments_and_index() {
        let out = parse("fn f(q: Qubits<4>) quantum { ry(q[0], 1.5); run(f, on: b) }");
        assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
    }

    #[test]
    fn parses_imports() {
        let out = parse(
            "import gala.vqa.{ angle_encode, Params }\nimport gala.core.*\nimport a.b as c\n",
        );
        assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
        assert_eq!(out.items.len(), 3);
    }

    #[test]
    fn parses_property_attribute() {
        let out = parse("#[property(unitary)]\nfn qft_is_unitary() { 1 }");
        assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
        let Item::FnDef(f) = &out.items[0] else { panic!() };
        assert_eq!(f.attributes[0].name.as_str(), "property");
        assert_eq!(f.attributes[0].args[0].as_str(), "unitary");
    }

    #[test]
    fn syntax_error_recovers_and_keeps_later_items() {
        let out = parse("fn broken( -> Int { 1 }\nfn fine() -> Int { 2 }\n");
        assert!(out.diagnostics.has_errors());
        assert!(error_codes(&out).iter().all(|c| c.starts_with("E01")));
        let names: Vec<&str> = out
            .items
            .iter()
            .filter_map(|i| if let Item::FnDef(f) = i { Some(f.ident.as_str()) } else { None })
            .collect();
        assert!(names.contains(&"fine"), "later item survived recovery: {names:?}");
    }

    #[test]
    fn missing_brace_reports_unexpected_eof() {
        let out = parse("fn f() -> Int { 1\n");
        assert!(out.diagnostics.has_errors());
        assert_eq!(error_codes(&out), vec!["E0110".to_string()]);
    }

    #[test]
    fn invalid_character_is_reported() {
        let out = parse("fn f() -> Int { 1 @ 2 }");
        assert!(error_codes(&out).contains(&"E0105".to_string()), "{:?}", out.diagnostics);
    }

    #[test]
    fn parse_file_returns_err_on_error() {
        let mut sm = SourceMap::new();
        let fid = sm.add_file("x.gala".into(), "fn f( {".to_string());
        assert!(parse_file(fid, "fn f( {", &mut sm).is_err());
        let fid2 = sm.add_file("y.gala".into(), "fn f() -> Int { 1 }".to_string());
        assert!(parse_file(fid2, "fn f() -> Int { 1 }", &mut sm).is_ok());
    }

    #[test]
    fn non_ascii_text_keeps_spans_aligned() {
        // Multi-byte characters before a statement must not shift its span, or the newline that
        // ends the previous statement would be read from the wrong place.
        let src = "// ⟨Z⟩ − multi-byte comment\nfn f() -> Int {\n    let a = 1\n    let b = a + 1\n    b\n}\n";
        let out = parse(src);
        assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
        let Item::FnDef(f) = &out.items[0] else { panic!() };
        let body = f.body.as_ref().unwrap();
        assert_eq!(body.stmts.len(), 2);
        assert!(body.tail.is_some());
    }

    #[test]
    fn parses_match_arms() {
        let out = parse("fn f(n: Int) -> String pure {\n let r = match n {\n 0 => \"zero\",\n _ => \"other\",\n }\n r\n}\n");
        assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
        let Item::FnDef(f) = &out.items[0] else { panic!() };
        let Stmt::Let(l) = &f.body.as_ref().unwrap().stmts[0] else { panic!() };
        let ExprKind::Match(m) = &l.init.as_ref().unwrap().kind else { panic!("not a match") };
        assert_eq!(m.arms.len(), 2);
    }

    #[test]
    fn expressions_keep_spans() {
        let src = "fn f() -> Int { 1 + 2 }";
        let out = parse(src);
        let Item::FnDef(f) = &out.items[0] else { panic!() };
        let tail = f.body.as_ref().unwrap().tail.as_ref().expect("tail");
        assert_eq!(
            &src[tail.span.byte_span.start as usize..tail.span.byte_span.end as usize],
            "1 + 2"
        );
    }
}
