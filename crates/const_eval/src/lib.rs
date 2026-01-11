use std::sync::Arc;

use ast::Type;
use query::QueryContext;

pub use info::*;

mod expr;
mod info;
pub mod queries;
mod stmt;

struct ConstEvalContext<'c> {
    ctx: Arc<QueryContext<'c>>,
}

impl ConstEvalContext<'_> {
    fn visit_type(&self, type_: &Type) -> Value {
        Value::new_type(Arc::new(type_.clone()))
    }
}
