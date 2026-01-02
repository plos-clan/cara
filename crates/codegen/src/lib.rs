use std::{
    ops::Deref,
    path::Path,
    sync::{Arc, LazyLock},
};

use ast::CompUnit;
use inkwell::{
    OptimizationLevel,
    builder::Builder,
    context::Context,
    module::Module,
    passes::PassBuilderOptions,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
};
use query::QueryContext;
use send_wrapper::SendWrapper;

use crate::{
    info::SymbolStack,
    queries::{CODEGEN_PROVIDER, CodegenResult},
};

mod defs;
mod expr;
mod info;
mod program;
pub mod queries;
mod types;

struct LLVMContext(SendWrapper<Context>);

impl Deref for LLVMContext {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

static LLVM_CONTEXT: LazyLock<LLVMContext> =
    LazyLock::new(|| LLVMContext(SendWrapper::new(Context::create())));

pub fn init() {
    LazyLock::force(&LLVM_CONTEXT);
    Target::initialize_all(&InitializationConfig::default());
}

pub fn codegen(comp_unit: &CompUnit) {
    let target_triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&target_triple).unwrap();
    let target_machine = target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            OptimizationLevel::Aggressive,
            RelocMode::Default,
            CodeModel::Default,
        )
        .unwrap();

    let queries = QueryContext::new(comp_unit);

    let main = queries.lookup_def_id("main").unwrap();
    println!("OK");
    let CodegenResult { module, .. } = queries
        .query_cached(&CODEGEN_PROVIDER, main)
        .unwrap()
        .take();

    println!("OK");

    let passes: &[&str] = &[
        "instcombine",
        "reassociate",
        "gvn",
        "simplifycfg",
        "mem2reg",
        "dce",
        "dse",
    ];

    let options = PassBuilderOptions::create();
    options.set_verify_each(true);

    module
        .run_passes(passes.join(",").as_str(), &target_machine, options)
        .unwrap();

    module.print_to_stderr();

    target_machine
        .write_to_file(&module, FileType::Object, Path::new("test.o"))
        .unwrap();

    target_machine
        .write_to_file(&module, FileType::Assembly, Path::new("test.asm"))
        .unwrap();
}

struct VisitorCtx<'v> {
    builder: Builder<'v>,
    symbols: SymbolStack<'v>,
    module: Module<'static>,
    queries: Arc<QueryContext<'v>>,
}
