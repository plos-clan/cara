use ast::{
    Return,
    visitor::{BlockVisitor, ExpVisitor},
};

use crate::{
    LLVM_CONTEXT, VisitorCtx,
    info::{Symbol, TypeKind, Value},
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
            let value = self.visit_right_value(value);
            self.builder
                .build_return(if matches!(value, Value::Unit) {
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
        let value = self.visit_right_value(&var_def.initial_value);

        if var_def.mutable && !matches!(value, Value::Unit) {
            let ty = value.type_();
            let alloca = self.create_entry_bb_alloca(&var_def.name, ty);
            let Value::Alloca { value: ptr, .. } = alloca else {
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

    fn visit_inline_asm(&mut self, inline_asm: &ast::InlineAsm) {
        let fn_ty = TypeKind::new_unit().function(Vec::new()).as_function_type();

        let asm_fn = LLVM_CONTEXT.create_inline_asm(
            fn_ty,
            inline_asm.asm.join("\n"),
            "".into(),
            true,
            false,
            None,
            false,
        );

        self.builder
            .build_indirect_call(fn_ty, asm_fn, &[], "")
            .unwrap();
    }
}
