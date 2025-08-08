use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("{reason}")]
    WithLocation {
        #[source]
        reason: LoxError,
        place: usize,
    },
    #[error("{reason}")]
    Without {
        #[from]
        #[source]
        reason: LoxError,
    },
}

impl RuntimeError {
    pub fn with_place(self, place: usize) -> Self {
        match self {
            Self::WithLocation { .. } => self, // you cannot mutate the location originally attached to it.
            Self::Without { reason } => Self::WithLocation { reason, place },
        }
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
    InvalidOperator,
    InvalidTypes,
}
