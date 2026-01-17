use std::sync::Arc;

use query::{DefId, QueryContext};
use symbol_table::SymbolTable;

mod block;
mod exp;
pub mod queries;
mod stmt;

struct MonomorphizeContext {
    ctx: Arc<QueryContext>,
    locals: SymbolTable<String>,
    required_items: Vec<DefId>,
}
