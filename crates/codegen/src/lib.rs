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

#[derive(Debug, Clone, Copy)]
pub enum CodeModel {
    Large,
    Medium,
    Small,
    Kernel,
    Default,
}

#[derive(Debug, Clone, Copy)]
pub enum OptimizeLevel {
    O0,
    O1,
    O2,
    O3,
}

#[derive(Debug, Clone, Copy)]
pub enum RelocMode {
    Default,
    Static,
    Pic,
    DynamicNoPic,
}

#[derive(Builder, Clone, Copy)]
pub struct BackendOptions {
    pub code_model: CodeModel,
    pub optimize_level: OptimizeLevel,
    pub reloc_mode: RelocMode,
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

pub trait CodegenBackendBase {
    fn new(backend_options: BackendOptions) -> Self;
}

pub trait CodegenBackend {
    fn codegen(
        &self,
        ctx: Arc<QueryContext<'_>>,
        codegen_units: Vec<DefId>,
    ) -> Box<dyn CodegenResult>;
}

pub fn codegen(ctx: Arc<QueryContext<'_>>, backend: &dyn CodegenBackend) -> Box<dyn CodegenResult> {
    let codegen_units = ctx.query_cached(&COLLECT_CODEGEN_UNITS, ()).unwrap();

    backend.codegen(ctx, codegen_units)
}
