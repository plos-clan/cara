use std::fmt::Display;

use crate::Span;

#[derive(Debug, Clone, Copy)]
pub enum TypeEnum {
    Signed(u32),
    Unsigned(u32),

    Unit,
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
            TypeEnum::Signed(bits) => write!(f, "i{}", bits),
            TypeEnum::Unsigned(bits) => write!(f, "u{}", bits),
            TypeEnum::Unit => write!(f, "()"),
        }
    }
}
