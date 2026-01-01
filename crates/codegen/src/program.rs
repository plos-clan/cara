use ast::{
    Return,
    visitor::{BlockVisitor, ExpVisitor},
};

use crate::{VisitorCtx, info::Value};

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
}
