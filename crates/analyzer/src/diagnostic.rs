use thiserror::Error;

use crate::Type;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Type mismatch: {0} and {1}")]
    TypeMismatch(Type, Type),
    #[error("Unsupported operator {0} for type {1}")]
    UnsupportedOperator(String, Type),
    #[error("{0}")]
    Custom(String),
}

#[derive(Debug, Error)]
pub enum Warning {
    #[error("{0}")]
    Custom(String),
}
