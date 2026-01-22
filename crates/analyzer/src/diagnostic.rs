use thiserror::Error;

use crate::Type;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid field {0}.")]
    InvalidField(String),
    #[error("Expected struct type, found {0}.")]
    ExpectedStructType(Type),
    #[error("Invalid type cast: {0} -> {1}.")]
    InvalidTypeCast(Type, Type),
    #[error("Dereferencing value with type {0}.")]
    WrongDeref(Type),
    #[error("Calling value with type {0}.")]
    WrongCall(Type),
    #[error("Type mismatch: Expected {0}, found {1}")]
    TypeMismatch(Type, Type),
    #[error("Unsupported operator {0} for type {1}")]
    UnsupportedOperator(String, Type),
    #[error("Unknown variable or const {0}")]
    Unknown(String),
    #[error("{0}")]
    Custom(String),
    #[error("Break statement outside of loop")]
    BreakOutsideLoop,
    #[error("Continue statement outside of loop")]
    ContinueOutsideLoop,
}

#[derive(Debug, Error)]
pub enum Warning {
    #[error("{0}")]
    Custom(String),
}
