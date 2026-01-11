use std::{collections::HashMap, sync::Arc};

use ast::{Exp, Span, TypeEnum, visitor::ExpVisitor};
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

struct AnalyzerContext<'ctx> {
    symbols: SymbolTable<Symbol>,
    ctx: Arc<QueryContext<'ctx>>,
    errors: Vec<(Error, Span)>,
    warnings: Vec<(Warning, Span)>,
    required: Vec<DefId>,
    ret_ty: Option<Type>,
}

impl<'ctx> AnalyzerContext<'ctx> {
    fn new(ctx: Arc<QueryContext<'ctx>>, ret_ty: Option<Type>) -> Self {
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

impl AnalyzerContext<'_> {
    fn try_infer(&mut self, exp: &Exp) -> Option<Type> {
        match exp {
            Exp::ProtoDef(proto) => {
                let ret_ty = proto
                    .return_type
                    .as_ref()
                    .map(|t| self.visit_type(t))
                    .unwrap_or(Type::Unit);
                let param_types = proto
                    .params
                    .iter()
                    .map(|p| self.visit_type(&p.param_type))
                    .collect::<Vec<_>>();
                Some(Type::Function(Box::new(ret_ty), param_types))
            }
            Exp::Function(func) => {
                let ret_ty = func
                    .return_type
                    .as_ref()
                    .map(|t| self.visit_type(t))
                    .unwrap_or(Type::Unit);
                let param_types = func
                    .params
                    .iter()
                    .map(|p| self.visit_type(&p.param_type))
                    .collect::<Vec<_>>();
                Some(Type::Function(Box::new(ret_ty), param_types))
            }
            _ => None,
        }
    }
}

impl AnalyzerContext<'_> {
    fn visit_type(&mut self, ty: &ast::Type) -> Type {
        let mut result = match &ty.kind {
            TypeEnum::Signed(width) => Type::Signed(*width),
            TypeEnum::Unsigned(width) => Type::Unsigned(*width),
            TypeEnum::Array(base, len) => self.visit_type(base).array(*len),
            TypeEnum::Unit => Type::Unit,
            TypeEnum::Structure(struct_ty) => {
                let mut fields = HashMap::new();
                for (name, ty) in struct_ty.iter() {
                    fields.insert(name.clone(), self.visit_type(ty));
                }
                Type::Structure(fields)
            }
            TypeEnum::Custom(var) => self.visit_var(var).into_type(),
        };
        for _ in 0..ty.ref_count {
            result = result.pointer();
        }
        result
    }
}
