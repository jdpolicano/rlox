use crate::lang::view::View;
use thiserror::Error;

// this is purly for routing logic to understand why something failed.
// It is not intended to be printed directly.
#[derive(Debug, Clone)]
pub enum BinaryError {
    LeftSide,
    RightSide,
    InvalidOperator,
}

#[derive(Error, Debug, Clone)]
pub enum LoxError {
    #[error("{msg} {view}")]
    TypeError { msg: String, view: View },
}
