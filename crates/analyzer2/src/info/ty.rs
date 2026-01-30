use std::{collections::HashMap, fmt::Display, sync::Arc};

#[derive(Debug, Default)]
pub enum Type {
    Int(bool, u64),
    Size(bool),
    Bool,

    #[default]
    Unit,

    Type(Arc<Self>),
    Ptr(Arc<Self>),
    Array(Arc<Self>, u64),
    Structure {
        fields: HashMap<String, Arc<Self>>,
        members: HashMap<String, Arc<Self>>,
    },
    Function(Arc<Self>, Vec<Arc<Self>>),
}

impl Type {
    pub fn int(signed: bool, width: u64) -> Arc<Self> {
        Arc::new(Self::Int(signed, width))
    }

    pub fn size(signed: bool) -> Arc<Self> {
        Arc::new(Self::Size(signed))
    }

    pub fn bool() -> Arc<Self> {
        Arc::new(Self::Bool)
    }

    pub fn unit() -> Arc<Self> {
        Arc::new(Self::Unit)
    }

    pub fn structure(
        fields: HashMap<String, Arc<Self>>,
        members: HashMap<String, Arc<Self>>,
    ) -> Arc<Self> {
        Arc::new(Self::Structure { fields, members })
    }

    pub fn to_type(self: &Arc<Self>) -> Arc<Self> {
        Arc::new(Self::Type(self.clone()))
    }

    pub fn ptr(self: &Arc<Self>) -> Arc<Self> {
        Arc::new(Self::Ptr(self.clone()))
    }

    pub fn array(self: &Arc<Self>, len: u64) -> Arc<Self> {
        Arc::new(Self::Array(self.clone(), len))
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(signed, width) => write!(f, "{}{}", if *signed { "i" } else { "u" }, width),
            Self::Size(signed) => write!(f, "{}size", if *signed { "i" } else { "u" }),
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
            Self::Structure { fields, .. } => {
                write!(f, "{{")?;
                for (name, ty) in fields {
                    write!(f, "{}: {}, ", name, ty)?;
                }
                write!(f, "}}")
            }
            Self::Type(ty) => write!(f, "type({})", ty),
        }
    }
}
