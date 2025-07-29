use crate::lang::view::View;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ScanError {
    #[error("unexpected end of file")]
    UnexpectedEOF,
    #[error("token is invalid '{0}' {1}")]
    InvalidToken(String, View),
    #[error("string literal is missing terminator '{0}' {1}")]
    StrMissingTerminator(String, View),
    #[error("invalid number '{0}' {1}")]
    InvalidNumber(String, View),
}
