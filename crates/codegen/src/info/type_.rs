use inkwell::{
    AddressSpace,
    types::{
        AnyType, AnyTypeEnum, ArrayType, AsTypeRef, BasicMetadataTypeEnum, BasicType,
        BasicTypeEnum, FunctionType, IntType, PointerType, VoidType,
    },
    values::BasicValue,
};

use crate::{LLVM_CONTEXT, info::Value};

#[derive(Debug, Clone)]
pub enum TypeKind<'t> {
    Unit(VoidType<'t>),
    Function(FunctionType<'t>),
    Int(IntType<'t>),
    Array {
        ty: ArrayType<'t>,
        element: Box<Self>,
    },
    Ptr {
        ty: PointerType<'t>,
        pointee: Box<Self>,
    },
}

impl<'t> TypeKind<'t> {
    pub fn new_unit() -> Self {
        TypeKind::Unit(LLVM_CONTEXT.void_type())
    }

    pub fn new_int(width: u32) -> Self {
        TypeKind::Int(LLVM_CONTEXT.custom_width_int_type(width))
    }

    pub fn new_ptr(&self) -> Self {
        TypeKind::Ptr {
            ty: LLVM_CONTEXT.ptr_type(AddressSpace::default()),
            pointee: Box::new(self.clone()),
        }
    }

    pub fn new_array(&self, size: u32) -> Self {
        if matches!(self, TypeKind::Unit(_)) {
            return Self::new_unit();
        }
        TypeKind::Array {
            ty: self.array_type(size),
            element: Box::new(self.clone()),
        }
    }

    pub fn derefed(&self) -> Self {
        match self {
            TypeKind::Ptr { pointee, .. } => pointee.as_ref().clone(),
            TypeKind::Array { ty: _, element } => element.as_ref().clone(),
            _ => panic!("Cannot dereference non-pointer type"),
        }
    }
}

impl<'t> TypeKind<'t> {
    pub fn function(&self, arg_types: Vec<Self>) -> Self {
        let arg_types = arg_types
            .into_iter()
            .map(BasicMetadataTypeEnum::from)
            .collect::<Vec<_>>();
        match self {
            TypeKind::Unit(void_type) => TypeKind::Function(void_type.fn_type(&arg_types, false)),
            TypeKind::Int(int_type) => TypeKind::Function(int_type.fn_type(&arg_types, false)),
            TypeKind::Array { ty, element: _ } => TypeKind::Function(ty.fn_type(&arg_types, false)),
            _ => unreachable!(),
        }
    }

    pub fn as_function_type(&self) -> FunctionType<'t> {
        match self {
            TypeKind::Function(function_type) => *function_type,
            _ => panic!("Incorrect usage of type."),
        }
    }

    pub fn as_array_type(&self) -> ArrayType<'t> {
        match self {
            TypeKind::Array { ty, element: _ } => *ty,
            _ => panic!("Incorrect usage of type."),
        }
    }

    pub fn const_array(&self, values: &[Value<'t>]) -> Value<'t> {
        let value_iter = values.iter().map(|v| v.as_basic_value_enum());
        let value = match self {
            TypeKind::Int(ty) => {
                ty.const_array(&value_iter.map(|v| v.into_int_value()).collect::<Vec<_>>())
            }
            TypeKind::Array { ty, element: _ } => {
                ty.const_array(&value_iter.map(|v| v.into_array_value()).collect::<Vec<_>>())
            }
            TypeKind::Ptr { ty, pointee: _ } => ty.const_array(
                &value_iter
                    .map(|v| v.into_pointer_value())
                    .collect::<Vec<_>>(),
            ),
            TypeKind::Unit(_) => return Value::Unit,
            _ => panic!("Incorrect usage of type."),
        };
        Value::Array {
            value,
            ty: self.new_array(values.len() as u32),
        }
    }
}

impl<'t> From<TypeKind<'t>> for BasicTypeEnum<'t> {
    fn from(value: TypeKind<'t>) -> Self {
        match value {
            TypeKind::Int(int_type) => int_type.into(),
            TypeKind::Ptr { ty, pointee: _ } => ty.into(),
            TypeKind::Array { ty, element: _ } => ty.into(),
            _ => unreachable!(),
        }
    }
}

impl<'t> From<TypeKind<'t>> for BasicMetadataTypeEnum<'t> {
    fn from(value: TypeKind<'t>) -> Self {
        match value {
            TypeKind::Int(int_type) => int_type.into(),
            TypeKind::Ptr { ty, pointee: _ } => ty.into(),
            TypeKind::Array { ty, element: _ } => ty.into(),
            _ => unreachable!(),
        }
    }
}

impl<'t> From<TypeKind<'t>> for AnyTypeEnum<'t> {
    fn from(value: TypeKind<'t>) -> Self {
        match value {
            TypeKind::Unit(void_type) => void_type.into(),
            TypeKind::Function(func_type) => func_type.into(),
            TypeKind::Int(int_type) => int_type.into(),
            TypeKind::Ptr { ty, pointee: _ } => ty.into(),
            TypeKind::Array { ty, element: _ } => ty.into(),
        }
    }
}

unsafe impl<'t> AsTypeRef for TypeKind<'t> {
    fn as_type_ref(&self) -> inkwell::llvm_sys::prelude::LLVMTypeRef {
        AnyTypeEnum::from(self.clone()).as_type_ref()
    }
}

unsafe impl<'t> AnyType<'t> for TypeKind<'t> {}

unsafe impl<'t> BasicType<'t> for TypeKind<'t> {}
