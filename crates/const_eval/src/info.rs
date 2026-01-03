use std::sync::Arc;

use ast::FunctionDef;

pub enum Value {
    Int(i64),
    Function(Arc<FunctionDef>),
    Unit,
}
