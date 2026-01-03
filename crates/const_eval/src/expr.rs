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
        let Value::Int(lhs) = lhs else { unreachable!() };
        let Value::Int(rhs) = rhs else { unreachable!() };
        match op {
            BinaryOp::Add => Value::Int(lhs + rhs),
            BinaryOp::Sub => Value::Int(lhs - rhs),
            BinaryOp::Mul => Value::Int(lhs * rhs),
            BinaryOp::Div => Value::Int(lhs / rhs),
            BinaryOp::Mod => Value::Int(lhs % rhs),
            BinaryOp::And => Value::Int(lhs & rhs),
            BinaryOp::Or => Value::Int(lhs | rhs),
            BinaryOp::LShift => Value::Int(lhs << rhs),
            BinaryOp::RShift => Value::Int(lhs >> rhs),
            BinaryOp::Le => Value::Int((lhs <= rhs) as i64),
            BinaryOp::Lt => Value::Int((lhs < rhs) as i64),
            BinaryOp::Ge => Value::Int((lhs >= rhs) as i64),
            BinaryOp::Gt => Value::Int((lhs > rhs) as i64),
            BinaryOp::Eq => Value::Int((lhs == rhs) as i64),
            BinaryOp::Ne => Value::Int((lhs != rhs) as i64),
        }
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
        Value::Int(number.num as i64)
    }

    fn visit_str(&mut self, _string: &str) -> Value {
        unimplemented!()
    }

    fn visit_unary(&mut self, op: &UnaryOp, value: Value) -> Value {
        let Value::Int(value) = value else {
            unreachable!()
        };
        match op {
            UnaryOp::Neg => Value::Int(-value),
            UnaryOp::Not => Value::Int(!value),
            UnaryOp::Pos => Value::Int(value),
        }
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
}
