use std::sync::Arc;

use ast::{CompUnit, ConstDef, ConstInitialValue, FunctionDef};
use inkwell::{builder::Builder, context::Context, module::Module};
use query::{ProviderId, QuerySystem};

use crate::info::{ConstDefTable, Value};

mod defs;
mod expr;
mod info;
mod program;
mod types;

pub fn codegen(comp_unit: &CompUnit) {
    let ctx = Context::create();
    let generator = Arc::new(Generator::new(&ctx, comp_unit));
    generator.query("main");
    generator.module.print_to_stderr();
}

struct Generator<'g> {
    ctx: &'g Context,
    module: Module<'g>,
    builder: Builder<'g>,
    queries: QuerySystem<(Arc<Self>, ConstDef), Value<'g>>,
    codegen_provider: ProviderId,
    table: ConstDefTable<'g>,
}

unsafe impl Send for Generator<'_> {}
unsafe impl Sync for Generator<'_> {}

impl<'g> Generator<'g> {
    fn new(ctx: &'g Context, comp_unit: &'g CompUnit) -> Self {
        let module = ctx.create_module("main");
        let builder = ctx.create_builder();
        let mut queries = QuerySystem::new();
        let codegen_provider = queries.register_provider(Box::new(codegen_provider));
        Generator {
            ctx,
            module,
            builder,
            queries,
            codegen_provider,
            table: ConstDefTable::new(comp_unit),
        }
    }

    pub fn query(self: &Arc<Self>, name: &str) -> Value<'g> {
        let def = self.table.get(name).unwrap();
        self.queries.query(self.codegen_provider, (self.clone(), def))
    }
}

fn codegen_provider<'g>(arg: (Arc<Generator<'g>>, ConstDef)) -> Value<'g> {
    let (generator, const_def) = arg;
    let ConstDef {
        name,
        initial_value:
            ConstInitialValue::Function(FunctionDef {
                params,
                return_type,
                ..
            }),
        span: _,
    } = const_def
    else {
        unreachable!()
    };

    let mut param_types = Vec::new();
    for param in params {
        param_types.push(generator.visit_type(&param.param_type));
    }

    let function_type = generator.visit_type(&return_type).function(param_types);
    let function = generator
        .module
        .add_function(&name, function_type.as_function_type(), None);
    function.set_call_conventions(0); // C

    Value::Function(function)
}
