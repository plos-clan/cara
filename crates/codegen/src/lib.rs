use std::{
    path::Path,
    sync::{Arc, Mutex},
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
use query::{DefId, ProviderId, Providers, QueryContext};

use crate::{
    info::{SymbolStack, Value},
    queries::codegen_provider,
};

mod defs;
mod expr;
mod info;
mod program;
mod queries;
mod types;

pub fn init() {
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

    let ctx = Context::create();
    let generator = Arc::new(Generator::new(&ctx, comp_unit));
    generator.query("main").unwrap();

    let module = generator.module.lock().unwrap();

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

struct Generator<'g> {
    ctx: &'g Context,
    module: Mutex<Module<'g>>,
    queries: QueryContext<'g>,
    providers: Providers<(Arc<Self>, DefId), Value<'g>>,
    codegen_provider: ProviderId,
}

unsafe impl Send for Generator<'_> {}
unsafe impl Sync for Generator<'_> {}

impl PartialEq for Generator<'_> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl Eq for Generator<'_> {}

impl PartialOrd for Generator<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Generator<'_> {
    fn cmp(&self, _other: &Self) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
    }
}

impl<'g> Generator<'g> {
    fn new(ctx: &'g Context, comp_unit: &'g CompUnit) -> Self {
        let module = Mutex::new(ctx.create_module("main"));
        let queries = QueryContext::new(comp_unit);
        let mut providers = Providers::new();
        let codegen_provider = providers.register(Box::new(codegen_provider));
        Generator {
            ctx,
            module,
            queries,
            providers,
            codegen_provider,
        }
    }

    pub fn query(self: &Arc<Self>, name: &str) -> Option<Value<'g>> {
        let id = self.queries.lookup_def_id(name)?;
        self.queries
            .query_cached(&self.providers, self.codegen_provider, (self.clone(), id))
    }
}

struct VisitorCtx<'v> {
    builder: Builder<'v>,
    symbols: SymbolStack<'v>,
    generator: Arc<Generator<'v>>,
}
