use std::fmt::Display;

use thiserror::Error;

use crate::ast::Span;

#[derive(Error, Debug, PartialEq)]
pub enum CompileErrorEnum {
    #[error("cannot find value `{0}` in this scope")]
    SymbolNotFound(String),
    #[error("cannot cast `{from}` as `{to}`!")]
    InvalidTypeCast {
        from: String,
        to: String,
    },
    #[error("terminated")]
    Terminated,
    #[error("cannot use value of type `{0}` as none-comptime value")]
    NonComptimeValue(String),
}

#[derive(Error, Debug)]
pub struct CompileError {
    pub span: Span,
    #[source]
    pub error: CompileErrorEnum,
}

impl CompileError {
    pub fn new_symbol_not_found(span: Span, id: String) -> Self {
        Self {
            span,
            error: CompileErrorEnum::SymbolNotFound(id),
        }
    }

    pub fn new_invalid_type_cast(span: Span, from: String, to: String) -> Self {
        Self {
            span,
            error: CompileErrorEnum::InvalidTypeCast { from, to },
        }
    }

    pub fn new_terminated(span: Span) -> Self {
        Self {
            span,
            error: CompileErrorEnum::Terminated,
        }
    }

    pub fn new_non_comptime_value(span: Span, ty: String) -> Self {
        Self {
            span,
            error: CompileErrorEnum::NonComptimeValue(ty),
        }
    }
}

impl Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.span.show(f, format!("{}",self.error))
    }
}
