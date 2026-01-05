use ast::{Array, visitor::ExpVisitor};

use crate::MonomorphizeContext;

impl ExpVisitor<()> for MonomorphizeContext<'_> {
    fn get_right_value(&self, _left_value: ()) {}
    fn pass_left_value_as_right_value(&self, _left_value: ()) {}

    fn visit_array(&mut self, array: &Array) {
        match array {
            Array::List(elements, _) => {
                for element in elements {
                    self.visit_right_value(element);
                }
            }
            _ => unimplemented!(),
        }
    }

    fn visit_assign(&mut self, assign: &ast::Assign) {
        self.visit_left_value(&assign.lhs);
        self.visit_right_value(&assign.rhs);
    }

    fn visit_binary(&mut self, _op: &ast::BinaryOp, _lhs: (), _rhs: ()) {}

    fn visit_block(&mut self, block: &ast::Block) {
        <Self as ast::visitor::BlockVisitor<()>>::visit_block(self, block);
    }

    fn visit_call(&mut self, call: &ast::Call) {
        self.visit_right_value(&call.func)
    }

    fn visit_deref(&mut self, deref: &ast::Deref) {
        self.visit_left_value(&deref.exp);
    }

    fn visit_function(&mut self, _func: &ast::FunctionDef) {
        unreachable!()
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

    fn visit_index(&mut self, index: &ast::Index) {
        self.visit_left_value(&index.exp);
        self.visit_right_value(&index.index);
    }

    fn visit_number(&mut self, _number: &ast::Number) {}

    fn visit_return(&mut self, return_stmt: &ast::Return) {
        if let Some(value) = return_stmt.value.as_ref() {
            self.visit_right_value(value);
        }
    }

    fn visit_str(&mut self, _string: &str) {}

    fn visit_unary(&mut self, _op: &ast::UnaryOp, _value: ()) {}

    fn visit_unit(&mut self) {}

    fn visit_var(&mut self, var: &ast::Var) {
        let name = var.path.path.join(".");
        if !self.contains(&name) {
            self.required_items
                .push(self.ctx.lookup_def_id(name).unwrap());
        }
    }
}
