use inkwell::{types::BasicTypeEnum, values::AnyValue};

use ast::{Array, BinaryOp, Call, LVal, visitor::ExpVisitor};

use crate::{VisitorCtx, info::Value};

impl<'v> ExpVisitor<Value<'v>> for VisitorCtx<'v> {
    fn visit_binary(&mut self, op: &BinaryOp, lhs: Value<'v>, rhs: Value<'v>) -> Value<'v> {
        let Value::Int(lhs) = lhs else { unreachable!() };
        let Value::Int(rhs) = rhs else { unreachable!() };

        let builder = &self.builder;
        let result = match op {
            BinaryOp::Add => builder.build_int_add(lhs, rhs, ""),
            BinaryOp::Sub => builder.build_int_sub(lhs, rhs, ""),
            BinaryOp::Mul => builder.build_int_mul(lhs, rhs, ""),
            BinaryOp::Div => builder.build_int_unsigned_div(lhs, rhs, ""),
            BinaryOp::Mod => builder.build_int_unsigned_rem(lhs, rhs, ""),
            BinaryOp::Lt => builder.build_int_compare(inkwell::IntPredicate::ULT, lhs, rhs, ""),
            BinaryOp::Le => builder.build_int_compare(inkwell::IntPredicate::ULE, lhs, rhs, ""),
            BinaryOp::Gt => builder.build_int_compare(inkwell::IntPredicate::UGT, lhs, rhs, ""),
            BinaryOp::Ge => builder.build_int_compare(inkwell::IntPredicate::UGE, lhs, rhs, ""),
            BinaryOp::Eq => builder.build_int_compare(inkwell::IntPredicate::EQ, lhs, rhs, ""),
            BinaryOp::Ne => builder.build_int_compare(inkwell::IntPredicate::NE, lhs, rhs, ""),
            BinaryOp::And => builder.build_and(lhs, rhs, ""),
            BinaryOp::Or => builder.build_or(lhs, rhs, ""),
            BinaryOp::LShift => builder.build_left_shift(lhs, rhs, ""),
            BinaryOp::RShift => builder.build_right_shift(lhs, rhs, false, ""),
        }
        .unwrap();

        Value::Int(result)
    }

    fn visit_block(&mut self, block: &ast::Block) -> Value<'v> {
        use ast::visitor::BlockVisitor;
        <Self as BlockVisitor<Value<'v>>>::visit_block(self, block).unwrap_or(Value::Void)
    }

    fn visit_deref(&mut self, _deref: &ast::Deref) -> Value<'v> {
        unimplemented!()
    }

    fn visit_get_addr(&mut self, _get_addr: &ast::GetAddr) -> Value<'v> {
        unimplemented!()
    }

    fn visit_index(&mut self, _index: &ast::Index) -> Value<'v> {
        unimplemented!()
    }

    fn visit_number(&mut self, number: &ast::Number) -> Value<'v> {
        Value::Int(self.generator.ctx.i32_type().const_int(number.num, true))
    }

    fn visit_str(&mut self, _string: &str) -> Value<'v> {
        unimplemented!()
    }

    fn visit_unary(&mut self, op: &ast::UnaryOp, value: Value<'v>) -> Value<'v> {
        let Value::Int(value) = value else {
            unreachable!()
        };
        Value::Int(match op {
            ast::UnaryOp::Neg => self.builder.build_int_neg(value, "").unwrap(),
            ast::UnaryOp::Pos => value,
            ast::UnaryOp::Not => self.builder.build_not(value, "").unwrap(),
        })
    }

    fn visit_call(&mut self, call: &Call) -> Value<'v> {
        let Value::Function(func, ret_ty) = self.visit_exp(&call.func) else {
            unreachable!()
        };
        let args = call
            .args
            .iter()
            .map(|arg| self.visit_exp(arg).into())
            .collect::<Vec<_>>();
        let result = self
            .builder
            .build_call(func, &args, "")
            .unwrap()
            .as_any_value_enum();

        Value::new_from(result, ret_ty)
    }

    fn visit_array(&mut self, array: &Array) -> Value<'v> {
        match array {
            Array::List(values, _) => {
                let values = values.iter().map(|e| self.visit_exp(e)).collect::<Vec<_>>();
                let type_ = values[0].type_(self.generator.ctx);

                let array = self
                    .builder
                    .build_alloca(BasicTypeEnum::from(type_.clone()), "")
                    .unwrap();
                Value::Pointer {
                    value: array,
                    ty: self.generator.new_ptr(type_),
                }
            }
            Array::Template(_, _, _) => {
                unimplemented!()
            }
        }
    }

    fn visit_lval(&mut self, lval: &LVal) -> Value<'v> {
        let name = lval.path.path.join(".");
        if let Some(value) = self.symbols.lookup(&name).map(|s| s.value()) {
            value.clone()
        } else {
            self.generator.query(&name).unwrap()
        }
    }
}
