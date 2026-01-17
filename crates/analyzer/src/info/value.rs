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
    ({
        $([$name:ident, $op:literal]),+ $(,)?
    } $pat:tt) => {
        $(check_op_impl!(@impl $pat $name $op);)+
    };
    (@impl {$($pat:pat => $e:expr),+ $(,)?} $name:ident $op:literal) => {
        pub fn $name(&self, other: &Self) -> Result<Self, Error> {
            match (self.type_.clone(), other.type_.clone()) {
                $($pat => Ok(Value::new($e)),)+
                _ => Err(Error::UnsupportedOperator($op.into(), self.type_.clone())),
            }
        }
    };

    (#unary {
        $([$name:ident, $op:literal]),+ $(,)?
    } $pat:tt) => {
        $(check_op_impl!(@impl_unary $pat $name $op);)+
    };
    (@impl_unary {$($pat:pat => $e:expr),+ $(,)?} $name:ident $op:literal) => {
        pub fn $name(&self) -> Result<Self, Error> {
            match self.type_.clone() {
                $($pat => Ok(Value::new($e)),)+
                _ => Err(Error::UnsupportedOperator($op.into(), self.type_.clone())),
            }
        }
    };
}

impl Value {
    check_op_impl!(
        {
            [check_add, "+"],
            [check_sub, "-"],
            [check_mul, "*"],
            [check_div, "/"],
            [check_mod, "%"],
            [check_lshift, "<<"],
            [check_rshift, ">>"],
        }
        {
            (Type::Signed(a), Type::Signed(b)) => Type::Signed(std::cmp::max(a, b)),
            (Type::Unsigned(a), Type::Unsigned(b)) => Type::Unsigned(std::cmp::max(a, b)),
            (Type::Isize, Type::Usize) => Type::Isize,
            (Type::Usize, Type::Isize) => Type::Usize,
        }
    );
    check_op_impl!(
        {
            [check_eq, "=="],
            [check_neq, "!="],
            [check_gt, ">"],
            [check_lt, "<"],
            [check_ge, ">="],
            [check_le, "<="],
        }
        {
            (Type::Signed(_), Type::Signed(_)) => Type::Bool,
            (Type::Unsigned(_), Type::Unsigned(_)) => Type::Bool,
            (Type::Isize, Type::Usize) => Type::Bool,
            (Type::Usize, Type::Isize) => Type::Bool,
            (Type::Bool, Type::Bool) => Type::Bool,
        }
    );

    check_op_impl!(
        {
            [check_and, "&&"],
            [check_or, "||"],
        }
        {
            (Type::Bool, Type::Bool) => Type::Bool
        }
    );

    check_op_impl!(#unary
        {
            [check_pos, "+"],
            [check_neg, "-"],
        }
        {
            Type::Signed(a) => Type::Signed(a),
            Type::Unsigned(a) => Type::Unsigned(a),
            Type::Isize => Type::Isize,
            Type::Usize => Type::Usize,
        }
    );
    check_op_impl!(#unary
        {
            [check_not, "!"],
        }
        {
            Type::Bool => Type::Bool
        }
    );
}
