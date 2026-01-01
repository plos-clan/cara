use inkwell::{
    AddressSpace,
    types::{
        AnyTypeEnum, BasicMetadataTypeEnum, BasicTypeEnum, FunctionType, IntType, PointerType,
        VoidType,
    },
};

use crate::Generator;

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

impl<'t> Generator<'t> {
    pub fn new_void(&self) -> TypeKind<'t> {
        TypeKind::Void(self.ctx.void_type())
    }

    pub fn new_int(&self, width: u32) -> TypeKind<'t> {
        TypeKind::Int(self.ctx.custom_width_int_type(width))
    }

    pub fn new_ptr(&self, pointee: TypeKind<'t>) -> TypeKind<'t> {
        TypeKind::Ptr {
            ty: self.ctx.ptr_type(AddressSpace::default()),
            pointee: Box::new(pointee),
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
