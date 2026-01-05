use ast::{Type, TypeEnum};

use crate::info::TypeKind;

pub(crate) fn get_llvm_type(ty: &Type) -> TypeKind<'static> {
    let mut type_ = match ty.kind {
        TypeEnum::Signed(width) | TypeEnum::Unsigned(width) => TypeKind::new_int(width),
        TypeEnum::Unit => TypeKind::new_unit(),
    };
    for _ in 0..ty.ref_count {
        type_ = type_.new_ptr();
    }
    type_
}
