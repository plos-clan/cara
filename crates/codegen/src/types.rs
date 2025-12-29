use ast::{Type, TypeEnum};

use crate::{Generator, info::TypeKind};

impl<'g> Generator<'g> {
    pub(crate) fn visit_type(&self, ty: &Type) -> TypeKind<'g> {
        let mut type_ = match ty.kind {
            TypeEnum::I8 | TypeEnum::U8 => self.new_int(8),
            TypeEnum::I16 | TypeEnum::U16 => self.new_int(16),
            TypeEnum::I32 | TypeEnum::U32 => self.new_int(32),
            TypeEnum::I64 | TypeEnum::U64 => self.new_int(64),
        };
        for _ in 0..ty.ref_count {
            type_ = self.new_ptr(type_);
        }
        type_
    }
}
