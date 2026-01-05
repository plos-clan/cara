use std::fmt::Display;

use crate::Span;

#[derive(Debug, Clone)]
pub enum TypeEnum {
    Signed(u32),
    Unsigned(u32),
    
    Array(Box<Type>, u32),

    Unit,
}

#[derive(Debug, Clone)]
pub struct Type {
    pub kind: TypeEnum,
    pub ref_count: usize,
    pub span: Span,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.ref_count {
            write!(f, "*")?;
        }
        match &self.kind {
            TypeEnum::Signed(bits) => write!(f, "i{}", bits),
            TypeEnum::Unsigned(bits) => write!(f, "u{}", bits),
            TypeEnum::Array(inner, len) => write!(f, "[{}; {}]", inner, len),
            TypeEnum::Unit => write!(f, "()"),
        }
    }
}
