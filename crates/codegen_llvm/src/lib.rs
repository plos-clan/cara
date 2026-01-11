use std::{
    collections::BTreeMap,
    ops::Deref,
    path::Path,
    sync::{Arc, LazyLock},
};

use ast::{FunctionDef, Param, ProtoDef, Type, visitor::BlockVisitor};
use codegen::{
    BackendOptions, CodegenBackend, CodegenBackendBase, CodegenResult, EmitOptions, OutputType,
};
use const_eval::queries::CONST_EVAL_PROVIDER;
use inkwell::{
    OptimizationLevel,
    builder::Builder,
    context::Context,
    llvm_sys::LLVMCallConv,
    module::Module,
    passes::PassBuilderOptions,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
};
use query::{DefId, QueryContext};
use symbol_table::SymbolTable;
use uuid::Uuid;

use crate::{
    info::{Symbol, TypeKind, Value},
    types::get_llvm_type,
};

mod expr;
mod info;
mod program;
mod stmt;
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

pub struct LLVMBackend {
    backend_options: BackendOptions,
}

impl CodegenBackendBase for LLVMBackend {
    fn new(backend_options: BackendOptions) -> Self {
        LazyLock::force(&LLVM_CONTEXT);
        Target::initialize_all(&InitializationConfig::default());
        Self { backend_options }
    }
}

impl CodegenBackend for LLVMBackend {
    fn codegen(
        &self,
        ctx: Arc<QueryContext<'_>>,
        codegen_units: Vec<DefId>,
    ) -> Box<dyn CodegenResult> {
        let (module, global_funcs) = Self::generate_defs(ctx.clone(), &codegen_units);

        let global_funcs = Arc::new(global_funcs);
        let module = Arc::new(module);

        for def_id in codegen_units {
            Self::codegen_item(ctx.clone(), def_id, global_funcs.clone(), module.clone());
        }

        Box::new(LLVMCodegenResult::new(module, self.backend_options))
    }
}

impl LLVMBackend {
    fn generate_defs(
        ctx: Arc<QueryContext<'_>>,
        codegen_units: &[DefId],
    ) -> (Module<'static>, FunctionMap) {
        let mut global_funcs = BTreeMap::new();
        let module = LLVM_CONTEXT.create_module("main");

        for unit in codegen_units.iter() {
            let value = ctx
                .query_cached(&CONST_EVAL_PROVIDER, *unit)
                .unwrap()
                .kind();

            if let const_eval::ValueKind::Proto(proto) = value {
                let ProtoDef {
                    abi,
                    params,
                    return_type,
                    ..
                } = proto.as_ref();

                let function_name = match abi {
                    ast::Abi::CAbi(name) => name.clone(),
                    _ => unreachable!(),
                };

                let (function_type, return_type) =
                    Self::llvm_fn_sig(ctx.clone(), params, return_type);
                let function_value =
                    module.add_function(&function_name, function_type.as_function_type(), None);
                function_value.set_call_conventions(LLVMCallConv::LLVMCCallConv as u32);
                let function = Value::Function(function_value, return_type);

                global_funcs.insert(*unit, function);

                continue;
            }

            let const_eval::ValueKind::Function(func) = value else {
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

            let (function_type, return_type) = Self::llvm_fn_sig(ctx.clone(), params, return_type);
            let function_value =
                module.add_function(&function_name, function_type.as_function_type(), None);
            function_value.set_call_conventions(LLVMCallConv::LLVMCCallConv as u32);
            let function = Value::Function(function_value, return_type);

            let entry = global_funcs.entry(*unit);
            entry.insert_entry(function);
        }

        (module, global_funcs)
    }

    #[inline(always)]
    fn llvm_fn_sig(
        ctx: Arc<QueryContext<'_>>,
        params: &[Param],
        return_type: &Option<Type>,
    ) -> (TypeKind<'static>, TypeKind<'static>) {
        let mut param_types = Vec::new();
        for param in params {
            param_types.push(get_llvm_type(ctx.clone(), &param.param_type));
        }

        let return_type = return_type
            .as_ref()
            .map(|return_type| get_llvm_type(ctx.clone(), return_type))
            .unwrap_or(TypeKind::new_unit());
        let function_type = return_type.function(param_types);

        (function_type, return_type)
    }

    fn codegen_item(
        ctx: Arc<QueryContext<'_>>,
        def_id: DefId,
        global_funcs: Arc<FunctionMap>,
        module: Arc<Module<'static>>,
    ) {
        let const_eval::ValueKind::Function(func) = ctx
            .query_cached(&CONST_EVAL_PROVIDER, def_id)
            .unwrap()
            .kind()
        else {
            return;
        };
        let FunctionDef { params, block, .. } = func.as_ref();

        let func_value = global_funcs.get(&def_id).cloned().unwrap();
        let function = func_value.as_fn();
        let entry_block = LLVM_CONTEXT.append_basic_block(function, "entry");

        let builder = LLVM_CONTEXT.create_builder();
        builder.position_at_end(entry_block);

        let mut visitor_ctx = VisitorCtx {
            builder,
            symbols: SymbolTable::new(),
            module,
            queries: ctx.clone(),
            current_fn: func_value,
            global_funcs,
        };

        for (id, param) in params.iter().enumerate() {
            let ty = get_llvm_type(ctx.clone(), &param.param_type);

            let ptr = visitor_ctx.create_entry_bb_alloca(&param.name, ty);
            visitor_ctx
                .builder
                .build_store(ptr.as_ptr(), function.get_nth_param(id as u32).unwrap())
                .unwrap();

            visitor_ctx
                .symbols
                .pre_push(Symbol::Var(param.name.clone(), ptr));
        }

        if let Some(value) = visitor_ctx.visit_block(block)
            && !matches!(value, Value::Unit)
        {
            visitor_ctx.builder.build_return(Some(&value)).unwrap();
        }
        if visitor_ctx
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            visitor_ctx.builder.build_return(None).unwrap();
        }
    }
}

