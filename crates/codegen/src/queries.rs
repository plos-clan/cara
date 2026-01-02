use std::sync::{Arc, LazyLock};

use ast::{ConstDef, ConstExp, ConstInitialValue, Exp, FunctionDef, visitor::BlockVisitor};
use inkwell::module::Module;
use query::{DefId, Provider, QueryContext};
use send_wrapper::SendWrapper;
use uuid::Uuid;

use crate::{
    LLVM_CONTEXT, VisitorCtx,
    info::{SymbolStack, Value},
    types::get_llvm_type,
};

#[derive(Clone)]
pub struct CodegenResult {
    pub module: Module<'static>,
    pub value: Value<'static>,
}

pub static CODEGEN_PROVIDER: LazyLock<Provider<DefId, SendWrapper<CodegenResult>>> =
    LazyLock::new(|| Provider::new(codegen_provider));

fn codegen_provider(ctx: Arc<QueryContext<'_>>, def_id: DefId) -> SendWrapper<CodegenResult> {
    let module: Module<'static> = LLVM_CONTEXT.create_module("main");

    let ConstDef { initial_value, .. } = ctx.get_def(def_id).unwrap();

    match initial_value {
        ConstInitialValue::Exp(ConstExp {
            exp: Exp::Function(func),
        }) => {
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
            let function =
                module.add_function(&function_name, function_type.as_function_type(), None);
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
            };
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
                module: ctx.module,
                value: result,
            })
        }
        ConstInitialValue::Exp(_) => unimplemented!(),
    }
}
