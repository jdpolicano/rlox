use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ScanError {
    #[error("ScanError: unexpected end of file")]
    UnexpectedEOF,
    #[error("ScanError: token is invalid '{0}'")]
    InvalidToken(String, usize),
    #[error("ScanError: string literal is missing terminator")]
    StrMissingTerminator(String, usize),
    #[error("ScanError: invalid number '{0}'")]
    InvalidNumber(String, usize),
}
