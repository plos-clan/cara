use std::{collections::HashMap, sync::Arc};

use ast::{Type, TypeEnum};
use const_eval::queries::CONST_EVAL_PROVIDER;
use inkwell::types::BasicType;
use query::QueryContext;

use crate::{LLVM_CONTEXT, VisitorCtx, info::TypeKind};

pub(crate) fn get_llvm_type(ctx: Arc<QueryContext<'_>>, ty: &Type) -> TypeKind<'static> {
    let mut type_ = match &ty.kind {
        TypeEnum::Signed(width) | TypeEnum::Unsigned(width) => TypeKind::new_int(*width),
        TypeEnum::Array(inner, len) => get_llvm_type(ctx.clone(), inner).new_array(*len),
        TypeEnum::Unit => TypeKind::new_unit(),
        TypeEnum::Structure(struct_ty) => {
            let field_ids = struct_ty
                .keys()
                .cloned()
                .enumerate()
                .collect::<HashMap<_, _>>();
            let field_types = struct_ty
                .values()
                .map(|ty| Box::new(get_llvm_type(ctx.clone(), ty)))
                .collect::<Vec<_>>();
            let fields = field_types
                .iter()
                .map(|ty| ty.as_basic_type_enum())
                .collect::<Vec<_>>();
            let ty = LLVM_CONTEXT.struct_type(&fields, false);

            TypeKind::Structure {
                ty,
                field_ids,
                field_types,
            }
        }
        TypeEnum::Custom(var) => {
            let name = var.path.path.join("::");
            let def_id = ctx.lookup_def_id(name).unwrap();

            let value = ctx
                .query_cached(&CONST_EVAL_PROVIDER, def_id)
                .unwrap()
                .kind();

            match value {
                const_eval::ValueKind::Type(ty) => get_llvm_type(ctx.clone(), &ty),
                _ => panic!("Invalid type"),
            }
        }
    };
    for _ in 0..ty.ref_count {
        type_ = type_.new_ptr();
    }
    type_
}

impl<'v> VisitorCtx<'v> {}
