use std::sync::Arc;

use ast::{AstContext, Span};
pub use lints::*;
use symbol_table::SymbolTable;

use crate::info::{Symbol, Type};

mod global;
pub mod info;
mod lints;

struct GlobalContext {
    root: Arc<Type>,
}

impl GlobalContext {
    fn new_from(ast: AstContext) -> Self {}
}

struct LocalContext {
    symbols: SymbolTable<Symbol>,
    errors: Vec<(Error, Span)>,
    warnings: Vec<(Warning, Span)>,
    ret_ty: Option<Type>,
    in_loop: bool,
}

impl LocalContext {
    fn new(ret_ty: Option<Type>) -> Self {
        Self {
            symbols: SymbolTable::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            ret_ty,
            in_loop: false,
        }
    }

    fn error_at(&mut self, e: Error, s: Span) {
        self.errors.push((e, s));
    }

    #[allow(unused)]
    fn warning_at(&mut self, w: Warning, s: Span) {
        self.warnings.push((w, s));
    }

    fn toggle_in_loop(&mut self) {
        self.in_loop = !self.in_loop;
    }
}
