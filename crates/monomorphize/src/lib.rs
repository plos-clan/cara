use std::sync::Arc;

use query::{DefId, QueryContext};

mod block;
mod exp;
mod stmt;
pub mod queries;

struct MonomorphizeContext<'ctx> {
    ctx: Arc<QueryContext<'ctx>>,
    locals: Vec<Vec<String>>,
    params: Vec<String>,
    required_items: Vec<DefId>,
}

impl MonomorphizeContext<'_> {
    fn push_scope(&mut self) {
        self.locals.push(Vec::new());
    }

    fn pop_scope(&mut self) {
        self.locals.pop();
    }

    fn push_symbol(&mut self, name: String) {
        self.locals.last_mut().unwrap().push(name);
    }

    fn contains(&self, name: &String) -> bool {
        for scope in self.locals.iter().rev() {
            if scope.contains(name) {
                return true;
            }
        }
        self.params.contains(name)
    }
}
