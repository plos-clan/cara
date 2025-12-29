use inkwell::values::{FunctionValue, IntValue, PointerValue};

use crate::info::TypeKind;

pub enum Value<'t> {
    Int(IntValue<'t>),
    Function(FunctionValue<'t>),
    Pointer {
        value: PointerValue<'t>,
        pointee: TypeKind<'t>,
    },
    Void,
}

unsafe impl Send for Value<'_> {}
unsafe impl Sync for Value<'_> {}
