use ast::{
    Assign, For, IfExp, While,
    visitor::{ExpVisitor, StatementVisitor},
};

use crate::{AnalyzerContext, Error, Symbol, Type, Value};

impl StatementVisitor<Value> for AnalyzerContext {
    fn visit_assign(&mut self, assign: &Assign) -> Value {
        let Assign { lhs, rhs, .. } = assign;

        let lhs_val = self.visit_left_value(*lhs);
        let lhs_type = lhs_val.into_type();

        let rhs_val = self.visit_right_value(*rhs);
        let rhs_type = rhs_val.into_type();

        if lhs_type != rhs_type {
            self.error_at(Error::TypeMismatch(lhs_type, rhs_type), rhs.span());
        }
        Value::new(Type::Unit)
    }

    fn visit_for(&mut self, for_: &For) -> Value {
        let For {
            var,
            start,
            end,
            step,
            body,
            ..
        } = for_;

        let start_val = self.visit_right_value(*start);
        let start_type = start_val.clone().into_type();
        let end_val = self.visit_right_value(*end);
        let end_type = end_val.into_type();

        if start_type != end_type {
            self.error_at(
                Error::TypeMismatch(start_type.clone(), end_type),
                end.span(),
            );
        }

        if let Some(step_val) = step.as_ref().map(|s| self.visit_right_value(*s)) {
            let step_type = step_val.into_type();
            if step_type != start_type {
                self.error_at(
                    Error::TypeMismatch(start_type, step_type),
                    step.as_ref().unwrap().span(),
                );
            }
        }

        self.symbols
            .pre_push(Symbol::Var(var.clone(), false, start_val));

        let block_ret_type = self.visit_block(body).into_type();
        if block_ret_type != Type::Unit {
            self.error_at(
                Error::TypeMismatch(Type::Unit, block_ret_type),
                body.return_value.as_ref().unwrap().span(),
            );
        }

        Value::new(Type::Unit)
    }

    fn visit_if_exp(&mut self, if_exp: &IfExp) -> Value {
        let IfExp {
            condition,
            then_branch,
            else_branch,
            else_if,
            ..
        } = if_exp;

        let condition_ty = self.visit_right_value(*condition).into_type();
        if !condition_ty.is_bool() {
            self.error_at(
                Error::TypeMismatch(Type::Bool, condition_ty),
                condition.span(),
            );
        }

        let then_branch_ty = self.visit_block(then_branch).into_type();
        let else_branch_ty = if let Some(else_branch) = else_branch {
            self.visit_block(else_branch).into_type()
        } else if let Some(else_if) = else_if {
            self.visit_if_exp(else_if).into_type()
        } else {
            Type::Unit
        };
        if then_branch_ty != else_branch_ty {
            self.error_at(
                Error::TypeMismatch(else_branch_ty, then_branch_ty.clone()),
                then_branch.span,
            );
        }

        Value::new(then_branch_ty)
    }

    fn visit_loop(&mut self, loop_: &ast::Loop) -> Value {
        let loop_ret_ty = self.visit_block(&loop_.body).into_type();
        if !loop_ret_ty.is_unit() {
            self.error_at(
                Error::TypeMismatch(Type::Unit, loop_ret_ty),
                loop_.body.return_value.as_ref().unwrap().span(),
            );
        }
        Value::new(Type::Unit)
    }

    fn visit_return(&mut self, return_stmt: &ast::Return) -> Value {
        let ty = return_stmt
            .value
            .as_ref()
            .map(|v| self.visit_right_value(*v).into_type())
            .unwrap_or(Type::Unit);
        let Some(should_be_ty) = self.ret_ty.as_ref() else {
            return Value::default();
        };
        if ty != *should_be_ty {
            self.error_at(
                Error::TypeMismatch(should_be_ty.clone(), ty),
                return_stmt
                    .value
                    .as_ref()
                    .map(|v| v.span())
                    .unwrap_or(return_stmt.span),
            );
        }
        Value::default()
    }

    fn visit_while(&mut self, while_: &While) -> Value {
        let While {
            condition, body, ..
        } = while_;

        let condition_ty = self.visit_right_value(*condition).into_type();
        if !condition_ty.is_bool() {
            self.error_at(
                Error::TypeMismatch(Type::Bool, condition_ty),
                condition.span(),
            );
        }

        let loop_ret_ty = self.visit_block(body).into_type();
        if !loop_ret_ty.is_unit() {
            self.error_at(
                Error::TypeMismatch(Type::Unit, loop_ret_ty),
                body.return_value.as_ref().unwrap().span(),
            );
        }
        Value::new(Type::Unit)
    }
}
