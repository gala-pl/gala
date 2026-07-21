//! Hand-written lexer for the Gala programming language.

use gala_span::{ByteSpan, FileId, Span};

/// All tokens in the Gala language.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Int(i64),
    Float(f64),
    Complex(f64),
    String(String),
    True,
    False,

    // Identifiers and keywords
    Ident(String),

    // Keywords
    KwFn,
    KwLet,
    KwMut,
    KwIf,
    KwElse,
    KwMatch,
    KwFor,
    KwIn,
    KwWhile,
    KwReturn,
    KwImport,
    KwAs,
    KwStruct,
    KwEnum,
    KwTrait,
    KwImpl,
    KwType,
    KwConst,
    KwPure,
    KwQuantum,
    KwProb,
    KwQubit,
    KwQubits,
    KwMeasure,
    KwReverse,
    KwAdjoint,
    KwControl,
    KwGrad,
    KwDrop,

    // Type keywords
    TyBool,
    TyInt,
    TyFloat,
    TyComplex,
    TyString,
    TyUnit,
    TyParams,
    TyMeasured,
    TyQubit,
    TyQubits,
    TyVec,

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
    AndAnd,
    OrOr,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Semicolon,
    Colon,
    Arrow,
    Dot,
    DotDot,
    LBracket,
    RBracket,
    LAngle,
    RAngle,
    Pipe,
    FatArrow,

    // End of file
    Eof,
    Underscore,
}

impl Token {
    pub fn is_type_keyword(&self) -> bool {
        matches!(
            self,
            Token::TyBool
                | Token::TyInt
                | Token::TyFloat
                | Token::TyComplex
                | Token::TyString
                | Token::TyUnit
                | Token::TyParams
                | Token::TyMeasured
                | Token::TyQubit
                | Token::TyQubits
                | Token::TyVec
        )
    }
}

/// Hand-written lexer with source position tracking.
pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    file_id: FileId,
}

