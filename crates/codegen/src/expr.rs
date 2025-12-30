use std::sync::Arc;

use inkwell::{types::BasicTypeEnum, values::AnyValue};

use ast::{Array, BinaryOp, Call, Exp, LVal};

use crate::{Generator, VisitorCtx, info::Value};

impl<'g> Generator<'g> {
    pub fn visit_exp(self: &Arc<Self>, ctx: &mut VisitorCtx<'g>, exp: &Exp) -> Value<'g> {
        match exp {
            Exp::Array(array) => self.visit_array(ctx, array),
            Exp::Binary(op, lhs, rhs, _) => {
                let Value::Int(lhs) = self.visit_exp(ctx, lhs) else {
                    unreachable!()
                };
                let Value::Int(rhs) = self.visit_exp(ctx, rhs) else {
                    unreachable!()
                };

                let builder = &ctx.builder;
                let result = match op {
                    BinaryOp::Add => builder.build_int_add(lhs, rhs, ""),
                    BinaryOp::Sub => builder.build_int_sub(lhs, rhs, ""),
                    BinaryOp::Mul => builder.build_int_mul(lhs, rhs, ""),
                    BinaryOp::Div => builder.build_int_unsigned_div(lhs, rhs, ""),
                    BinaryOp::Mod => builder.build_int_unsigned_rem(lhs, rhs, ""),
                    BinaryOp::Lt => {
                        builder.build_int_compare(inkwell::IntPredicate::ULT, lhs, rhs, "")
                    }
                    BinaryOp::Le => {
                        builder.build_int_compare(inkwell::IntPredicate::ULE, lhs, rhs, "")
                    }
                    BinaryOp::Gt => {
                        builder.build_int_compare(inkwell::IntPredicate::UGT, lhs, rhs, "")
                    }
                    BinaryOp::Ge => {
                        builder.build_int_compare(inkwell::IntPredicate::UGE, lhs, rhs, "")
                    }
                    BinaryOp::Eq => {
                        builder.build_int_compare(inkwell::IntPredicate::EQ, lhs, rhs, "")
                    }
                    BinaryOp::Ne => {
                        builder.build_int_compare(inkwell::IntPredicate::NE, lhs, rhs, "")
                    }
                    BinaryOp::And => builder.build_and(lhs, rhs, ""),
                    BinaryOp::Or => builder.build_or(lhs, rhs, ""),
                    BinaryOp::LShift => builder.build_left_shift(lhs, rhs, ""),
                    BinaryOp::RShift => builder.build_right_shift(lhs, rhs, false, ""),
                }
                .unwrap();

                Value::Int(result)
            }
            Exp::Call(call) => self.visit_call(ctx, call),
            Exp::Exp(e, _) => self.visit_exp(ctx, e),
            Exp::Number(n) => Value::Int(self.ctx.i32_type().const_int(n.num, true)),
            Exp::LVal(lval) => self.visit_lval(ctx, lval),
            _ => unimplemented!(),
        }
    }

    fn visit_call(self: &Arc<Self>, ctx: &mut VisitorCtx<'g>, call: &Call) -> Value<'g> {
        let Value::Function(func, ret_ty) = self.visit_exp(ctx, &call.func) else {
            unreachable!()
        };
        let args = call
            .args
            .iter()
            .map(|arg| self.visit_exp(ctx, arg).into())
            .collect::<Vec<_>>();
        let result = ctx
            .builder
            .build_call(func, &args, "")
            .unwrap()
            .as_any_value_enum();

        Value::new_from(result, ret_ty)
    }

    fn visit_array(self: &Arc<Self>, ctx: &mut VisitorCtx<'g>, array: &Array) -> Value<'g> {
        match array {
            Array::List(values, _) => {
                let values = values
                    .iter()
                    .map(|e| self.visit_exp(ctx, e))
                    .collect::<Vec<_>>();
                let type_ = values[0].type_(&self.ctx);

                let array = ctx
                    .builder
                    .build_alloca(BasicTypeEnum::from(type_.clone()), "")
                    .unwrap();
                Value::Pointer {
                    value: array,
                    ty: self.new_ptr(type_),
                }
            }
            Array::Template(_, _, _) => {
                unimplemented!()
            }
        }
    }

    fn visit_lval(self: &Arc<Self>, ctx: &mut VisitorCtx<'g>, lval: &LVal) -> Value<'g> {
        let name = lval.path.path.join(".");
        if let Some(value) = ctx.symbols.lookup(&name).map(|s| s.value()) {
            value.clone()
        } else if let Some(value) = self.globals.read().unwrap().get(&name) {
            value.clone()
        } else {
            self.query(&name).unwrap()
        }
    }
}
