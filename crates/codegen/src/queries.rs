use std::sync::Arc;

use ast::{ConstDef, ConstInitialValue, FunctionDef, visitor::BlockVisitor};
use query::{DefId, QueryContext};

use crate::{
    Generator, VisitorCtx,
    info::{SymbolStack, Value},
};

pub fn codegen_provider<'g>(ctx: &QueryContext, arg: (Arc<Generator<'g>>, DefId)) -> Value<'g> {
    let (generator, def_id) = arg;

    let ConstDef { initial_value, .. } = ctx.get_def(def_id).unwrap();

    match initial_value {
        ConstInitialValue::Function(FunctionDef {
            abi,
            params,
            return_type,
            block,
            span: _,
        }) => {
            let function_name = match abi {
                ast::Abi::CAbi(name) => name.clone(),
                _ => "".into(),
            };

            let mut param_types = Vec::new();
            for param in params {
                param_types.push(generator.visit_type(&param.param_type));
            }

            let return_type = generator.visit_type(&return_type);
            let function_type = return_type.function(param_types);
            let function = generator.module.lock().unwrap().add_function(
                &function_name,
                function_type.as_function_type(),
                None,
            );
            function.set_call_conventions(0); // C
            let result = Value::Function(function, return_type);

            let entry_bb = generator.ctx.append_basic_block(function, "entry");
            let builder = generator.ctx.create_builder();
            builder.position_at_end(entry_bb);

            let mut ctx = VisitorCtx {
                builder,
                symbols: SymbolStack::new(),
                generator: generator.clone(),
            };
            if let Some(value) = ctx.visit_block(&block)
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

            result
        }
        ConstInitialValue::Exp(_) => unimplemented!(),
    }
}
