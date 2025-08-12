use crate::lang::tokenizer::span::Span;
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub enum BinOpSide {
    Lhs,
    Rhs,
}

impl std::fmt::Display for BinOpSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOpSide::Lhs => write!(f, "left-hand side"),
            BinOpSide::Rhs => write!(f, "right-hand side"),
        }
    }
}

#[derive(Debug, Clone, Copy, Error)]
pub enum BinOpError {
    #[error("division by zero encountered.")]
    DivByZero,
    #[error("negation of a non-number value encountered")]
    NegOpFailure,
    #[error("addition operation failed on the {0} side.")]
    AddOpFailure(BinOpSide),
    #[error("subtraction operation failed on the {0} side.")]
    SubOpFailure(BinOpSide),
    #[error("division operation failed on the {0} side.")]
    DivOpFailure(BinOpSide),
    #[error("multiplication operation failed on the {0} side.")]
    MulOpFailure(BinOpSide),
}

#[derive(Debug, Clone, Copy, Error)]
pub enum TypeError {
    #[error(transparent)]
    BinaryOp(#[from] BinOpError),
}

#[derive(Debug, Clone, Copy, Error)]
pub enum LoxError {
    #[error(transparent)]
    TypeError(#[from] TypeError),
}

#[derive(Debug, Clone, Copy, Error)]
pub struct ErrorObject {
    #[source]
    pub source: LoxError,
    pub span: Option<Span>,
}

impl ErrorObject {
    pub fn new(source: LoxError) -> Self {
        Self { source, span: None }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
}

impl std::fmt::Display for ErrorObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.span {
            Some(span) => write!(
                f,
                "{} (at indices {}..{})",
                self.source, span.start, span.end
            ),
            None => write!(f, "{}", self.source),
        }
    }
}

impl From<LoxError> for ErrorObject {
    fn from(source: LoxError) -> Self {
        Self { source, span: None }
    }
}

impl From<TypeError> for ErrorObject {
    fn from(source: TypeError) -> Self {
        Self {
            source: LoxError::from(source),
            span: None,
        }
    }
}

impl From<BinOpError> for ErrorObject {
    fn from(source: BinOpError) -> Self {
        Self {
            source: LoxError::from(TypeError::from(source)),
            span: None,
        }
    }
}
