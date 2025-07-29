use crate::lang::view::View;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ScanError {
    #[error("ScanError: unexpected end of file")]
    UnexpectedEOF,
    #[error("ScanError: token is invalid '{0}' {1}")]
    InvalidToken(String, View),
    #[error("ScanError: string literal is missing terminator '{0}' {1}")]
    StrMissingTerminator(String, View),
    #[error("ScanError: invalid number '{0}' {1}")]
    InvalidNumber(String, View),
}
