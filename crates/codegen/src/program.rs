use ast::visitor::{BlockVisitor, ExpVisitor};

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

    fn visit_var_def(&mut self, var_def: &ast::VarDef) {
        let value = self.visit_right_value(&var_def.initial_value);
        let alloca = self.create_entry_bb_alloca_with_init(&var_def.name, value);

        self.symbols.push(Symbol::Var(var_def.name.clone(), alloca));
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
