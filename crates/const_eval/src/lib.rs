use std::sync::Arc;

use query::QueryContext;

pub use info::*;

mod expr;
mod info;
pub mod queries;
mod stmt;

struct ConstEvalContext<'c> {
    ctx: Arc<QueryContext<'c>>,
}
