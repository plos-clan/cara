use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex, RwLock},
};

use ast::{CompUnit, ConstDef, ConstInitialValue, FunctionDef};
use inkwell::{
    OptimizationLevel,
    builder::Builder,
    context::Context,
    module::Module,
    passes::PassBuilderOptions,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
};
use query::{ProviderId, QuerySystem};

use crate::info::{ConstDefTable, SymbolStack, Value};

mod defs;
mod expr;
mod info;
mod program;
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
    queries: QuerySystem<(Arc<Self>, ConstDef), Value<'g>>,
    codegen_provider: ProviderId,
    table: ConstDefTable<'g>,
    globals: RwLock<BTreeMap<String, Value<'g>>>,
}

unsafe impl Send for Generator<'_> {}
unsafe impl Sync for Generator<'_> {}

impl<'g> Generator<'g> {
    fn new(ctx: &'g Context, comp_unit: &'g CompUnit) -> Self {
        let module = Mutex::new(ctx.create_module("main"));
        let mut queries = QuerySystem::new();
        let codegen_provider = queries.register_provider(Box::new(codegen_provider));
        Generator {
            ctx,
            module,
            queries,
            codegen_provider,
            table: ConstDefTable::new(comp_unit),
            globals: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn query(self: &Arc<Self>, name: &str) -> Option<Value<'g>> {
        let def = self.table.get(name)?;
        Some(
            self.queries
                .query(self.codegen_provider, (self.clone(), def)),
        )
    }
}

fn codegen_provider<'g>(arg: (Arc<Generator<'g>>, ConstDef)) -> Value<'g> {
    let (
        generator,
        ConstDef {
            name,
            initial_value,
            span: _,
        },
    ) = arg;

    match initial_value {
        ConstInitialValue::Function(FunctionDef {
            params,
            return_type,
            block,
            span: _,
        }) => {
            let mut param_types = Vec::new();
            for param in params {
                param_types.push(generator.visit_type(&param.param_type));
            }

            let return_type = generator.visit_type(&return_type);
            let function_type = return_type.function(param_types);
            let function = generator.module.lock().unwrap().add_function(
                &name,
                function_type.as_function_type(),
                None,
            );
            function.set_call_conventions(0); // C
            let result = Value::Function(function, return_type);
            generator
                .globals
                .write()
                .unwrap()
                .insert(name, result.clone());

            let entry_bb = generator.ctx.append_basic_block(function, "entry");
            let builder_creator = || {
                let builder = generator.ctx.create_builder();
                builder.position_at_end(entry_bb);
                builder
            };

            if let Some(value) = generator.visit_block(
                &mut VisitorCtx {
                    builder: builder_creator(),
                    symbols: SymbolStack::new(),
                },
                &block,
            ) {
                builder_creator().build_return(Some(&value)).unwrap();
            }

            result
        }
        ConstInitialValue::Exp(_) => unimplemented!(),
    }
}

struct VisitorCtx<'v> {
    builder: Builder<'v>,
    symbols: SymbolStack<'v>,
}
