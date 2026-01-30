use std::sync::Arc;

use thiserror::Error;

use crate::info::Type;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid field {0}.")]
    InvalidField(String),
    #[error("Expected struct type, found {0}.")]
    ExpectedStructType(Arc<Type>),
    #[error("Invalid type cast: {0} -> {1}.")]
    InvalidTypeCast(Arc<Type>, Arc<Type>),
    #[error("Dereferencing value with type {0}.")]
    WrongDeref(Arc<Type>),
    #[error("Calling value with type {0}.")]
    WrongCall(Arc<Type>),
    #[error("Type mismatch: Expected {0}, found {1}")]
    TypeMismatch(Arc<Type>, Arc<Type>),
    #[error("Unsupported operator {0} for type {1}")]
    UnsupportedOperator(String, Arc<Type>),
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
