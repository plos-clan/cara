use std::sync::Arc;

use crate::{Error, info::Type};

macro_rules! check_op_impl {
    ({
        $([$name:ident, $op:literal]),+ $(,)?
    } $pat:tt) => {
        $(check_op_impl!(@impl $pat $name $op);)+
    };
    (@impl {$($pat:pat => $e:expr),+ $(,)?} $name:ident $op:literal) => {
        pub fn $name(self: &Arc<Self>, other: &Arc<Self>) -> Result<Arc<Self>, Error> {
            match (self.as_ref(), other.as_ref()) {
                $($pat => Ok($e),)+
                _ => Err(Error::UnsupportedOperator($op.into(), self.clone())),
            }
        }
    };

    (#unary {
        $([$name:ident, $op:literal]),+ $(,)?
    } $pat:tt) => {
        $(check_op_impl!(@impl_unary $pat $name $op);)+
    };
    (@impl_unary {$($pat:pat => $e:expr),+ $(,)?} $name:ident $op:literal) => {
        pub fn $name(self: &Arc<Self>) -> Result<Arc<Self>, Error> {
            match self.as_ref() {
                $($pat => Ok($e),)+
                _ => Err(Error::UnsupportedOperator($op.into(), self.clone())),
            }
        }
    };
}

impl Type {
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
            (Type::Int(signed_a, a), Type::Int(signed_b, b)) => Type::int(*signed_a || *signed_b, std::cmp::max(*a, *b)),
            (Type::Size(a), Type::Size(b)) => Type::size(*a || *b),
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
            (Type::Int(_, _), Type::Int(_, _)) => Type::bool(),
            (Type::Size(_), Type::Size(_)) => Type::bool(),
            (Type::Bool, Type::Bool) => Type::bool(),
        }
    );

    check_op_impl!(
        {
            [check_and, "&&"],
            [check_or, "||"],
        }
        {
            (Type::Bool, Type::Bool) => Type::bool()
        }
    );

    check_op_impl!(#unary
        {
            [check_pos, "+"],
            [check_neg, "-"],
        }
        {
            Type::Int(signed, w) => Type::int(*signed, *w),
            Type::Size(a) => Type::size(*a),
        }
    );
    check_op_impl!(#unary
        {
            [check_not, "!"],
        }
        {
            Type::Bool => Type::bool(),
        }
    );
}
