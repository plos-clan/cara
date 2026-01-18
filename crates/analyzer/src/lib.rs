use std::{collections::HashMap, sync::Arc};

use ast::{Exp, ExpId, Span, StructType, TypeEnum, visitor::ExpVisitor};
pub use diagnostic::*;
pub use info::*;
use query::{DefId, QueryContext};
use symbol_table::SymbolTable;

mod diagnostic;
mod exp;
mod info;
mod program;
pub mod queries;
mod stmt;

struct AnalyzerContext {
    symbols: SymbolTable<Symbol>,
    ctx: Arc<QueryContext>,
    errors: Vec<(Error, Span)>,
    warnings: Vec<(Warning, Span)>,
    required: Vec<DefId>,
    ret_ty: Option<Type>,
}

impl AnalyzerContext {
    fn new(ctx: Arc<QueryContext>, ret_ty: Option<Type>) -> Self {
        Self {
            symbols: SymbolTable::new(),
            ctx,
            errors: Vec::new(),
            warnings: Vec::new(),
            required: Vec::new(),
            ret_ty,
        }
    }

    fn error_at(&mut self, e: Error, s: Span) {
        self.errors.push((e, s));
    }

    #[allow(unused)]
    fn warning_at(&mut self, w: Warning, s: Span) {
        self.warnings.push((w, s));
    }
}

impl AnalyzerContext {
    fn try_infer(&mut self, exp: ExpId) -> Option<Type> {
        let ast_ctx = self.ctx.ast_ctx();
        let exp_body = ast_ctx.exp(exp);
        match exp_body {
            Exp::ProtoDef(proto) => {
                let ret_ty = proto
                    .return_type
                    .as_ref()
                    .map(|t| self.visit_right_value(*t).into_type())
                    .unwrap_or(Type::Unit);
                let param_types = proto
                    .params
                    .iter()
                    .map(|p| self.visit_right_value(p.param_type).into_type())
                    .collect::<Vec<_>>();
                Some(Type::Function(Box::new(ret_ty), param_types))
            }
            Exp::Function(func) => {
                let ret_ty = func
                    .return_type
                    .as_ref()
                    .map(|t| self.visit_right_value(*t).into_type())
                    .unwrap_or(Type::Unit);
                let param_types = func
                    .params
                    .iter()
                    .map(|p| self.visit_right_value(p.param_type).into_type())
                    .collect::<Vec<_>>();
                Some(Type::Function(Box::new(ret_ty), param_types))
            }
            _ => None,
        }
    }
}

impl AnalyzerContext {
    fn convert_type(&mut self, ty: &ast::Type) -> Type {
        match &ty.kind {
            TypeEnum::Signed(width) => Type::Signed(*width),
            TypeEnum::Unsigned(width) => Type::Unsigned(*width),
            TypeEnum::Array(base, len) => self.visit_right_value(*base).into_type().array(*len),
            TypeEnum::Unit => Type::Unit,
            TypeEnum::Structure(StructType { fields, .. }) => {
                let mut new_fields = HashMap::new();
                for (name, &ty) in fields.iter() {
                    new_fields.insert(name.clone(), self.visit_right_value(ty).into_type());
                }
                Type::Structure(new_fields)
            }
            TypeEnum::Isize => Type::Isize,
            TypeEnum::Usize => Type::Usize,
        }
    }
}
