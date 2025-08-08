use crate::lang::tokenizer::error::ScanError;
use crate::lang::tokenizer::token::{OwnedToken, TokenType};
use thiserror::Error;

#[derive(Debug, Error)]
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
#[derive(Error, Debug)]
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
    },
    #[error("SyntaxError: cannot assign to type '{type_str}'")]
    UnexpectedAssignment { type_str: String, location: usize },
    #[error("SyntaxError: cannot use '{type_str}' out side of a loop")]
    InvalidLoopKeyword { type_str: String, location: usize },
    #[error("SyntaxError: cannot use 'return' out side of a function")]
    InvalidReturn { location: usize },
    #[error("SyntaxError: function arguments cannot exceed 255")]
    FuncExceedMaxArgs { max: usize, location: usize },
    #[error("SyntaxError: invalid function statement")]
    InvalidFuncStatement { location: usize },
    #[error("SyntaxError: invalid class method")]
    InvalidClassMethod { location: usize },
    #[error("SyntaxError: unexpected end of file")]
    UnexpectedEof,
}
