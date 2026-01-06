use std::sync::Arc;

use bon::Builder;
use monomorphize::queries::COLLECT_CODEGEN_UNITS;
use query::{DefId, QueryContext};

#[derive(Debug, Clone, Copy)]
pub enum OutputType {
    Ir,
    Asm,
    Object,
}

#[derive(Builder)]
pub struct EmitOptions {
    pub output_type: OutputType,
    pub path: String,
}

pub trait CodegenResult {
    fn dump(&self);
    fn optimize(&self);
    fn emit(&self, options: EmitOptions);
}

pub trait CodegenBackend {
    fn init(&self);
    fn codegen(
        &self,
        ctx: Arc<QueryContext<'_>>,
        codegen_units: Vec<DefId>,
    ) -> Box<dyn CodegenResult>;
}

pub fn codegen(ctx: Arc<QueryContext<'_>>, backend: &dyn CodegenBackend) -> Box<dyn CodegenResult> {
    backend.init();

    let codegen_units = ctx.query(&COLLECT_CODEGEN_UNITS, ()).unwrap();

    backend.codegen(ctx, codegen_units)
}
