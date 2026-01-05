use inkwell::{
    builder::Builder,
    values::{
        AnyValue, AnyValueEnum, ArrayValue, AsValueRef, BasicMetadataValueEnum, BasicValue,
        FunctionValue, IntValue, PointerValue,
    },
};

use crate::info::TypeKind;

#[derive(Debug, Clone)]
pub enum Value<'v> {
    Int(IntValue<'v>),
    Function(FunctionValue<'v>, TypeKind<'v>),
    Pointer {
        value: PointerValue<'v>,
        ty: TypeKind<'v>,
    },
    Array {
        value: ArrayValue<'v>,
        ty: TypeKind<'v>,
    },
    Alloca {
        value: PointerValue<'v>,
        value_ty: TypeKind<'v>,
    },
    Unit,
}

impl<'v> Value<'v> {
    pub fn as_int(&self, builder: &Builder<'v>) -> IntValue<'v> {
        let Value::Int(v) = self.as_right_value(builder) else {
            unreachable!()
        };
        v
    }

    pub fn as_fn(&self) -> FunctionValue<'v> {
        let Value::Function(f, _) = self else {
            unreachable!()
        };
        *f
    }

    pub fn get_pointer(&self) -> PointerValue<'v> {
        match self {
            Value::Pointer { value, .. } => *value,
            Value::Alloca { value, .. } => *value,
            _ => unreachable!(),
        }
    }

    pub fn as_right_value(&self, builder: &Builder<'v>) -> Self {
        match self {
            Self::Alloca { value, value_ty } => {
                let loaded = builder.build_load(value_ty.clone(), *value, "").unwrap();
                Self::new_from(loaded.as_any_value_enum(), value_ty.clone())
            }
            _ => self.clone(),
        }
    }

    pub fn convert_to_right_value(&self) -> Self {
        match self {
            Self::Alloca { value, value_ty } => Self::Pointer {
                value: *value,
                ty: value_ty.new_ptr(),
            },
            _ => self.clone(),
        }
    }
}

impl<'v> Value<'v> {
    pub fn type_(&self) -> TypeKind<'v> {
        match self {
            Value::Int(v) => TypeKind::Int(v.get_type()),
            Value::Function(f, _) => TypeKind::Function(f.get_type()),
            Value::Pointer { ty, .. } => ty.clone(),
            Value::Alloca { value_ty, .. } => value_ty.new_ptr(),
            Value::Unit => TypeKind::new_unit(),
            Value::Array { ty, .. } => ty.clone(),
        }
    }
}

impl<'v> Value<'v> {
    pub fn new_from(value: AnyValueEnum<'v>, ty: TypeKind<'v>) -> Self {
        if matches!(ty, TypeKind::Unit(_)) {
            return Value::Unit;
        }
        match value {
            AnyValueEnum::IntValue(v) => Value::Int(v),
            AnyValueEnum::PointerValue(v) => Value::Pointer {
                value: v,
                ty: ty.clone(),
            },
            AnyValueEnum::FunctionValue(v) => Value::Function(v, ty.clone()),
            AnyValueEnum::ArrayValue(v) => Value::Array {
                value: v,
                ty: ty.clone(),
            },
            _ => panic!("unexpected: {}", value),
        }
    }
}

impl<'v> From<Value<'v>> for BasicMetadataValueEnum<'v> {
    fn from(value: Value<'v>) -> Self {
        match value {
            Value::Int(v) => v.into(),
            Value::Pointer { value, .. } => value.into(),
            Value::Array { value, .. } => value.into(),
            _ => unreachable!(),
        }
    }
}

unsafe impl<'v> AsValueRef for Value<'v> {
    fn as_value_ref(&self) -> inkwell::llvm_sys::prelude::LLVMValueRef {
        match self {
            Value::Int(v) => v.as_value_ref(),
            Value::Function(v, _) => v.as_value_ref(),
            Value::Pointer { value, .. } => value.as_value_ref(),
            Value::Alloca { value, .. } => value.as_value_ref(),
            Value::Unit => unreachable!(),
            Value::Array { value, .. } => value.as_value_ref(),
        }
    }
}

impl<'v> From<Value<'v>> for PointerValue<'v> {
    fn from(value: Value<'v>) -> Self {
        match value {
            Value::Pointer { value, .. } => value,
            _ => unreachable!(),
        }
    }
}

unsafe impl<'v> AnyValue<'v> for Value<'v> {}
unsafe impl<'v> BasicValue<'v> for Value<'v> {}
