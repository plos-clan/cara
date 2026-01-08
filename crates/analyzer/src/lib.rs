pub use diagnostic::*;
pub use info::*;

mod diagnostic;
mod info;

pub struct AnalyzerContext {}

impl AnalyzerContext {
    pub fn new() -> Self {
        Self {}
    }
}
