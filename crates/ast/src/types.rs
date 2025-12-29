use std::fmt::Display;

use crate::Span;

#[derive(Debug, Clone, Copy)]
pub enum TypeEnum {
    I8,
    I16,
    I32,
    I64,

    U8,
    U16,
    U32,
    U64,
}

#[derive(Debug, Clone)]
pub struct Type {
    pub kind: TypeEnum,
    pub ref_count: usize,
    pub span: Span,
}

impl Display for TypeEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeEnum::I8 => write!(f, "i8"),
            TypeEnum::I16 => write!(f, "i16"),
            TypeEnum::I32 => write!(f, "i32"),
            TypeEnum::I64 => write!(f, "i64"),

            TypeEnum::U8 => write!(f, "u8"),
            TypeEnum::U16 => write!(f, "u16"),
            TypeEnum::U32 => write!(f, "u32"),
            TypeEnum::U64 => write!(f, "u64"),
        }
    }
}
