use std::sync::Arc;

use ast::{Exp, Span, TypeEnum};
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

fn get_analyzer_type(ty: &ast::Type) -> Type {
    let mut result = match &ty.kind {
        TypeEnum::Signed(width) => Type::Signed(*width),
        TypeEnum::Unsigned(width) => Type::Unsigned(*width),
        TypeEnum::Array(base, len) => get_analyzer_type(base).array(*len),
        TypeEnum::Unit => Type::Unit,
    };
    for _ in 0..ty.ref_count {
        result = result.pointer();
    }
    result
}

fn try_infer(exp: &Exp) -> Option<Type> {
    match exp {
        Exp::ProtoDef(proto) => {
            let ret_ty = proto
                .return_type
                .as_ref()
                .map(get_analyzer_type)
                .unwrap_or(Type::Unit);
            let param_types = proto
                .params
                .iter()
                .map(|p| get_analyzer_type(&p.param_type))
                .collect::<Vec<_>>();
            Some(Type::Function(Box::new(ret_ty), param_types))
        }
        Exp::Function(func) => {
            let ret_ty = func
                .return_type
                .as_ref()
                .map(get_analyzer_type)
                .unwrap_or(Type::Unit);
            let param_types = func
                .params
                .iter()
                .map(|p| get_analyzer_type(&p.param_type))
                .collect::<Vec<_>>();
            Some(Type::Function(Box::new(ret_ty), param_types))
        }
        _ => None,
    }
}
