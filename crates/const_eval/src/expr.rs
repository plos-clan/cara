use std::sync::Arc;

use ast::{
    Array, BinaryOp, Block, Call, Deref, FunctionDef, Index, Number, Span, Type, TypeEnum, UnaryOp,
    Var, visitor::ExpVisitor,
};

use crate::{ConstEvalContext, info::Value, queries::CONST_EVAL_PROVIDER};

impl<'c> ExpVisitor<Value> for ConstEvalContext<'c> {
    fn get_right_value(&self, left_value: Value) -> Value {
        left_value
    }

    fn pass_left_value_as_right_value(&self, left_value: Value) -> Value {
        left_value
    }

    fn visit_array(&mut self, _array: &Array) -> Value {
        unimplemented!()
    }

    fn visit_binary(&mut self, op: &BinaryOp, lhs_val: Value, rhs_val: Value, _: &Span) -> Value {
        let lhs = lhs_val.as_int();
        let rhs = rhs_val.as_int();
        let mut result = Value::new_int(match op {
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
        });
        result.set_type(lhs_val.ty());
        result
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

    fn visit_proto(&mut self, proto_def: &ast::ProtoDef) -> Value {
        Value::new_proto(Arc::new(proto_def.clone()))
    }

    fn visit_function(&mut self, func: &FunctionDef) -> Value {
        Value::new_function(Arc::new(func.clone()))
    }

    fn visit_index(&mut self, _index: &Index) -> Value {
        unimplemented!()
    }

    fn visit_var(&mut self, var: &Var) -> Value {
        let name = var.path.path.join(".");
        let def_id = self.ctx.lookup_def_id(name).unwrap();
        self.ctx.query_cached(&CONST_EVAL_PROVIDER, def_id).unwrap()
    }

    fn visit_number(&mut self, number: &Number) -> Value {
        let mut value = Value::new_int(number.num as i64);
        if let Some((signed, width)) = number.ty {
            value.set_type(Arc::new(Type {
                kind: if signed {
                    TypeEnum::Signed(width)
                } else {
                    TypeEnum::Unsigned(width)
                },
                ref_count: 0,
                span: Span::default(),
            }));
        }
        value
    }

    fn visit_str(&mut self, _string: &str) -> Value {
        unimplemented!()
    }

    fn visit_unary(&mut self, op: &UnaryOp, value: Value, _: &Span) -> Value {
        let int_value = value.as_int();
        let mut result = Value::new_int(match op {
            UnaryOp::Neg => -int_value,
            UnaryOp::Not => !int_value,
            UnaryOp::Pos => int_value,
        });
        result.set_type(value.ty());
        result
    }

    fn visit_unit(&mut self) -> Value {
        Value::new_unit()
    }

    fn visit_type_cast(&mut self, type_cast: &ast::TypeCast) -> Value {
        self.visit_right_value(&type_cast.exp)
    }
}
