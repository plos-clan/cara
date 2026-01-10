use crate::{Error, Type};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Value {
    type_: Type,
}

impl Value {
    pub fn new(type_: Type) -> Self {
        Value { type_ }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn into_type(self) -> Type {
        self.type_
    }
}

macro_rules! check_op_impl {
    ($name: ident, $op: expr, $($pat: pat => $ty_out:expr),*) => {
        pub fn $name(&self, other: &Self) -> Result<Self, Error> {
            match (self.type_.clone(), other.type_.clone()) {
                $(
                    $pat => Ok(Value::new($ty_out)),
                )*
                _ => Err(Error::TypeMismatch(self.type_.clone(), other.type_.clone())),
            }
        }
    };

    (#unary $name: ident, $op: expr, $($pat: pat => $ty_out:expr),*) => {
        pub fn $name(&self) -> Result<Self, Error> {
            match self.type_ {
                $(
                    $pat => Ok(Value::new($ty_out)),
                )*
                _ => Err(Error::UnsupportedOperator($op.into(), self.type_.clone())),
            }
        }
    };
}

impl Value {
    check_op_impl!(check_add, "+",
        (Type::Signed(a), Type::Signed(b)) => Type::Signed(std::cmp::max(a, b))
    );
    check_op_impl!(check_sub, "-",
        (Type::Signed(a), Type::Signed(b)) => Type::Signed(std::cmp::max(a, b))
    );
    check_op_impl!(check_mul, "*",
        (Type::Signed(a), Type::Signed(b)) => Type::Signed(std::cmp::max(a, b))
    );
    check_op_impl!(check_div, "/",
        (Type::Signed(a), Type::Signed(b)) => Type::Signed(std::cmp::max(a, b))
    );
    check_op_impl!(check_mod, "%",
        (Type::Signed(a), Type::Signed(b)) => Type::Signed(std::cmp::max(a, b))
    );

    check_op_impl!(check_lshift, "<<",
        (Type::Signed(a), Type::Signed(b)) => Type::Signed(std::cmp::max(a, b))
    );
    check_op_impl!(check_rshift, ">>",
        (Type::Signed(a), Type::Signed(b)) => Type::Signed(std::cmp::max(a, b))
    );

    check_op_impl!(check_eq, "==",
        (Type::Signed(_), Type::Signed(_)) => Type::Bool,
        (Type::Bool, Type::Bool) => Type::Bool
    );
    check_op_impl!(check_neq, "!=",
        (Type::Signed(_), Type::Signed(_)) => Type::Bool,
        (Type::Bool, Type::Bool) => Type::Bool
    );

    check_op_impl!(check_gt, ">",
        (Type::Signed(_), Type::Signed(_)) => Type::Bool
    );
    check_op_impl!(check_lt, "<",
        (Type::Signed(_), Type::Signed(_)) => Type::Bool
    );
    check_op_impl!(check_ge, ">=",
        (Type::Signed(_), Type::Signed(_)) => Type::Bool
    );
    check_op_impl!(check_le, "<=",
        (Type::Signed(_), Type::Signed(_)) => Type::Bool
    );

    check_op_impl!(check_and, "&&",
        (Type::Bool, Type::Bool) => Type::Bool
    );
    check_op_impl!(check_or, "||",
        (Type::Bool, Type::Bool) => Type::Bool
    );

    check_op_impl!(#unary check_pos, "+",
        Type::Signed(a) => Type::Signed(a)
    );
    check_op_impl!(#unary check_neg, "-",
        Type::Signed(a) => Type::Signed(a)
    );
    check_op_impl!(#unary check_not, "!",
        Type::Bool => Type::Bool
    );
}
