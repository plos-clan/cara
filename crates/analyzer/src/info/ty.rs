use std::{collections::HashMap, fmt::Display};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Signed(u32),
    Unsigned(u32),

    #[default]
    Unit,
    Bool,

    Ptr(Box<Self>),
    Array(Box<Self>, u32),
    Function(Box<Self>, Vec<Self>),
    Structure(HashMap<String, Type>),
}

impl Type {
    pub fn is_int(&self) -> bool {
        matches!(self, Self::Signed(_)) || matches!(self, Self::Unsigned(_))
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool)
    }

    pub fn is_unit(&self) -> bool {
        matches!(self, Self::Unit)
    }

    pub fn is_ptr(&self) -> bool {
        matches!(self, Self::Ptr(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_, _))
    }

    pub fn is_function(&self) -> bool {
        matches!(self, Self::Function(_, _))
    }

    pub fn is_structure(&self) -> bool {
        matches!(self, Self::Structure(_))
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Signed(bits) => write!(f, "i{}", bits),
            Self::Unsigned(bits) => write!(f, "u{}", bits),
            Self::Bool => write!(f, "bool"),
            Self::Unit => write!(f, "()"),

            Self::Ptr(ty) => write!(f, "*{}", ty),
            Self::Array(ty, len) => write!(f, "[{}; {}]", ty, len),
            Self::Function(ret_ty, param_types) => {
                write!(f, "fn(")?;
                for ty in param_types {
                    write!(f, "{}, ", ty)?;
                }
                write!(f, ") -> ")?;
                write!(f, "{}", ret_ty)
            }
            Self::Structure(fields) => {
                write!(f, "{{")?;
                for (name, ty) in fields {
                    write!(f, "{}: {}, ", name, ty)?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl Type {
    pub fn pointer(&self) -> Self {
        Self::Ptr(Box::new(self.clone()))
    }

    pub fn array(&self, len: u32) -> Self {
        Self::Array(Box::new(self.clone()), len)
    }
}
