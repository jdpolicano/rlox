use crate::lang::view::View;
use thiserror::Error;

// this is purly for routing logic to understand why something failed.
// It is not intended to be printed directly.
#[derive(Debug, Clone)]
pub enum BinaryError {
    LeftSide,
    RightSide,
    InvalidOperator,
    InvalidTypes,
}

#[derive(Error, Debug, Clone)]
pub enum LoxError {
    #[error("TypeError: {msg} {view}")]
    TypeError { msg: String, view: View },
    #[error("ReferenceError: {name} is undefined {view}")]
    ReferenceError { name: String, view: View },
    #[error("NativeError: {0}")]
    NativeError(#[from] NativeError),
    #[error("DebugError: {0}")]
    DebugError(&'static str),
}

#[derive(Error, Debug, Clone)]
pub enum NativeError {
    #[error("{0}")]
    SystemError(String),
}
