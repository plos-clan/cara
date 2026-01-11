use const_eval::queries::CONST_EVAL_PROVIDER;
use inkwell::{
    IntPredicate,
    module::Linkage,
    values::{AnyValue, BasicValue, InstructionOpcode},
};

use ast::{Array, BinaryOp, Call, Span, Var, visitor::ExpVisitor};
use query::DefId;
use uuid::Uuid;

use crate::{
    LLVM_CONTEXT, VisitorCtx,
    info::{Symbol, TypeKind, Value},
    types::get_llvm_type,
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

    fn visit_binary(
        &mut self,
        op: &BinaryOp,
        lhs_: Value<'v>,
        rhs_: Value<'v>,
        _: &Span,
    ) -> Value<'v> {
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
        self.visit_right_value(&deref.exp)
    }

    fn visit_index(&mut self, index_node: &ast::Index) -> Value<'v> {
        let ptr_value = self.visit_left_value(&index_node.exp);
        let index = self.visit_right_value(&index_node.index);

        let pointee_ty = ptr_value.type_().derefed().derefed();
        let ptr = ptr_value.as_basic_value_enum().into_pointer_value();

        let ptr = unsafe {
            self.builder
                .build_gep(pointee_ty.clone(), ptr, &[index.as_int()], "")
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
        global.set_unnamed_addr(true);
        global.set_initializer(&string);
        global.set_linkage(Linkage::Private);
        global.set_constant(true);
        global.set_alignment(1);
        Value::Pointer {
            value: global.as_pointer_value(),
            ty: TypeKind::new_int(8).new_ptr(),
        }
    }

    fn visit_unary(&mut self, op: &ast::UnaryOp, value: Value<'v>, _: &Span) -> Value<'v> {
        let value = value.as_int();
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
        let name = var.path.path.join("::");
        if let Some(symbol) = self.symbols.lookup(&name) {
            match symbol {
                Symbol::Var(_, value) => value.clone(),
            }
        } else {
            let def_id = self.queries.lookup_def_id(&name).unwrap();

            let value = self
                .queries
                .query_cached(&CONST_EVAL_PROVIDER, def_id)
                .unwrap();

            self.const_value_to_llvm_value(Some(def_id), &value)
        }
    }

    fn visit_proto(&mut self, _proto_def: &ast::ProtoDef) -> Value<'v> {
        unreachable!()
    }

    fn visit_function(&mut self, _func: &ast::FunctionDef) -> Value<'v> {
        unreachable!()
    }

    fn visit_type_cast(&mut self, type_cast: &ast::TypeCast) -> Value<'v> {
        let value = self.visit_right_value(&type_cast.exp);
        let target_ty = get_llvm_type(self.queries.clone(), &type_cast.ty);

        if value.is_int() && target_ty.is_int() {
            Value::Int(
                self.builder
                    .build_int_cast(value.as_int(), target_ty.as_int_type(), "")
                    .unwrap(),
            )
        } else if value.is_ptr() && target_ty.is_ptr() {
            Value::Pointer {
                value: value.as_ptr(),
                ty: target_ty,
            }
        } else if value.is_ptr() && target_ty.is_int() {
            Value::Int(
                self.builder
                    .build_ptr_to_int(value.as_ptr(), target_ty.as_int_type(), "")
                    .unwrap(),
            )
        } else if value.is_int() && target_ty.is_ptr() {
            Value::Pointer {
                value: self
                    .builder
                    .build_int_to_ptr(value.as_int(), target_ty.as_ptr_type(), "")
                    .unwrap(),
                ty: target_ty,
            }
        } else {
            unimplemented!()
        }
    }

    fn visit_unit(&mut self) -> Value<'v> {
        Value::Unit
    }

    fn visit_field_access(&mut self, field_access: &ast::FieldAccess) -> Value<'v> {
        let value = self.visit_left_value(&field_access.lhs);
        let value_ty = value.type_().derefed();
        let value = value.as_ptr();

        let TypeKind::Structure {
            field_ids,
            field_types,
            ..
        } = value_ty
        else {
            panic!("Invalid type {:?}", value_ty)
        };

        let field_id = *field_ids
            .iter()
            .find(|(_, name)| *name == &field_access.field)
            .unwrap()
            .0;
        let field_type = field_types[field_id].clone();

        let field_ptr = unsafe {
            self.builder.build_in_bounds_gep(
                (*field_type).clone(),
                value,
                &[TypeKind::new_int(64).const_int(field_id as i64).as_int()],
                "",
            )
        }
        .unwrap();

        Value::Alloca {
            value: field_ptr,
            value_ty: (*field_type).clone(),
        }
    }

    fn visit_structure(&mut self, structure: &ast::Structure) -> Value<'v> {
        let ty = get_llvm_type(self.queries.clone(), &structure.ty);
        let TypeKind::Structure { field_ids, .. } = &ty else {
            unreachable!()
        };

        let mut field_values = Vec::new();
        for id in 0..field_ids.len() {
            let name = field_ids[&id].clone();
            let field_value = self.visit_right_value(&structure.fields[&name]);
            field_values.push(field_value.as_basic_value_enum());
        }

        Value::Structure {
            value: LLVM_CONTEXT.const_struct(&field_values, false),
            ty,
        }
    }
}

impl<'v> VisitorCtx<'v> {
    fn const_value_to_llvm_value(
        &mut self,
        def_id: Option<DefId>,
        value: &const_eval::Value,
    ) -> Value<'v> {
        match value.kind() {
            const_eval::ValueKind::Function(_) | const_eval::ValueKind::Proto(_) => {
                self.global_funcs.get(&def_id.unwrap()).unwrap().clone()
            }
            const_eval::ValueKind::Int(i) => {
                get_llvm_type(self.queries.clone(), value.ty().as_ref()).const_int(i)
            }
            const_eval::ValueKind::Unit => Value::Unit,
            const_eval::ValueKind::Type(ty) => {
                Value::Type(get_llvm_type(self.queries.clone(), &ty))
            }
            const_eval::ValueKind::Structure(ty, fields) => {
                let ty = get_llvm_type(self.queries.clone(), &ty);
                let TypeKind::Structure { field_ids, .. } = &ty else {
                    unreachable!()
                };

                let mut field_values = Vec::new();
                for id in 0..field_ids.len() {
                    let name = field_ids.get(&id).unwrap();
                    let value = self
                        .const_value_to_llvm_value(None, fields.get(name).unwrap())
                        .as_basic_value_enum();
                    field_values.push(value);
                }

                Value::Structure {
                    value: LLVM_CONTEXT.const_struct(&field_values, false),
                    ty,
                }
            }
        }
    }
}
