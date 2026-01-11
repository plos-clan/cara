use std::{collections::HashMap, fmt::Display};

use crate::{Exp, GlobalItem, Span};

#[derive(Debug, Clone)]
pub enum TypeEnum {
    Signed(u32),
    Unsigned(u32),

    Array(Box<Exp>, u32),
    Structure(HashMap<String, Exp>, Vec<GlobalItem>),

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
            TypeEnum::Array(inner, len) => write!(f, "[{:?}; {}]", inner, len),
            TypeEnum::Structure(fields, _) => {
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
