use const_eval::queries::CONST_EVAL_PROVIDER;
use inkwell::{
    IntPredicate,
    values::{AnyValue, BasicValue, InstructionOpcode},
};

use ast::{Array, Assign, BinaryOp, Call, Var, visitor::ExpVisitor};
use uuid::Uuid;

use crate::{
    LLVM_CONTEXT, VisitorCtx,
    info::{Symbol, TypeKind, Value},
};

impl<'v> ExpVisitor<Value<'v>> for VisitorCtx<'v> {
    fn get_right_value(&self, left_value: Value<'v>) -> Value<'v> {
        left_value.as_right_value(&self.builder)
    }

    fn pass_left_value_as_right_value(&self, left_value: Value<'v>) -> Value<'v> {
        match left_value {
            Value::Alloca { .. } => left_value.convert_to_right_value(),
            _ => self
                .create_entry_bb_alloca_with_init("", left_value)
                .convert_to_right_value(),
        }
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

    fn visit_deref(&mut self, deref: &ast::Deref) -> Value<'v> {
        let ptr = self.visit_right_value(&deref.exp);
        if matches!(ptr, Value::Unit) {
            return Value::Unit;
        }
        let ty = ptr.type_();
        let pointee_ty = ty.derefed();

        if pointee_ty.is_unit() {
            return Value::Unit;
        }

        let result = self
            .builder
            .build_load(pointee_ty.clone(), ptr.get_pointer(), "")
            .unwrap();
        Value::new_from(result.as_any_value_enum(), pointee_ty)
    }

    fn visit_index(&mut self, index_node: &ast::Index) -> Value<'v> {
        let ptr_value = self.visit_left_value(&index_node.exp);
        let index = self.visit_right_value(&index_node.index);

        let pointee_ty = ptr_value.type_().derefed().derefed();
        let ptr = ptr_value.as_basic_value_enum().into_pointer_value();

        let ptr = unsafe {
            self.builder
                .build_gep(pointee_ty.clone(), ptr, &[index.as_int(&self.builder)], "")
                .unwrap()
        };
        Value::Alloca {
            value: ptr,
            value_ty: pointee_ty,
        }
    }

    fn visit_number(&mut self, number: &ast::Number) -> Value<'v> {
        let ty = if let Some((_, width)) = number.ty {
            LLVM_CONTEXT.custom_width_int_type(width)
        } else {
            LLVM_CONTEXT.i32_type()
        };
        Value::Int(ty.const_int(number.num, true))
    }

    fn visit_str(&mut self, string: &str) -> Value<'v> {
        let string = LLVM_CONTEXT.const_string(string.as_bytes(), true);
        let global = self.module.add_global(
            string.get_type(),
            None,
            &format!("alloc_{}", Uuid::new_v4()),
        );
        Value::Pointer {
            value: global.as_pointer_value(),
            ty: TypeKind::new_int(8).new_ptr(),
        }
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
        let result = self.builder.build_call(func, &args, "").unwrap();

        Value::new_from(result.as_any_value_enum(), ret_ty)
    }

    fn visit_array(&mut self, array: &Array) -> Value<'v> {
        match array {
            Array::List(values, _) => {
                let values = values
                    .iter()
                    .map(|e| self.visit_right_value(e))
                    .collect::<Vec<_>>();
                let ty = values[0].type_();

                ty.const_array(&values)
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
                Symbol::Var(_, value) => value.clone(),
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

    fn visit_loop(&mut self, loop_: &ast::Loop) -> Value<'v> {
        let current_fn = self.current_fn.as_fn();

        let loop_block = LLVM_CONTEXT.append_basic_block(current_fn, "loop");
        let end_block = LLVM_CONTEXT.append_basic_block(current_fn, "end");

        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(loop_block);
        self.visit_block(&loop_.body);
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(end_block);
        Value::Unit
    }

    fn visit_for(&mut self, for_: &ast::For) -> Value<'v> {
        let current_fn = self.current_fn.as_fn();

        let start = self.visit_right_value(&for_.start);
        let end = self.visit_right_value(&for_.end);
        let step = for_
            .step
            .as_ref()
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
                alloca.as_right_value(&self.builder).as_int(&self.builder),
                end.as_int(&self.builder),
                "",
            )
            .unwrap();
        self.builder
            .build_conditional_branch(condition, loop_block, end_block)
            .unwrap();

        self.builder.position_at_end(loop_block);
        self.visit_block(&for_.body);
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
            .build_store(alloca.get_pointer(), new_value)
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
        let condition = self.visit_right_value(&while_.condition);
        self.builder
            .build_conditional_branch(condition.as_int(&self.builder), loop_block, end_block)
            .unwrap();

        self.builder.position_at_end(loop_block);
        self.visit_block(&while_.body);
        self.builder
            .build_unconditional_branch(condition_block)
            .unwrap();

        self.builder.position_at_end(end_block);

        Value::Unit
    }
}
