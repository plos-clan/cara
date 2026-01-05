use std::sync::Arc;

use ast::{
    Array, BinaryOp, Block, Call, Deref, FunctionDef, GetAddr, Index, Number, UnaryOp, Var,
    visitor::ExpVisitor,
};

use crate::{ConstEvalContext, info::Value, queries::CONST_EVAL_PROVIDER};

impl<'c> ExpVisitor<Value> for ConstEvalContext<'c> {
    fn get_right_value(&self, left_value: Value) -> Value {
        left_value
    }

    fn visit_array(&mut self, _array: &Array) -> Value {
        unimplemented!()
    }

    fn visit_binary(&mut self, op: &BinaryOp, lhs: Value, rhs: Value) -> Value {
        let Value::Int(_, rhs) = rhs else {
            unreachable!()
        };
        lhs.apply(|lhs| match op {
            BinaryOp::Add => lhs + rhs,
            BinaryOp::Sub => lhs - rhs,
            BinaryOp::Mul => lhs * rhs,
            BinaryOp::Div => lhs / rhs,
            BinaryOp::Mod => lhs % rhs,
            BinaryOp::And => lhs & rhs,
            BinaryOp::Or => lhs | rhs,
            BinaryOp::LShift => lhs << rhs,
            BinaryOp::RShift => lhs >> rhs,
            BinaryOp::Le => (lhs <= rhs) as i64,
            BinaryOp::Lt => (lhs < rhs) as i64,
            BinaryOp::Ge => (lhs >= rhs) as i64,
            BinaryOp::Gt => (lhs > rhs) as i64,
            BinaryOp::Eq => (lhs == rhs) as i64,
            BinaryOp::Ne => (lhs != rhs) as i64,
        })
    }

    fn visit_block(&mut self, _block: &Block) -> Value {
        unimplemented!()
    }

    fn visit_call(&mut self, _call: &Call) -> Value {
        unimplemented!()
    }

    fn visit_deref(&mut self, _deref: &Deref) -> Value {
        unimplemented!()
    }

    fn visit_function(&mut self, func: &FunctionDef) -> Value {
        Value::Function(Arc::new(func.clone()))
    }

    fn visit_get_addr(&mut self, _get_addr: &GetAddr) -> Value {
        unimplemented!()
    }

    fn visit_index(&mut self, _index: &Index) -> Value {
        unimplemented!()
    }

    fn visit_var(&mut self, var: &Var) -> Value {
        let name = var.path.path.join(".");
        let def_id = self.ctx.lookup_def_id(name).unwrap();
        self.ctx.query(&CONST_EVAL_PROVIDER, def_id).unwrap()
    }

    fn visit_number(&mut self, number: &Number) -> Value {
        Value::Int(number.ty.unwrap(), number.num as i64)
    }

    fn visit_str(&mut self, _string: &str) -> Value {
        unimplemented!()
    }

    fn visit_unary(&mut self, op: &UnaryOp, value: Value) -> Value {
        value.apply(|value| match op {
            UnaryOp::Neg => -value,
            UnaryOp::Not => !value,
            UnaryOp::Pos => value,
        })
    }

    fn visit_assign(&mut self, _assign: &ast::Assign) -> Value {
        Value::Unit
    }

    fn visit_unit(&mut self) -> Value {
        Value::Unit
    }

    fn visit_return(&mut self, _return_stmt: &ast::Return) -> Value {
        unimplemented!()
    }

    fn visit_if_exp(&mut self, _if_exp: &ast::IfExp) -> Value {
        unimplemented!()
    }
}
