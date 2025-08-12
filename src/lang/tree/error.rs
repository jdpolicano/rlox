use crate::lang::tokenizer::error::ScanError;
use crate::lang::tokenizer::span::Span;
use crate::lang::tokenizer::token::{OwnedToken, TokenType};
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum ConversionError {
    #[error("Invalid binary operator conversion {0}")]
    InvalidBinaryOperator(OwnedToken),
    #[error("Invalid unary operator conversion {0}")]
    InvalidUnaryOperator(OwnedToken),
    #[error("Invalid logical operator conversion {0}")]
    InvalidLogicalOperator(OwnedToken),
    #[error("Invalid literal conversion {0}")]
    InvalidLiteralType(OwnedToken),
    #[error("Failed to convert src string to a number {0}")]
    InvalidNumber(OwnedToken),
    #[error("Invalid token for identifier {0}")]
    InvalidIdentifier(OwnedToken),
}

// todo: fill this out.s
#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("{0}")]
    ScanError(#[from] ScanError),
    #[error("SyntaxError: {0}")]
    ConversionError(#[from] ConversionError),
    #[error("SyntaxError: {msg} expected {expected} but recieved {recieved}")]
    UnexpectedToken {
        expected: TokenType,
        recieved: String,
        msg: &'static str,
        span: Span,
    },
    #[error("SyntaxError: cannot assign to type '{type_str}'")]
    UnexpectedAssignment { type_str: String, span: Span },
    #[error("SyntaxError: cannot use '{type_str}' out side of a loop")]
    InvalidLoopKeyword { type_str: String, span: Span },
    #[error("SyntaxError: cannot use 'return' out side of a function")]
    InvalidReturn { span: Span },
    #[error("SyntaxError: function arguments cannot exceed 255")]
    FuncExceedMaxArgs { max: usize, span: Span },
    #[error("SyntaxError: invalid function statement")]
    InvalidFuncStatement { span: Span },
    #[error("SyntaxError: invalid class method")]
    InvalidClassMethod { span: Span },
    #[error("SyntaxError: unexpected end of file")]
    UnexpectedEof,
}

impl ParseError {
    pub fn span(&self) -> Option<Span> {
        match self {
            Self::FuncExceedMaxArgs { span, .. } => Some(*span),
            Self::InvalidClassMethod { span } => Some(*span),
            Self::InvalidFuncStatement { span } => Some(*span),
            Self::InvalidLoopKeyword { span, .. } => Some(*span),
            Self::InvalidReturn { span } => Some(*span),
            Self::UnexpectedAssignment { span, .. } => Some(*span),
            Self::UnexpectedToken { span, .. } => Some(*span),
            _ => None,
        }
    }
    pub fn print_code_block(&self, src: &str) {
        if let Some(span) = self.span() {
            let mut line_cnt = 0;
            let mut line_begin = 0;
            let idx = 0;
            for (i, ch) in src.char_indices() {
                if i >= span.start {
                    break;
                }
                if ch == '\n' {
                    line_cnt += 1;
                    line_begin = i + 1;
                }
            }
            println!("{line_cnt}  |   {}", &src[line_begin..span.end]);
        }
    }
}
