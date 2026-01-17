use std::{collections::HashMap, fmt::Display};

use crate::{ExpId, GlobalItem, Span};

#[derive(Debug, Clone)]
pub enum TypeEnum {
    Signed(u32),
    Unsigned(u32),
    Usize,
    Isize,

    Array(ExpId, u32),
    Structure(StructType),

    Unit,
}

#[derive(Debug, Clone)]
pub struct Type {
    pub kind: TypeEnum,
    pub span: Span,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            TypeEnum::Signed(bits) => write!(f, "i{}", bits),
            TypeEnum::Unsigned(bits) => write!(f, "u{}", bits),
            TypeEnum::Usize => write!(f, "usize"),
            TypeEnum::Isize => write!(f, "isize"),
            TypeEnum::Array(inner, len) => write!(f, "[{:?}; {}]", inner, len),
            TypeEnum::Structure(StructType { fields, .. }) => {
                write!(f, "{{")?;
                for (name, ty) in fields {
                    write!(f, "{}: {:?}, ", name, ty)?;
                }
                write!(f, "}}")
            }
            TypeEnum::Unit => write!(f, "()"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StructType {
    pub fields: HashMap<String, ExpId>,
    pub members: Vec<GlobalItem>,
    pub span: Span,
}
