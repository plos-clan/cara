use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Type {
    Signed(u8),
    Unsigned(u8),

    Bool,

    Ptr(Box<Type>),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Signed(bits) => write!(f, "i{}", bits),
            Type::Unsigned(bits) => write!(f, "u{}", bits),
            Type::Bool => write!(f, "bool"),

            Type::Ptr(ty) => write!(f, "*{}", ty),
        }
    }
}

impl Type {
    pub fn pointer(&self) -> Type {
        Type::Ptr(Box::new(self.clone()))
    }
}
