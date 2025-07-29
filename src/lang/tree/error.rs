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
    #[error("Invalid token for identifier {0}")]
    InvalidIdentifier(OwnedToken),
}

// todo: fill this out.s
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("ParseError: {0}")]
    ScanError(#[from] ScanError),
    #[error("ParseError: {msg} expected {expected} but recieved {recieved}")]
    UnexpectedToken {
        expected: TokenType,
        recieved: String,
        msg: &'static str,
    },
    #[error("ParseError: unexpected end of file")]
    UnexpectedEof,
    #[error("ParseError: {0}")]
    ConversionError(#[from] ConversionError),
}
