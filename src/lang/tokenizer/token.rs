use std::fmt;

use crate::lang::tokenizer::span::Span;

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Semicolon,

    // One or two character tokens.
    Minus,
    MinusEqual,
    Plus,
    PlusEqual,
    Slash,
    SlashEqual,
    Star,
    StarEqual,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    False,
    Fun,
    For,
    If,
    Else,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Break,
    Continue,
    Static,

    // End of file
    Eof,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let representation = match self {
            TokenType::LeftParen => "(",
            TokenType::RightParen => ")",
            TokenType::LeftBrace => "{",
            TokenType::RightBrace => "}",
            TokenType::Comma => ",",
            TokenType::Dot => ".",
            TokenType::Semicolon => ";",
            TokenType::Minus => "-",
            TokenType::MinusEqual => "-=",
            TokenType::Plus => "+",
            TokenType::PlusEqual => "+=",
            TokenType::Slash => "/",
            TokenType::SlashEqual => "/=",
            TokenType::Star => "*",
            TokenType::StarEqual => "*=",
            TokenType::Bang => "!",
            TokenType::BangEqual => "!=",
            TokenType::Equal => "=",
            TokenType::EqualEqual => "==",
            TokenType::Greater => ">",
            TokenType::GreaterEqual => ">=",
            TokenType::Less => "<",
            TokenType::LessEqual => "<=",
            TokenType::Identifier => "identifier",
            TokenType::String => "string",
            TokenType::Number => "number",
            TokenType::And => "and",
            TokenType::Class => "class",
            TokenType::False => "false",
            TokenType::Fun => "fun",
            TokenType::For => "for",
            TokenType::If => "if",
            TokenType::Else => "else",
            TokenType::Nil => "nil",
            TokenType::Or => "or",
            TokenType::Print => "print",
            TokenType::Return => "return",
            TokenType::Super => "super",
            TokenType::This => "this",
            TokenType::True => "true",
            TokenType::Var => "var",
            TokenType::While => "while",
            TokenType::Break => "break",
            TokenType::Continue => "continue",
            TokenType::Static => "static",
            TokenType::Eof => "eof",
        };
        write!(f, "{}", representation)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token<'src> {
    pub token_type: TokenType,
    pub lexeme: &'src str,
    pub span: Span,
}

impl<'src> fmt::Display for Token<'src> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}'", self.lexeme)
    }
}

impl<'src> Token<'src> {
    pub fn new(token_type: TokenType, lexeme: &'src str, start: usize) -> Token<'src> {
        Token {
            token_type,
            lexeme,
            span: Span::new(start, start + lexeme.len()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OwnedToken {
    pub token_type: TokenType,
    pub lexeme: String,
    pub span: Span,
}

impl fmt::Display for OwnedToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}'", self.lexeme)
    }
}

impl OwnedToken {
    pub fn new(token_type: TokenType, lexeme: String, start: usize) -> Self {
        let end = start + lexeme.len();
        Self {
            token_type,
            lexeme,
            span: Span::new(start, end),
        }
    }
}

impl<'a> From<Token<'a>> for OwnedToken {
    fn from(value: Token<'a>) -> Self {
        Self::new(value.token_type, value.lexeme.to_string(), value.span.start)
    }
}