type FunctionMap = BTreeMap<DefId, Value<'static>>;

pub struct LLVMCodegenResult {
    module: Arc<Module<'static>>,
    target_machine: TargetMachine,
}

impl LLVMCodegenResult {
    fn new(module: Arc<Module<'static>>, backend_options: BackendOptions) -> Self {
        let BackendOptions {
            code_model,
            optimize_level,
            reloc_mode,
        } = backend_options;

        let code_model = match code_model {
            codegen::CodeModel::Default => CodeModel::Default,
            codegen::CodeModel::Large => CodeModel::Large,
            codegen::CodeModel::Medium => CodeModel::Medium,
            codegen::CodeModel::Small => CodeModel::Small,
            codegen::CodeModel::Kernel => CodeModel::Kernel,
        };
        let optimize_level = match optimize_level {
            codegen::OptimizeLevel::O0 => OptimizationLevel::None,
            codegen::OptimizeLevel::O1 => OptimizationLevel::Less,
            codegen::OptimizeLevel::O2 => OptimizationLevel::Default,
            codegen::OptimizeLevel::O3 => OptimizationLevel::Aggressive,
        };
        let reloc_mode = match reloc_mode {
            codegen::RelocMode::Default => RelocMode::Default,
            codegen::RelocMode::Static => RelocMode::Static,
            codegen::RelocMode::Pic => RelocMode::PIC,
            codegen::RelocMode::DynamicNoPic => RelocMode::DynamicNoPic,
        };

        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple).unwrap();
        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                optimize_level,
                reloc_mode,
                code_model,
            )
            .unwrap();

        Self {
            module,
            target_machine,
        }
    }
}

impl CodegenResult for LLVMCodegenResult {
    fn dump(&self) {
        self.module.print_to_stderr();
    }

    fn optimize(&self) {
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

        self.module
            .run_passes(passes.join(",").as_str(), &self.target_machine, options)
            .unwrap();
    }

    fn emit(&self, options: EmitOptions) {
        let path = Path::new(&options.path);
        let file_type = match options.output_type {
            OutputType::Ir => {
                self.module.print_to_file(path).unwrap();
                return;
            }
            OutputType::Asm => FileType::Assembly,
            OutputType::Object => FileType::Object,
        };

        self.target_machine
            .write_to_file(&self.module, file_type, path)
            .unwrap();
    }
}

struct VisitorCtx<'v> {
    builder: Builder<'v>,
    symbols: SymbolTable<Symbol<'v>>,
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

        let alloca_ty = match ty {
            TypeKind::Unit(_) => TypeKind::new_int(8).new_array(0),
            _ => ty.clone(),
        };

        Value::Alloca {
            value: builder.build_alloca(alloca_ty, name).unwrap(),
            value_ty: ty,
        }
    }

    fn create_entry_bb_alloca_with_init(&self, name: &str, init: Value<'v>) -> Value<'v> {
        let alloca = self.create_entry_bb_alloca(name, init.type_());
        let Value::Alloca { value: ptr, .. } = alloca else {
            unreachable!()
        };
        if !init.is_unit() {
            init.init_alloca(ptr, &self.builder);
        }
        alloca
    }
}