impl<'a> Lexer {
    pub fn new(file_id: FileId, source: &'a str) -> Self {
        Lexer { chars: source.chars().collect(), pos: 0, file_id }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_n(&self, n: usize) -> Option<char> {
        self.chars.get(self.pos + n).copied()
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

    fn skip_comment(&mut self) {
        if self.peek() == Some('/') && self.peek_n(1) == Some('/') {
            // Line comment
            self.advance();
            self.advance();
            while let Some(c) = self.advance() {
                if c == '\n' {
                    break;
                }
            }
        } else if self.peek() == Some('/') && self.peek_n(1) == Some('*') {
            // Block comment (support nested)
            self.advance();
            self.advance();
            let mut depth = 1;
            while depth > 0 {
                match (self.peek(), self.peek_n(1)) {
                    (Some('/'), Some('*')) => {
                        self.advance();
                        self.advance();
                        depth += 1;
                    }
                    (Some('*'), Some('/')) => {
                        self.advance();
                        self.advance();
                        depth -= 1;
                    }
                    (Some(_), _) => {
                        self.advance();
                    }
                    (None, _) => break,
                }
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
            } else if c == '.' && !is_float {
                is_float = true;
                s.push(c);
                self.advance();
            } else if (c == 'e' || c == 'E') && !is_float {
                is_float = true;
                s.push(c);
                self.advance();
                if let Some(sign) = self.peek() {
                    if sign == '+' || sign == '-' {
                        s.push(sign);
                        self.advance();
                    }
                }
            } else if c == 'i' || c == 'j' || c == 'I' || c == 'J' {
                // Complex number - the imaginary unit follows the number
                let num = s.parse::<f64>().unwrap_or(0.0);
                self.advance();
                return Token::Complex(num);
            } else {
                break;
            }
        }
        if is_float {
            Token::Float(s.parse().unwrap_or(0.0))
        } else {
            Token::Int(s.parse().unwrap_or(0))
        }
    }

    fn read_ident(&mut self, first: char) -> Token {
        let mut s = String::new();
        s.push(first);
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || ('\u{0391}'..='\u{03C9}').contains(&c) {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        match s.as_str() {
            "fn" => Token::KwFn,
            "let" => Token::KwLet,
            "mut" => Token::KwMut,
            "if" => Token::KwIf,
            "else" => Token::KwElse,
            "match" => Token::KwMatch,
            "for" => Token::KwFor,
            "in" => Token::KwIn,
            "while" => Token::KwWhile,
            "return" => Token::KwReturn,
            "import" => Token::KwImport,
            "as" => Token::KwAs,
            "struct" => Token::KwStruct,
            "enum" => Token::KwEnum,
            "trait" => Token::KwTrait,
            "impl" => Token::KwImpl,
            "type" => Token::KwType,
            "const" => Token::KwConst,
            "pure" => Token::KwPure,
            "quantum" => Token::KwQuantum,
            "prob" => Token::KwProb,
            "qubit" => Token::KwQubit,
            "qubits" => Token::KwQubits,
            "measure" => Token::KwMeasure,
            "reverse" => Token::KwReverse,
            "adjoint" => Token::KwAdjoint,
            "control" => Token::KwControl,
            "grad" => Token::KwGrad,
            "drop" => Token::KwDrop,
            "Bool" => Token::TyBool,
            "Int" => Token::TyInt,
            "Float" => Token::TyFloat,
            "Complex" => Token::TyComplex,
            "String" => Token::TyString,
            "Unit" => Token::TyUnit,
            "Params" => Token::TyParams,
            "Measured" => Token::TyMeasured,
            "Qubit" => Token::TyQubit,
            "Qubits" => Token::TyQubits,
            "Vec" => Token::TyVec,
            "true" => Token::True,
            "false" => Token::False,
            _ => Token::Ident(s),
        }
    }

    fn read_string(&mut self) -> Token {
        let mut s = String::new();
        // opening quote was already consumed by next_token
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

    pub fn next_token(&mut self) -> Option<(Token, Span)> {
        self.skip_whitespace();
        self.skip_comment();
        self.skip_whitespace();

        let start = self.pos;
        let c = self.advance()?;
        let token = match c {
            '0'..='9' => self.read_number(c),
            'a'..='z' | 'A'..='Z' | '_' | '\u{0391}'..='\u{03C9}' => self.read_ident(c),
            '"' => self.read_string(),
            '+' => Token::Plus,
            '-' => {
                if matches!(self.peek(), Some('>')) {
                    self.advance();
                    Token::Arrow
                } else {
                    Token::Minus
                }
            }
            '*' => Token::Star,
            '/' => {
                if matches!(self.peek(), Some('/')) {
                    while self.advance().is_some() && self.peek() != Some('\n') {}
                    return self.next_token();
                } else if matches!(self.peek(), Some('*')) {
                    self.advance();
                    let mut depth = 1;
                    while depth > 0 {
                        match (self.peek(), self.peek_n(1)) {
                            (Some('/'), Some('*')) => {
                                self.advance();
                                self.advance();
                                depth += 1;
                            }
                            (Some('*'), Some('/')) => {
                                self.advance();
                                self.advance();
                                depth -= 1;
                            }
                            (Some(_), _) => {
                                self.advance();
                            }
                            (None, _) => break,
                        }
                    }
                    return self.next_token();
                } else {
                    Token::Slash
                }
            }
            '%' => Token::Percent,
            '=' => match self.peek() {
                Some('=') => {
                    self.advance();
                    Token::EqEq
                }
                Some('>') => {
                    self.advance();
                    Token::FatArrow
                }
                _ => Token::Eq,
            },
            '!' => {
                if matches!(self.peek(), Some('=')) {
                    self.advance();
                    Token::BangEq
                } else {
                    Token::Bang
                }
            }
            '<' => {
                if matches!(self.peek(), Some('=')) {
                    self.advance();
                    Token::Le
                } else {
                    Token::Lt
                }
            }
            '>' => {
                if matches!(self.peek(), Some('=')) {
                    self.advance();
                    Token::Ge
                } else {
                    Token::Gt
                }
            }
            '&' => {
                if matches!(self.peek(), Some('&')) {
                    self.advance();
                    Token::AndAnd
                } else {
                    return self.next_token(); // Skip single &
                }
            }
            '|' => {
                if matches!(self.peek(), Some('|')) {
                    self.advance();
                    Token::OrOr
                } else if matches!(self.peek(), Some('>')) {
                    self.advance();
                    Token::FatArrow
                } else {
                    Token::Pipe
                }
            }
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            ',' => Token::Comma,
            ';' => Token::Semicolon,
            ':' => Token::Colon,
            '.' => {
                if matches!(self.peek(), Some('.')) {
                    self.advance();
                    Token::DotDot
                } else {
                    Token::Dot
                }
            }
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            _ => return self.next_token(), // Skip unknown characters
        };

        let end = self.pos;
        let byte_span = ByteSpan { start: start as u32, end: end as u32 };
        Some((token, Span::new(self.file_id, byte_span)))
    }

    pub fn collect_all(mut self) -> Vec<(Token, Span)> {
        let mut tokens = Vec::new();
        while let Some(tok) = self.next_token() {
            tokens.push(tok);
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gala_span::SourceMap;

    #[test]
    fn test_lexer_keywords() {
        let source =
            "fn let if else match for while return import as struct enum trait impl type const";
        let mut map = SourceMap::new();
        let fid = map.add_file("test.gala".into(), source.to_string());
        let lexer = Lexer::new(fid, source);
        let tokens = lexer.collect_all();

        assert_eq!(tokens.len(), 16);
        assert!(matches!(tokens[0].0, Token::KwFn));
        assert!(matches!(tokens[1].0, Token::KwLet));
        assert!(matches!(tokens[2].0, Token::KwIf));
    }

    #[test]
    fn test_lexer_quantum_keywords() {
        let source = "pure quantum prob qubit qubits measure reverse adjoint control grad drop";
        let mut map = SourceMap::new();
        let fid = map.add_file("test.gala".into(), source.to_string());
        let lexer = Lexer::new(fid, source);
        let tokens = lexer.collect_all();

        assert_eq!(tokens.len(), 11);
        assert!(matches!(tokens[0].0, Token::KwPure));
        assert!(matches!(tokens[1].0, Token::KwQuantum));
        assert!(matches!(tokens[2].0, Token::KwProb));
        assert!(matches!(tokens[3].0, Token::KwQubit));
        assert!(matches!(tokens[4].0, Token::KwQubits));
        assert!(matches!(tokens[5].0, Token::KwMeasure));
        assert!(matches!(tokens[6].0, Token::KwReverse));
        assert!(matches!(tokens[7].0, Token::KwAdjoint));
        assert!(matches!(tokens[8].0, Token::KwControl));
        assert!(matches!(tokens[9].0, Token::KwGrad));
        assert!(matches!(tokens[10].0, Token::KwDrop));
    }

    #[test]
    fn test_lexer_literals() {
        let source = "42 2.5 2+3i true false \"hello\"";
        let mut map = SourceMap::new();
        let fid = map.add_file("test.gala".into(), source.to_string());
        let lexer = Lexer::new(fid, source);
        let tokens = lexer.collect_all();

        assert_eq!(
            tokens.len(),
            8,
            "expected 8 tokens but got {}: {:?}",
            tokens.len(),
            tokens.iter().map(|(t, _)| format!("{:?}", t)).collect::<Vec<_>>()
        );
        assert!(matches!(tokens[0].0, Token::Int(42)));
        assert!(matches!(tokens[1].0, Token::Float(f) if (f - 2.5).abs() < f64::EPSILON));
        assert!(matches!(tokens[2].0, Token::Int(2)));
        assert!(matches!(tokens[3].0, Token::Plus));
        assert!(matches!(tokens[4].0, Token::Complex(c) if (c - 3.0).abs() < f64::EPSILON));
        assert!(matches!(tokens[5].0, Token::True));
        assert!(matches!(tokens[6].0, Token::False));
        assert!(matches!(tokens[7].0, Token::String(ref s) if s == "hello"));
    }

    #[test]
    fn test_lexer_unicode_identifiers() {
        let source = "θ φ ψ Θ Φ Ψ α β γ";
        let mut map = SourceMap::new();
        let fid = map.add_file("test.gala".into(), source.to_string());
        let lexer = Lexer::new(fid, source);
        let tokens = lexer.collect_all();

        assert_eq!(tokens.len(), 9);
        for tok in tokens {
            assert!(matches!(tok.0, Token::Ident(_)));
        }
    }

    #[test]
    fn test_lexer_nested_comments() {
        let source = "/* outer /* inner */ comment */ fn main() {}";
        let mut map = SourceMap::new();
        let fid = map.add_file("test.gala".into(), source.to_string());
        let lexer = Lexer::new(fid, source);
        let tokens = lexer.collect_all();

        // Should skip the entire nested comment
        assert!(matches!(tokens[0].0, Token::KwFn));
    }
}
