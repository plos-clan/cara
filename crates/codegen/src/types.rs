use ast::{Type, TypeEnum};
use inkwell::{AddressSpace, context::Context};

use crate::info::TypeKind;

pub(crate) fn get_llvm_type<'ctx>(ctx: &'ctx Context, ty: &Type) -> TypeKind<'ctx> {
    let mut type_ = match ty.kind {
        TypeEnum::I8 | TypeEnum::U8 => TypeKind::new_int(8),
        TypeEnum::I16 | TypeEnum::U16 => TypeKind::new_int(16),
        TypeEnum::I32 | TypeEnum::U32 => TypeKind::new_int(32),
        TypeEnum::I64 | TypeEnum::U64 => TypeKind::new_int(64),
        TypeEnum::Void => TypeKind::new_void(),
    };
    for _ in 0..ty.ref_count {
        type_ = TypeKind::Ptr {
            ty: ctx.ptr_type(AddressSpace::default()),
            pointee: Box::new(type_),
        };
    }
    type_
}
