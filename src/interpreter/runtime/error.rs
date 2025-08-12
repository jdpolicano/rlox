use crate::lang::tokenizer::span::Span;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("{reason}")]
pub struct RuntimeError {
    #[source]
    reason: LoxError,
    place: Span,
}

impl RuntimeError {
    pub fn new(reason: LoxError, place: Span) -> Self {
        Self { reason, place }
    }
}

#[derive(Error, Debug, Clone)]
pub enum LoxError {
    #[error("TypeError: {0}")]
    TypeError(String),
    #[error("ReferenceError: {0}")]
    ReferenceError(String),
    #[error(transparent)]
    NativeError(#[from] NativeError),
    #[error("DebugError: {0}")]
    DebugError(&'static str),
    #[error("TypeError: {0}")]
    EvalUnwrapError(String),
    #[error("Uncaught SyntaxError: {0}")]
    UncaughtSyntaxError(String),
}

#[derive(Error, Debug, Clone)]
pub enum NativeError {
    #[error("NativeError: {0}")]
    SystemError(String),
    #[error("NativeError: {0}")]
    InvalidArguments(String),
}

// this is purly for routing logic to understand why something failed.
// It is not intended to be printed directly.
#[derive(Debug, Clone)]
pub enum BinaryError {
    LeftSide,
    RightSide,
    InvalidTypes,
}
