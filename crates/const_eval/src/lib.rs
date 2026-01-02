use std::sync::Arc;

use query::QueryContext;

mod expr;
mod info;
pub mod queries;

struct ConstEvalContext<'c> {
    ctx: Arc<QueryContext<'c>>,
}
