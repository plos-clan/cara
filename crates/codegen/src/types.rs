use ast::{Type, TypeEnum};
use inkwell::{AddressSpace, context::Context};

use crate::info::TypeKind;

pub(crate) fn get_llvm_type<'ctx>(ctx: &'ctx Context, ty: &Type) -> TypeKind<'ctx> {
    let mut type_ = match ty.kind {
        TypeEnum::Signed(width) | TypeEnum::Unsigned(width) => TypeKind::new_int(width),
        TypeEnum::Unit => TypeKind::new_unit(),
    };
    for _ in 0..ty.ref_count {
        type_ = TypeKind::Ptr {
            ty: ctx.ptr_type(AddressSpace::default()),
            pointee: Box::new(type_),
        };
    }
    type_
}
