use const_eval::queries::CONST_EVAL_PROVIDER;
use inkwell::{
    IntPredicate,
    types::BasicTypeEnum,
    values::{AnyValue, BasicValue, InstructionOpcode},
};

use ast::{Array, Assign, BinaryOp, Call, Var, visitor::ExpVisitor};

use crate::{
    LLVM_CONTEXT, VisitorCtx,
    info::{Symbol, Value},
};

impl<'v> ExpVisitor<Value<'v>> for VisitorCtx<'v> {
    fn get_right_value(&self, left_value: Value<'v>) -> Value<'v> {
        left_value.as_right_value(&self.builder)
    }

    fn visit_binary(&mut self, op: &BinaryOp, lhs_: Value<'v>, rhs_: Value<'v>) -> Value<'v> {
        let lhs = lhs_.as_basic_value_enum();
        let rhs = rhs_.as_basic_value_enum();

        let builder = &self.builder;
        let op_code = match op {
            BinaryOp::Add => InstructionOpcode::Add,
            BinaryOp::Sub => InstructionOpcode::Sub,
            BinaryOp::Mul => InstructionOpcode::Mul,
            BinaryOp::Div => InstructionOpcode::UDiv,
            BinaryOp::Mod => InstructionOpcode::URem,
            BinaryOp::And => InstructionOpcode::And,
            BinaryOp::Or => InstructionOpcode::Or,
            BinaryOp::LShift => InstructionOpcode::Shl,
            BinaryOp::RShift => InstructionOpcode::LShr,
            _ => {
                let cmp = match op {
                    BinaryOp::Lt => IntPredicate::SLT,
                    BinaryOp::Le => IntPredicate::SLE,
                    BinaryOp::Gt => IntPredicate::SGT,
                    BinaryOp::Ge => IntPredicate::SGE,
                    BinaryOp::Eq => IntPredicate::EQ,
                    BinaryOp::Ne => IntPredicate::NE,
                    _ => unreachable!(),
                };
                return Value::Int(
                    builder
                        .build_int_compare(cmp, lhs.into_int_value(), rhs.into_int_value(), "")
                        .unwrap(),
                );
            }
        };

        let result = builder.build_binop(op_code, lhs, rhs, "").unwrap();

        Value::new_from(result.as_any_value_enum(), lhs_.type_())
    }

    fn visit_block(&mut self, block: &ast::Block) -> Value<'v> {
        use ast::visitor::BlockVisitor;
        <Self as BlockVisitor<Value<'v>>>::visit_block(self, block).unwrap_or(Value::Unit)
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
        let ty = if let Some((_, width)) = number.ty {
            LLVM_CONTEXT.custom_width_int_type(width)
        } else {
            LLVM_CONTEXT.i32_type()
        };
        Value::Int(ty.const_int(number.num, true))
    }

    fn visit_str(&mut self, _string: &str) -> Value<'v> {
        unimplemented!()
    }

    fn visit_unary(&mut self, op: &ast::UnaryOp, value: Value<'v>) -> Value<'v> {
        let value = value.as_int(&self.builder);
        Value::Int(match op {
            ast::UnaryOp::Neg => self.builder.build_int_neg(value, "").unwrap(),
            ast::UnaryOp::Pos => value,
            ast::UnaryOp::Not => self.builder.build_not(value, "").unwrap(),
        })
    }

    fn visit_call(&mut self, call: &Call) -> Value<'v> {
        let Value::Function(func, ret_ty) = self.visit_right_value(&call.func) else {
            unreachable!()
        };
        let args = call
            .args
            .iter()
            .map(|arg| self.visit_right_value(arg).into())
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
                let values = values
                    .iter()
                    .map(|e| self.visit_right_value(e))
                    .collect::<Vec<_>>();
                let type_ = values[0].type_();

                let array = self
                    .builder
                    .build_alloca(BasicTypeEnum::from(type_.clone()), "")
                    .unwrap();
                Value::Pointer {
                    value: array,
                    ty: type_.new_ptr(),
                }
            }
            Array::Template(_, _, _) => {
                unimplemented!()
            }
        }
    }

    fn visit_var(&mut self, var: &Var) -> Value<'v> {
        let name = var.path.path.join(".");
        if let Some(symbol) = self.symbols.lookup(&name) {
            match symbol {
                Symbol::MutableVar(_, ptr) => ptr.clone(),
                Symbol::ImmutableVar(_, value) => value.clone(),
            }
        } else {
            let def_id = self.queries.lookup_def_id(&name).unwrap();

            let value = self.queries.query(&CONST_EVAL_PROVIDER, def_id).unwrap();

            match value {
                const_eval::Value::Function(_) => self.global_funcs.get(&def_id).unwrap().clone(),
                const_eval::Value::Int((signed, width), i) => Value::Int(
                    LLVM_CONTEXT
                        .custom_width_int_type(width)
                        .const_int(i as u64, signed),
                ),
                const_eval::Value::Unit => Value::Unit,
            }
        }
    }

    fn visit_function(&mut self, _func: &ast::FunctionDef) -> Value<'v> {
        unreachable!()
    }

    fn visit_unit(&mut self) -> Value<'v> {
        Value::Unit
    }

    fn visit_assign(&mut self, assign: &Assign) -> Value<'v> {
        let lhs = self.visit_left_value(&assign.lhs);
        let rhs = self.visit_right_value(&assign.rhs);

        if let Value::Unit = rhs {
            return Value::Unit;
        }

        let ptr = lhs.get_pointer();
        self.builder.build_store(ptr, rhs).unwrap();

        Value::Unit
    }

    fn visit_return(&mut self, ret: &ast::Return) -> Value<'v> {
        if let Some(value) = ret.value.as_ref() {
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
        let condition = self
            .visit_right_value(&if_exp.condition)
            .as_int(&self.builder);
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
        self.builder.build_unconditional_branch(end_block).unwrap();

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
        self.builder.build_unconditional_branch(end_block).unwrap();

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
}
