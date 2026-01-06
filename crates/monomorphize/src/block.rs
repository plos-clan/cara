use ast::visitor::{BlockVisitor, ExpVisitor};

use crate::MonomorphizeContext;

impl BlockVisitor<()> for MonomorphizeContext<'_> {
    fn on_enter_block(&mut self) {
        self.push_scope();
    }

    fn on_leave_block(&mut self) {
        self.pop_scope();
    }

    fn visit_inline_asm(&mut self, _inline_asm: &ast::InlineAsm) {}

    fn visit_var_def(&mut self, var_def: &ast::VarDef) {
        self.visit_right_value(&var_def.initial_value);
        self.push_symbol(var_def.name.clone());
    }
}
