use std::sync::Arc;

use query::QueryContext;

pub use info::*;

mod expr;
mod info;
mod stmt;
pub mod queries;

struct ConstEvalContext<'c> {
    ctx: Arc<QueryContext<'c>>,
}
