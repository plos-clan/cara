use std::{
    collections::HashMap,
    iter::zip,
    ops::Deref,
    sync::{LazyLock, RwLock},
};

use ast::{
    Array, BinaryOp, ConstInitialValue, Span, TypeCast, UnaryOp,
    visitor::{BlockVisitor, ExpVisitor},
};
use query::DefId;

use crate::{
    AnalyzerContext, Error, Symbol, Type, Value, get_analyzer_type,
    queries::{AnalyzeResult, CHECK_CONST_DEF},
    try_infer,
};

impl ExpVisitor<Value> for AnalyzerContext<'_> {
    fn get_right_value(&self, left_value: Value) -> Value {
        left_value
    }

    fn pass_left_value_as_right_value(&self, left_value: Value) -> Value {
        left_value
    }

    fn visit_get_addr(&mut self, get_addr: &ast::GetAddr) -> Value {
        let ty = self.visit_right_value(&get_addr.exp).into_type();
        Value::new(ty.pointer())
    }

    fn visit_array(&mut self, array: &ast::Array) -> Value {
        match array {
            Array::List(values, span) => {
                let types = values
                    .iter()
                    .map(|v| self.visit_right_value(v))
                    .collect::<Vec<_>>();
                let should_be_type = types[0].clone();
                for type_ in types.iter().skip(1) {
                    if *type_ != should_be_type {
                        self.error_at(
                            Error::TypeMismatch(
                                should_be_type.type_().clone(),
                                type_.type_().clone(),
                            ),
                            *span,
                        );
                    }
                }
                Value::new(should_be_type.type_().array(values.len() as u32))
            }
            _ => unimplemented!(),
        }
    }

    fn visit_binary(&mut self, op: &BinaryOp, lhs: Value, rhs: Value, span: &Span) -> Value {
        let result = match op {
            BinaryOp::Add => lhs.check_add(&rhs),
            BinaryOp::Sub => lhs.check_sub(&rhs),
            BinaryOp::Mul => lhs.check_mul(&rhs),
            BinaryOp::Div => lhs.check_div(&rhs),
            BinaryOp::Mod => lhs.check_mod(&rhs),
            BinaryOp::LShift => lhs.check_lshift(&rhs),
            BinaryOp::RShift => lhs.check_rshift(&rhs),
            BinaryOp::Eq => lhs.check_eq(&rhs),
            BinaryOp::Ne => lhs.check_neq(&rhs),
            BinaryOp::Lt => lhs.check_lt(&rhs),
            BinaryOp::Le => lhs.check_le(&rhs),
            BinaryOp::Gt => lhs.check_gt(&rhs),
            BinaryOp::Ge => lhs.check_ge(&rhs),
            BinaryOp::And => lhs.check_and(&rhs),
            BinaryOp::Or => lhs.check_or(&rhs),
        };
        match result {
            Ok(value) => value,
            Err(err) => {
                self.error_at(err, *span);
                lhs
            }
        }
    }

    fn visit_block(&mut self, block: &ast::Block) -> Value {
        <Self as BlockVisitor<_>>::visit_block(self, block).unwrap_or(Value::new(Type::Unit))
    }

    fn visit_call(&mut self, call: &ast::Call) -> Value {
        let func = self.visit_right_value(&call.func);
        if let Type::Function(ret_ty, param_types) = func.type_() {
            for (arg, param_ty) in zip(call.args.iter(), param_types.iter()) {
                let param_ty = param_ty.clone();
                let arg_ty = self.visit_right_value(arg).into_type();
                if arg_ty != param_ty {
                    self.error_at(Error::TypeMismatch(param_ty, arg_ty), arg.span());
                }
            }

            Value::new(ret_ty.deref().clone())
        } else {
            self.error_at(Error::WrongCall(func.type_().clone()), call.span);
            Value::new(Type::Unit)
        }
    }

    fn visit_deref(&mut self, deref: &ast::Deref) -> Value {
        let value = self.visit_right_value(&deref.exp);
        let type_ = value.type_();
        if let Type::Ptr(target) = type_ {
            Value::new(target.deref().clone())
        } else {
            self.error_at(Error::WrongDeref(type_.clone()), deref.exp.span());
            Value::new(Type::Unit)
        }
    }

    fn visit_function(&mut self, _func: &ast::FunctionDef) -> Value {
        unreachable!()
    }

    fn visit_index(&mut self, index: &ast::Index) -> Value {
        let array = self.visit_left_value(&index.exp);
        let index_value = self.visit_right_value(&index.index);

        if !matches!(index_value.type_(), Type::Signed(_) | Type::Unsigned(_)) {
            self.error_at(
                Error::TypeMismatch(Type::Unsigned(64), index_value.type_().clone()),
                index.index.span(),
            );
        }

        match array.type_() {
            Type::Array(target, _) => Value::new(target.deref().clone()),
            Type::Ptr(target) => Value::new(target.deref().clone()),
            _ => {
                self.error_at(Error::WrongDeref(array.type_().clone()), index.exp.span());
                Value::new(Type::Unit)
            }
        }
    }

    fn visit_number(&mut self, number: &ast::Number) -> Value {
        if let Some((signed, width)) = number.ty {
            Value::new(if signed {
                Type::Signed(width)
            } else {
                Type::Unsigned(width)
            })
        } else {
            Value::new(Type::Signed(32))
        }
    }

    fn visit_proto(&mut self, _proto_def: &ast::ProtoDef) -> Value {
        unreachable!()
    }

    fn visit_str(&mut self, _string: &str) -> Value {
        Value::new(Type::Signed(8).pointer())
    }

    fn visit_type_cast(&mut self, type_cast: &TypeCast) -> Value {
        let TypeCast { exp, ty, span } = type_cast;
        let value = self.visit_right_value(exp);
        let value_type = value.type_();
        let target = get_analyzer_type(ty);

        if *value_type == target {
            return value;
        }

        if value_type.is_int() && target.is_int() {
            Value::new(target)
        } else if value_type.is_bool() && target.is_int() {
            Value::new(target)
        } else if value_type.is_ptr() && target.is_ptr() {
            Value::new(target)
        } else if value_type.is_ptr() && target.is_int() {
            Value::new(target)
        } else if value_type.is_int() && target.is_ptr() {
            Value::new(target)
        } else if value_type.is_function() && target.is_ptr() {
            Value::new(target)
        } else {
            self.error_at(
                Error::InvalidTypeCast(value_type.clone(), target.clone()),
                *span,
            );
            Value::new(target)
        }
    }

    fn visit_unary(&mut self, op: &UnaryOp, value: Value, span: &Span) -> Value {
        let result = match op {
            UnaryOp::Pos => value.check_pos(),
            UnaryOp::Neg => value.check_neg(),
            UnaryOp::Not => value.check_not(),
        };
        match result {
            Ok(value) => value,
            Err(err) => {
                self.error_at(err, *span);
                Value::new(Type::Unit)
            }
        }
    }

    fn visit_unit(&mut self) -> Value {
        Value::new(Type::Unit)
    }

    fn visit_var(&mut self, var: &ast::Var) -> Value {
        let name = var.path.path.join(".");
        if let Some(symbol) = self.symbols.lookup(&name) {
            match symbol {
                Symbol::Var(_, _, value) => value.clone(),
            }
        } else {
            static CHECKED: LazyLock<RwLock<HashMap<DefId, Value>>> =
                LazyLock::new(|| RwLock::new(HashMap::new()));

            let def_id = self.ctx.lookup_def_id(&name).unwrap();

            if let Some(value) = CHECKED.read().unwrap().get(&def_id) {
                value.clone()
            } else if let Some(ty) = try_infer({
                let const_def = self.ctx.get_def(def_id).unwrap();
                let ConstInitialValue::Exp(exp) = &const_def.initial_value;
                &exp.exp
            }) {
                self.required.push(def_id);
                Value::new(ty)
            } else {
                let result = self.ctx.query(&CHECK_CONST_DEF, def_id).unwrap();

                let AnalyzeResult {
                    value,
                    errors,
                    warnings,
                    required,
                } = result;

                self.errors.extend(errors);
                self.warnings.extend(warnings);
                self.required.extend(required);

                CHECKED.write().unwrap().insert(def_id, value.clone());

                value
            }
        }
    }
}
