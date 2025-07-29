use crate::lang::tokenizer::error::ScanError;
use crate::lang::tokenizer::token::{OwnedToken, TokenType};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("Invalid binary operator conversion {0}")]
    InvalidBinaryOperator(OwnedToken),
    #[error("Invalid unary operator conversion {0}")]
    InvalidUnaryOperator(OwnedToken),
    #[error("Invalid literal conversion {0}")]
    InvalidLiteralType(OwnedToken),
    #[error("Failed to convert src string to a number {0}")]
    InvalidNumber(OwnedToken),
}

// todo: fill this out.s
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("{0}")]
    ScanError(#[from] ScanError),
    #[error("{msg} expected {expected} but recieved {recieved}")]
    UnexpectedToken {
        expected: TokenType,
        recieved: String,
        msg: &'static str,
    },
    #[error("unexpected end of file")]
    UnexpectedEof,
    #[error("{0}")]
    ConversionError(#[from] ConversionError),
}
