use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    sync::Arc,
};

use ast::{FunctionDef, ProtoDef};
use query::QueryContext;
use symbol_table::SymbolTable;

mod block;
mod exp;
pub mod queries;
mod stmt;

#[derive(Clone)]
pub enum CodegenItem {
    Func(Arc<FunctionDef>),
    Proto(Arc<ProtoDef>),
}

impl PartialEq for CodegenItem {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CodegenItem::Func(func1), CodegenItem::Func(func2)) => Arc::ptr_eq(func1, func2),
            (CodegenItem::Proto(proto1), CodegenItem::Proto(proto2)) => Arc::ptr_eq(proto1, proto2),
            _ => false,
        }
    }
}

impl Eq for CodegenItem {}

impl Hash for CodegenItem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            CodegenItem::Func(func) => Arc::as_ptr(func).hash(state),
            CodegenItem::Proto(proto) => Arc::as_ptr(proto).hash(state),
        }
    }
}

struct MonomorphizeContext {
    ctx: Arc<QueryContext>,
    locals: SymbolTable<String>,
    required_items: HashSet<CodegenItem>,
}
