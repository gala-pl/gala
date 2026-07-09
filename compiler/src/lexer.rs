use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),

    // Identifiers & keywords
    Ident(String),
    Let,
    Fn,
    If,
    Else,
    Return,
    True,
    False,

    // Symbols
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    EqEq,
    Bang,
    BangEq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    AndAnd,
    Or,
    OrOr,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Semicolon,
    Colon,
    Arrow,
    // ... is a "rest" / spread operator? Not needed yet.

    // Special
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Int(n) => write!(f, "{n}"),
            Token::Float(n) => write!(f, "{n}"),
            Token::String(s) => write!(f, "\"{s}\""),
            Token::Bool(b) => write!(f, "{b}"),
            Token::Ident(s) => write!(f, "{s}"),
            Token::Let => write!(f, "let"),
            Token::Fn => write!(f, "fn"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::Return => write!(f, "return"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Eq => write!(f, "="),
            Token::EqEq => write!(f, "=="),
            Token::Bang => write!(f, "!"),
            Token::BangEq => write!(f, "!="),
            Token::Lt => write!(f, "<"),
            Token::Le => write!(f, "<="),
            Token::Gt => write!(f, ">"),
            Token::Ge => write!(f, ">="),
            Token::And => write!(f, "&"),
            Token::AndAnd => write!(f, "&&"),
            Token::Or => write!(f, "|"),
            Token::OrOr => write!(f, "||"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::Colon => write!(f, ":"),
            Token::Arrow => write!(f, "->"),
            Token::Eof => write!(f, "<eof>"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self { chars: input.chars().collect(), pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        self.pos += 1;
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self, first: char) -> Token {
        let mut s = String::new();
        s.push(first);
        let mut is_float = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(c);
                self.advance();
            } else if c == '.' {
                is_float = true;
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        if is_float {
            Token::Float(s.parse().unwrap())
        } else {
            Token::Int(s.parse().unwrap())
        }
    }

    fn read_ident(&mut self, first: char) -> Token {
        let mut s = String::new();
        s.push(first);
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        match s.as_str() {
            "let" => Token::Let,
            "fn" => Token::Fn,
            "if" => Token::If,
            "else" => Token::Else,
            "return" => Token::Return,
            "true" => Token::True,
            "false" => Token::False,
            _ => Token::Ident(s),
        }
    }

    fn read_string(&mut self) -> Token {
        let mut s = String::new();
        while let Some(c) = self.advance() {
            if c == '"' {
                break;
            }
            if c == '\\' {
                match self.advance() {
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('"') => s.push('"'),
                    Some('\\') => s.push('\\'),
                    Some(c) => s.push(c),
                    None => break,
                }
            } else {
                s.push(c);
            }
        }
        Token::String(s)
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let Some(c) = self.advance() else {
            return Token::Eof;
        };
        match c {
            '0'..='9' => self.read_number(c),
            'a'..='z' | 'A'..='Z' | '_' => self.read_ident(c),
            '"' => self.read_string(),
            '+' => Token::Plus,
            '-' => {
                if self.peek() == Some('>') {
                    self.advance();
                    Token::Arrow
                } else {
                    Token::Minus
                }
            }
            '*' => Token::Star,
            '/' => {
                if self.peek() == Some('/') {
                    while self.advance().is_some() && self.peek() != Some('\n') {}
                    self.next_token()
                } else {
                    Token::Slash
                }
            }
            '%' => Token::Percent,
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::EqEq
                } else {
                    Token::Eq
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::BangEq
                } else {
                    Token::Bang
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::Le
                } else {
                    Token::Lt
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::Ge
                } else {
                    Token::Gt
                }
            }
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                    Token::AndAnd
                } else {
                    Token::And
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    Token::OrOr
                } else {
                    Token::Or
                }
            }
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            ',' => Token::Comma,
            ';' => Token::Semicolon,
            ':' => Token::Colon,
            _ => Token::Ident(c.to_string()),
        }
    }
}
