use ast::{
    Return,
    visitor::{BlockVisitor, ExpVisitor},
};

use crate::{
    LLVM_CONTEXT, VisitorCtx,
    info::{Symbol, Value},
};

impl<'v> BlockVisitor<Value<'v>> for VisitorCtx<'v> {
    fn on_enter_block(&mut self) {
        self.symbols.push_scope();
    }

    fn on_leave_block(&mut self) {
        self.symbols.pop_scope();
    }

    fn visit_return(&mut self, ret: &Return) -> Option<Value<'v>> {
        if let Some(value) = ret.value.as_ref() {
            let value = self.visit_exp(value);
            self.builder
                .build_return(if matches!(value, Value::Void) {
                    None
                } else {
                    Some(&value)
                })
                .unwrap();
        } else {
            self.builder.build_return(None).unwrap();
        }

        None
    }

    fn visit_var_def(&mut self, var_def: &ast::VarDef) {
        let value = self.visit_exp(&var_def.initial_value);

        if var_def.mutable {
            let ty = value.type_(&LLVM_CONTEXT);
            let alloca = self.create_entry_bb_alloca(&var_def.name, ty);
            let Value::Pointer { value: ptr, .. } = alloca else {
                unreachable!()
            };

            self.builder.build_store(ptr, value).unwrap();

            self.symbols
                .push(Symbol::MutableVar(var_def.name.clone(), alloca));
        } else {
            self.symbols
                .push(Symbol::ImmutableVar(var_def.name.clone(), value));
        }
    }
}
