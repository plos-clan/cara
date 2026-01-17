use ast::{Array, Span, StructType, TypeEnum, visitor::ExpVisitor};
use const_eval::{ValueKind, queries::CONST_EVAL_PROVIDER};

use crate::MonomorphizeContext;

impl ExpVisitor<()> for MonomorphizeContext {
    fn ast_ctx(&self) -> std::sync::Arc<ast::AstContext> {
        self.ctx.ast_ctx()
    }
    fn get_right_value(&self, _left_value: ()) {}
    fn pass_left_value_as_right_value(&self, _left_value: ()) {}

    fn visit_array(&mut self, array: &Array) {
        match array {
            Array::List(elements, _) => {
                for element in elements {
                    self.visit_right_value(*element);
                }
            }
            _ => unimplemented!(),
        }
    }

    fn visit_binary(&mut self, _op: &ast::BinaryOp, _lhs: (), _rhs: (), _: &Span) {}

    fn visit_block(&mut self, block: &ast::Block) {
        <Self as ast::visitor::BlockVisitor<()>>::visit_block(self, block);
    }

    fn visit_call(&mut self, call: &ast::Call) {
        for &arg in &call.args {
            self.visit_right_value(arg);
        }
        self.visit_right_value(call.func)
    }

    fn visit_deref(&mut self, deref: &ast::Deref) {
        self.visit_left_value(deref.exp);
    }

    fn visit_proto(&mut self, _proto_def: &ast::ProtoDef) {
        unreachable!()
    }

    fn visit_function(&mut self, _func: &ast::FunctionDef) {
        unreachable!()
    }

    fn visit_index(&mut self, index: &ast::Index) {
        self.visit_left_value(index.exp);
        self.visit_right_value(index.index);
    }

    fn visit_number(&mut self, _number: &ast::Number) {}

    fn visit_str(&mut self, _string: &str) {}

    fn visit_unary(&mut self, _op: &ast::UnaryOp, _value: (), _: &Span) {}

    fn visit_unit(&mut self) {}

    fn visit_var(&mut self, var: &ast::Var) {
        let name = var.path.path.join("::");
        if !self.locals.contains(&name) {
            let def_id = self.ctx.lookup_def_id(name).unwrap();
            let result = self.ctx.query_cached(&CONST_EVAL_PROVIDER, def_id).unwrap();
            if matches!(result.kind(), ValueKind::Function(_) | ValueKind::Proto(_)) {
                self.required_items.push(def_id);
            }
        }
    }

    fn visit_type_cast(&mut self, type_cast: &ast::TypeCast) {
        self.visit_right_value(type_cast.exp);
    }

    fn visit_structure(&mut self, structure: &ast::Structure) {
        self.visit_right_value(structure.ty);
        for (_, &value) in structure.fields.iter() {
            self.visit_right_value(value);
        }
    }

    fn visit_field_access(&mut self, field_access: &ast::FieldAccess) {
        self.visit_right_value(field_access.lhs);
    }

    fn visit_type(&mut self, type_: &ast::Type) {
        if let TypeEnum::Structure(StructType { fields, .. }) = &type_.kind {
            for (_, &ty) in fields.iter() {
                self.visit_right_value(ty);
            }
        }
    }
}
