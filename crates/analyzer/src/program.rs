use ast::visitor::{BlockVisitor, ExpVisitor};

use crate::{AnalyzerContext, Error, Symbol, Value, get_analyzer_type};

impl BlockVisitor<Value> for AnalyzerContext<'_> {
    fn on_enter_block(&mut self) {
        self.symbols.push_scope();
    }

    fn on_leave_block(&mut self) {
        self.symbols.pop_scope();
    }

    fn visit_inline_asm(&mut self, _inline_asm: &ast::InlineAsm) {}

    fn visit_var_def(&mut self, var_def: &ast::VarDef) {
        let value = self.visit_right_value(&var_def.initial_value);
        if let Some(should_be_type) = var_def.var_type.as_ref().map(|ty| get_analyzer_type(ty))
            && should_be_type != *value.type_()
        {
            self.error_at(
                Error::TypeMismatch(should_be_type, value.type_().clone()),
                var_def.initial_value.span(),
            );
        }

        self.symbols
            .push(Symbol::Var(var_def.name.clone(), var_def.mutable, value));
    }
}
