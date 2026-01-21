use ast::{
    Assign,
    visitor::{ExpVisitor, StatementVisitor},
};
use inkwell::{
    IntPredicate,
    values::{AnyValue, InstructionOpcode},
};

use crate::{
    LLVM_CONTEXT, VisitorCtx,
    info::{Symbol, TypeKind, Value},
};

impl<'v> StatementVisitor<Value<'v>> for VisitorCtx<'v> {
    fn visit_assign(&mut self, assign: &Assign) -> Value<'v> {
        let lhs = self.visit_left_value(assign.lhs);
        let rhs = self.visit_right_value(assign.rhs);

        if let Value::Unit = rhs {
            return Value::Unit;
        }

        let ptr = lhs.as_ptr();
        self.builder.build_store(ptr, rhs).unwrap();

        Value::Unit
    }

    fn visit_return(&mut self, ret: &ast::Return) -> Value<'v> {
        if let Some(value) = ret.value {
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

        Value::Unit
    }

    fn visit_if_exp(&mut self, if_exp: &ast::IfExp) -> Value<'v> {
        let condition = self.visit_right_value(if_exp.condition).as_int();
        let condition = self
            .builder
            .build_bit_cast(condition, LLVM_CONTEXT.bool_type(), "")
            .unwrap()
            .into_int_value();

        let current_fn = self.current_fn.as_fn();
        let then_block = LLVM_CONTEXT.append_basic_block(current_fn, "then");
        let else_block = LLVM_CONTEXT.append_basic_block(current_fn, "else");
        let end_block = LLVM_CONTEXT.append_basic_block(current_fn, "end");

        self.builder
            .build_conditional_branch(condition, then_block, else_block)
            .unwrap();

        self.builder.position_at_end(then_block);
        let then_value = self.visit_block(&if_exp.then_branch);
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.builder.build_unconditional_branch(end_block).unwrap();
        }

        self.builder.position_at_end(else_block);
        let (else_present, else_value) = if let Some(else_block) = if_exp.else_branch.as_ref() {
            let else_value = self.visit_block(else_block);
            (true, else_value)
        } else if let Some(else_if) = if_exp.else_if.as_ref() {
            let else_value = self.visit_if_exp(else_if);
            (true, else_value)
        } else {
            (false, Value::Unit)
        };
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.builder.build_unconditional_branch(end_block).unwrap();
        }

        self.builder.position_at_end(end_block);
        if else_present && !matches!(then_value, Value::Unit) {
            let phi = self
                .builder
                .build_phi(then_value.type_(), "if_result")
                .unwrap();
            phi.add_incoming(&[(&then_value, then_block), (&else_value, else_block)]);
            Value::new_from(phi.as_any_value_enum(), then_value.type_())
        } else {
            Value::Unit
        }
    }

    fn visit_loop(&mut self, loop_: &ast::Loop) -> Value<'v> {
        let current_fn = self.current_fn.as_fn();

        let loop_block = LLVM_CONTEXT.append_basic_block(current_fn, "loop");
        let end_block = LLVM_CONTEXT.append_basic_block(current_fn, "end");

        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(loop_block);
        self.push_loop(loop_block, end_block);
        self.visit_block(&loop_.body);
        self.pop_loop_block();
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.builder.build_unconditional_branch(loop_block).unwrap();
        }

        self.builder.position_at_end(end_block);
        Value::Unit
    }

    fn visit_for(&mut self, for_: &ast::For) -> Value<'v> {
        let current_fn = self.current_fn.as_fn();

        let start = self.visit_right_value(for_.start);
        let end = self.visit_right_value(for_.end);
        let step = for_
            .step
            .map(|step| self.visit_right_value(step))
            .unwrap_or(TypeKind::new_int(32).const_int(1));

        let alloca = self.create_entry_bb_alloca_with_init(&for_.var, start);
        self.symbols
            .pre_push(Symbol::Var(for_.var.clone(), alloca.clone()));

        let condition_block = LLVM_CONTEXT.append_basic_block(current_fn, "condition");
        let loop_block = LLVM_CONTEXT.append_basic_block(current_fn, "loop");
        let end_block = LLVM_CONTEXT.append_basic_block(current_fn, "end");

        self.builder
            .build_unconditional_branch(condition_block)
            .unwrap();
        self.builder.position_at_end(condition_block);
        let condition = self
            .builder
            .build_int_compare(
                IntPredicate::SLT,
                alloca.as_right_value(&self.builder).as_int(),
                end.as_int(),
                "",
            )
            .unwrap();
        self.builder
            .build_conditional_branch(condition, loop_block, end_block)
            .unwrap();

        self.builder.position_at_end(loop_block);
        self.push_loop(condition_block, end_block);
        self.visit_block(&for_.body);
        self.pop_loop_block();
        let new_value = self
            .builder
            .build_binop(
                InstructionOpcode::Add,
                alloca.as_right_value(&self.builder),
                step,
                "",
            )
            .unwrap();
        self.builder
            .build_store(alloca.as_ptr(), new_value)
            .unwrap();
        self.builder
            .build_unconditional_branch(condition_block)
            .unwrap();

        self.builder.position_at_end(end_block);

        Value::Unit
    }

    fn visit_while(&mut self, while_: &ast::While) -> Value<'v> {
        let current_fn = self.current_fn.as_fn();

        let condition_block = LLVM_CONTEXT.append_basic_block(current_fn, "condition");
        let loop_block = LLVM_CONTEXT.append_basic_block(current_fn, "loop");
        let end_block = LLVM_CONTEXT.append_basic_block(current_fn, "end");

        self.builder
            .build_unconditional_branch(condition_block)
            .unwrap();
        self.builder.position_at_end(condition_block);
        let condition = self.visit_right_value(while_.condition);
        self.builder
            .build_conditional_branch(condition.as_int(), loop_block, end_block)
            .unwrap();

        self.builder.position_at_end(loop_block);
        self.push_loop(condition_block, end_block);
        self.visit_block(&while_.body);
        self.pop_loop_block();
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.builder
                .build_unconditional_branch(condition_block)
                .unwrap();
        }

        self.builder.position_at_end(end_block);

        Value::Unit
    }

    fn visit_break(&mut self, _span: ast::Span) -> Value<'v> {
        let exit = self.current_loop().unwrap().1;
        self.builder.build_unconditional_branch(exit).unwrap();
        Value::Unit
    }

    fn visit_continue(&mut self, _span: ast::Span) -> Value<'v> {
        let entry = self.current_loop().unwrap().0;
        self.builder.build_unconditional_branch(entry).unwrap();
        Value::Unit
    }
}
