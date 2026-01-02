use inkwell::{
    AddressSpace,
    types::{
        AnyType, AnyTypeEnum, AsTypeRef, BasicMetadataTypeEnum, BasicType, BasicTypeEnum,
        FunctionType, IntType, PointerType, VoidType,
    },
};

use crate::LLVM_CONTEXT;

#[derive(Debug, Clone)]
pub enum TypeKind<'t> {
    Void(VoidType<'t>),
    Function(FunctionType<'t>),
    Int(IntType<'t>),
    Ptr {
        ty: PointerType<'t>,
        pointee: Box<Self>,
    },
}

impl<'t> TypeKind<'t> {
    pub fn new_void() -> Self {
        TypeKind::Void(LLVM_CONTEXT.void_type())
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

    pub fn derefed(&self) -> Self {
        match self {
            TypeKind::Ptr { pointee, .. } => pointee.as_ref().clone(),
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
            TypeKind::Void(void_type) => TypeKind::Function(void_type.fn_type(&arg_types, false)),
            TypeKind::Int(int_type) => TypeKind::Function(int_type.fn_type(&arg_types, false)),
            _ => unreachable!(),
        }
    }

    pub fn as_function_type(&self) -> FunctionType<'t> {
        match self {
            TypeKind::Function(function_type) => *function_type,
            _ => panic!("Incorrect usage of type."),
        }
    }
}

impl<'t> From<TypeKind<'t>> for BasicTypeEnum<'t> {
    fn from(value: TypeKind<'t>) -> Self {
        match value {
            TypeKind::Int(int_type) => int_type.into(),
            TypeKind::Ptr { ty, pointee: _ } => ty.into(),
            _ => unreachable!(),
        }
    }
}

impl<'t> From<TypeKind<'t>> for BasicMetadataTypeEnum<'t> {
    fn from(value: TypeKind<'t>) -> Self {
        match value {
            TypeKind::Int(int_type) => int_type.into(),
            TypeKind::Ptr { ty, pointee: _ } => ty.into(),
            _ => unreachable!(),
        }
    }
}

impl<'t> From<TypeKind<'t>> for AnyTypeEnum<'t> {
    fn from(value: TypeKind<'t>) -> Self {
        match value {
            TypeKind::Void(void_type) => void_type.into(),
            TypeKind::Function(func_type) => func_type.into(),
            TypeKind::Int(int_type) => int_type.into(),
            TypeKind::Ptr { ty, pointee: _ } => ty.into(),
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
