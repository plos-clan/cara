pub use defs::*;
pub use expr::*;
pub use program::*;
pub use types::*;

mod defs;
mod expr;
mod program;
mod types;
pub mod visitor;

#[derive(Debug, Clone, Copy)]
pub struct Span(usize, usize);

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span(start, end)
    }

    pub fn len(&self) -> usize {
        self.1 - self.0
    }

    pub fn start(&self) -> usize {
        self.0
    }

    pub fn end(&self) -> usize {
        self.1
    }
}
