use std::sync::{Arc, LazyLock};

use ast::{FunctionDef, visitor::BlockVisitor};
use const_eval::queries::CONST_EVAL_PROVIDER;
use inkwell::{module::Module, values::AnyValue};
use query::{DefId, Provider, QueryContext};
use send_wrapper::SendWrapper;
use uuid::Uuid;

use crate::{
    LLVM_CONTEXT, VisitorCtx,
    info::{Symbol, SymbolStack, Value},
    types::get_llvm_type,
};

#[derive(Clone)]
pub struct CodegenResult {
    pub module: Option<Module<'static>>,
    pub value: Value<'static>,
}

pub static CODEGEN_PROVIDER: LazyLock<Provider<DefId, SendWrapper<CodegenResult>>> =
    LazyLock::new(|| Provider::new(codegen_provider));

fn codegen_provider(ctx: Arc<QueryContext<'_>>, def_id: DefId) -> SendWrapper<CodegenResult> {
    let eval_result = ctx.query(&CONST_EVAL_PROVIDER, def_id).unwrap();

    let func = match eval_result {
        const_eval::Value::Function(func) => func,
        const_eval::Value::Int(int) => {
            return SendWrapper::new(CodegenResult {
                module: None,
                value: Value::Int(LLVM_CONTEXT.i32_type().const_int(int as u64, true)),
            });
        }
    };

    let module = LLVM_CONTEXT.create_module("main");

    let FunctionDef {
        abi,
        params,
        return_type,
        block,
        span: _,
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
    let result = Value::Function(function, return_type);

    let entry_bb = LLVM_CONTEXT.append_basic_block(function, "entry");
    let builder = LLVM_CONTEXT.create_builder();
    builder.position_at_end(entry_bb);

    let mut ctx = VisitorCtx {
        builder,
        symbols: SymbolStack::new(),
        module,
        queries: ctx.clone(),
        current_fn: function,
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
        && !matches!(value, Value::Void)
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

    SendWrapper::new(CodegenResult {
        module: Some(ctx.module),
        value: result,
    })
}
