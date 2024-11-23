/*
* @file    :   lib.rs
* @time    :   2024/03/30 10:52:20
* @author  :   zzjcarrot
*/

use crate::ast::Span;
use std::fmt;

mod log;

pub use log::*;

/// Error returned by IR generator.
#[allow(dead_code)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ErrorTypes {
    DuplicatedDef,
    SymbolNotFound,
    FailedToEval,
    InvalidArrayLen,
    InvalidInit,
    ArrayAssign,
    NotInLoop,
    RetValInVoidFunc,
    DerefInt,
    UseVoidValue,
    ArgMismatch,
    NonIntCalc,
    DeclGlobalVar,
    Terminated,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Error(pub ErrorTypes, pub Span);

impl fmt::Display for ErrorTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Terminated => Ok(()),
            Self::DuplicatedDef => write!(f, "duplicated symbol definition"),
            Self::SymbolNotFound => write!(f, "找不到符号"),
            Self::FailedToEval => write!(f, "failed to evaluate constant"),
            Self::InvalidArrayLen => write!(f, "invalid array length"),
            Self::InvalidInit => write!(f, "invalid initializer"),
            Self::ArrayAssign => write!(f, "assigning to array"),
            Self::NotInLoop => write!(f, "using break/continue outside of loop"),
            Self::RetValInVoidFunc => write!(f, "returning value in void fucntion"),
            Self::DerefInt => write!(f, "解引用非指针类型"),
            Self::UseVoidValue => write!(f, "使用void值"),
            Self::ArgMismatch => write!(f, "argument mismatch"),
            Self::NonIntCalc => write!(f, "non-integer calculation"),
            Self::DeclGlobalVar => write!(f, "定义全局变量"),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        crate::error!("{}\n{}", self.0, self.1);
        Ok(())
    }
}

/// Result type of IR generator.
pub type Result<T> = std::result::Result<T, Error>;
