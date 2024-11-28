use super::*;

mod expr;

pub trait ComptimeEvaluate<'gen> {
    type Out;
    
    fn evaluate(&self, gen: Arc<RwLock<Generator<'gen>>>) -> Result<Self::Out>;
}
