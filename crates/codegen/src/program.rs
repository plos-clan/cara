use std::sync::Arc;

use ast::{Block, BlockItem, Return, Statement};

use crate::{Generator, VisitorCtx, info::Value};

impl<'v> Generator<'v> {
    pub fn visit_block(
        self: &Arc<Self>,
        ctx: &mut VisitorCtx<'v>,
        block: &Block,
    ) -> Option<Value<'v>> {
        ctx.symbols.push_scope();
        for item in &block.items {
            match item {
                BlockItem::Statement(stmt) => self.visit_statement(ctx, stmt),
            }
        }
        let return_value = block.return_value.as_ref().map(|v| self.visit_exp(ctx, v));
        ctx.symbols.pop_scope();
        return_value
    }

    fn visit_statement(self: &Arc<Self>, ctx: &mut VisitorCtx<'v>, stmt: &Statement) {
        match stmt {
            Statement::Exp(exp) => {
                self.visit_exp(ctx, exp);
            }
            Statement::Return(ret) => self.visit_return(ctx, ret),
        }
    }

    fn visit_return(self: &Arc<Self>, ctx: &mut VisitorCtx<'v>, ret: &Return) {
        if let Some(value) = ret.value.as_ref() {
            let value = self.visit_exp(ctx, value);
            ctx.builder
                .build_return(if matches!(value, Value::Void) {
                    None
                } else {
                    Some(&value)
                })
                .unwrap();
        } else {
            ctx.builder.build_return(None).unwrap();
        }
    }
}
