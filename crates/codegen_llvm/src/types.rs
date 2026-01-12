use std::{collections::HashMap, sync::Arc};

use ast::{Exp, Type, TypeEnum, UnaryOp};
use const_eval::queries::CONST_EVAL_PROVIDER;
use inkwell::types::BasicType;
use query::QueryContext;

use crate::{LLVM_CONTEXT, VisitorCtx, info::TypeKind};

pub(crate) fn const_eval_type_to_llvm_type(
    ctx: Arc<QueryContext<'_>>,
    ty: &Arc<const_eval::TypeKind>,
) -> TypeKind<'static> {
    match ty.as_ref() {
        const_eval::TypeKind::Primary(primary) => get_llvm_type(ctx, primary),
        const_eval::TypeKind::Ptr(primary) => const_eval_type_to_llvm_type(ctx, primary).new_ptr(),
    }
}

pub(crate) fn get_llvm_type_from_exp(ctx: Arc<QueryContext<'_>>, ty: &Exp) -> TypeKind<'static> {
    match ty {
        Exp::Type(ty) => get_llvm_type(ctx.clone(), ty),
        Exp::Var(var) => {
            let name = var.path.path.join("::");
            let def_id = ctx.lookup_def_id(&name).unwrap();

            let value = ctx
                .query_cached(&CONST_EVAL_PROVIDER, def_id)
                .unwrap()
                .kind();

            match value {
                const_eval::ValueKind::Type(ty) => const_eval_type_to_llvm_type(ctx.clone(), &ty),
                _ => panic!("Invalid type {} {:?}", name, value),
            }
        }
        Exp::Unary(UnaryOp::Ptr, value, _) => get_llvm_type_from_exp(ctx, value).new_ptr(),
        _ => unreachable!(),
    }
}

pub(crate) fn get_llvm_type(ctx: Arc<QueryContext<'_>>, ty: &Type) -> TypeKind<'static> {
    match &ty.kind {
        TypeEnum::Signed(width) | TypeEnum::Unsigned(width) => TypeKind::new_int(*width),
        TypeEnum::Array(inner, len) => get_llvm_type_from_exp(ctx.clone(), inner).new_array(*len),
        TypeEnum::Unit => TypeKind::new_unit(),
        TypeEnum::Structure(struct_ty, _) => {
            let field_ids = struct_ty
                .keys()
                .cloned()
                .enumerate()
                .collect::<HashMap<_, _>>();
            let field_types = struct_ty
                .values()
                .map(|ty| get_llvm_type_from_exp(ctx.clone(), ty))
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
    }
}

impl<'v> VisitorCtx<'v> {}
