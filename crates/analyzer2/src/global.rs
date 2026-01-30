use std::sync::Arc;

use ast::{Array, AstContext, Exp, ExpId, StructType, visitor::ExpVisitor};

use crate::{GlobalContext, info::Type};

impl GlobalContext {
    pub fn new_impl(ast: AstContext) -> Self {
        Self {
            root: Self::infer_structure(&ast.root, &ast),
        }
    }
}

impl GlobalContext {
    fn infer_exp(exp: ExpId, ctx: &AstContext) -> Arc<Type> {
        let exp = ctx.exp(exp);
        match exp {
            Exp::Array(array) => Self::infer_array(array, ctx),
        }
    }

    fn infer_structure(structure: &StructType, ctx: &AstContext) -> Arc<Type> {}

    fn infer_array(array: &Array, ctx: &AstContext) -> Arc<Type> {
        match array {
            Array::List(values, _) => {
                let element_type = Self::infer_exp(values[0], ctx);
                Arc::new(Type::Array(element_type))
            }
            _ => unimplemented!(),
        }
    }
}
