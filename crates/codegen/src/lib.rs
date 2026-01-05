use std::{
    collections::BTreeMap,
    ops::Deref,
    path::Path,
    sync::{Arc, LazyLock},
};

use ast::{FunctionDef, visitor::BlockVisitor};
use const_eval::queries::CONST_EVAL_PROVIDER;
use inkwell::{
    OptimizationLevel,
    builder::Builder,
    context::Context,
    module::Module,
    passes::PassBuilderOptions,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    values::AnyValue,
};
use monomorphize::queries::COLLECT_CODEGEN_UNITS;
use query::{DefId, QueryContext};
use uuid::Uuid;

use crate::{
    info::{Symbol, SymbolStack, TypeKind, Value},
    types::get_llvm_type,
};

mod defs;
mod expr;
mod info;
mod program;
mod types;

struct LLVMContext(Context);

unsafe impl Send for LLVMContext {}
unsafe impl Sync for LLVMContext {}

impl Deref for LLVMContext {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

static LLVM_CONTEXT: LazyLock<LLVMContext> = LazyLock::new(|| LLVMContext(Context::create()));

pub fn init() {
    LazyLock::force(&LLVM_CONTEXT);
    Target::initialize_all(&InitializationConfig::default());
}

pub fn codegen(ctx: Arc<QueryContext<'_>>) {
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

    let mut global_funcs = BTreeMap::new();
    let module = LLVM_CONTEXT.create_module("main");

    let codegen_units = ctx.query(&COLLECT_CODEGEN_UNITS, ()).unwrap();
    for unit in codegen_units.iter() {
        let const_eval::Value::Function(func) = ctx.query(&CONST_EVAL_PROVIDER, *unit).unwrap()
        else {
            panic!("Expected function value");
        };
        let FunctionDef {
            abi,
            params,
            return_type,
            ..
        } = func.as_ref();

        let function_name = match abi {
            ast::Abi::CAbi(name) => name.clone(),
            _ => Uuid::new_v4().to_string(),
        };

        let mut param_types = Vec::new();
        for param in params {
            param_types.push(get_llvm_type(&LLVM_CONTEXT, &param.param_type));
        }

        let return_type = get_llvm_type(&LLVM_CONTEXT, return_type);
        let function_type = return_type.function(param_types);
        let function = module.add_function(&function_name, function_type.as_function_type(), None);
        function.set_call_conventions(0); // C

        global_funcs.insert(*unit, Value::Function(function, return_type));
    }

    let global_funcs = Arc::new(global_funcs);
    let module = Arc::new(module);

    for def_id in codegen_units {
        let const_eval::Value::Function(func) = ctx.query(&CONST_EVAL_PROVIDER, def_id).unwrap()
        else {
            panic!("Expected function value");
        };
        let FunctionDef { params, block, .. } = func.as_ref();

        let func_value = global_funcs.get(&def_id).cloned().unwrap();
        let function = func_value.as_fn();
        let entry_block = LLVM_CONTEXT.append_basic_block(function, "entry");

        let builder = LLVM_CONTEXT.create_builder();
        builder.position_at_end(entry_block);

        let mut ctx = VisitorCtx {
            builder,
            symbols: SymbolStack::new(),
            module: module.clone(),
            queries: ctx.clone(),
            current_fn: func_value,
            global_funcs: global_funcs.clone(),
        };

        for (id, param) in params.iter().enumerate() {
            let ty = get_llvm_type(&LLVM_CONTEXT, &param.param_type);

            ctx.symbols.pre_push(Symbol::ImmutableVar(
                param.name.clone(),
                Value::new_from(
                    function
                        .get_nth_param(id as u32)
                        .unwrap()
                        .as_any_value_enum(),
                    ty,
                ),
            ));
        }

        if let Some(value) = ctx.visit_block(block)
            && !matches!(value, Value::Unit)
        {
            ctx.builder.build_return(Some(&value)).unwrap();
        }
        if ctx
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            ctx.builder.build_return(None).unwrap();
        }
    }

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
    #[allow(unused)]
    module: Arc<Module<'static>>,
    queries: Arc<QueryContext<'v>>,
    current_fn: Value<'v>,
    global_funcs: Arc<BTreeMap<DefId, Value<'v>>>,
}

impl<'v> VisitorCtx<'v> {
    fn create_entry_bb_alloca(&self, name: &str, ty: TypeKind<'v>) -> Value<'v> {
        let builder = LLVM_CONTEXT.create_builder();

        let entry_bb = self.current_fn.as_fn().get_first_basic_block().unwrap();

        match entry_bb.get_first_instruction() {
            Some(first_ins) => {
                builder.position_before(&first_ins);
            }
            None => {
                builder.position_at_end(entry_bb);
            }
        }

        Value::Alloca {
            value: builder.build_alloca(ty.clone(), name).unwrap(),
            value_ty: ty,
        }
    }
}
