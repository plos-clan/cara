use ast::visitor::{ExpVisitor, StatementVisitor};

use crate::MonomorphizeContext;

impl StatementVisitor<()> for MonomorphizeContext {
    fn visit_assign(&mut self, assign: &ast::Assign) {
        self.visit_left_value(assign.lhs);
        self.visit_right_value(assign.rhs);
    }

    fn visit_return(&mut self, return_stmt: &ast::Return) {
        if let Some(value) = return_stmt.value {
            self.visit_right_value(value);
        }
    }

    fn visit_for(&mut self, for_: &ast::For) {
        self.visit_right_value(for_.start);
        self.visit_right_value(for_.end);
        if let Some(step) = for_.step {
            self.visit_right_value(step);
        }
        self.locals.pre_push(for_.var.clone());
        self.visit_block(&for_.body);
    }

    fn visit_loop(&mut self, loop_: &ast::Loop) {
        self.visit_block(&loop_.body);
    }

    fn visit_while(&mut self, while_: &ast::While) {
        self.visit_right_value(while_.condition);
        self.visit_block(&while_.body);
    }

    fn visit_if_exp(&mut self, if_exp: &ast::IfExp) {
        self.visit_block(&if_exp.then_branch);
        if let Some(else_block) = if_exp.else_branch.as_ref() {
            self.visit_block(else_block);
        }
        if let Some(else_if_exp) = if_exp.else_if.as_ref() {
            self.visit_if_exp(else_if_exp);
        }
    }
}
