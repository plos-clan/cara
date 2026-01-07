use ast::visitor::StatementVisitor;

use crate::{ConstEvalContext, Value};

impl StatementVisitor<Value> for ConstEvalContext<'_> {
    fn visit_assign(&mut self, _assign: &ast::Assign) -> Value {
        Value::Unit
    }
    
    fn visit_return(&mut self, _return_stmt: &ast::Return) -> Value {
        unimplemented!()
    }

    fn visit_if_exp(&mut self, _if_exp: &ast::IfExp) -> Value {
        unimplemented!()
    }

    fn visit_for(&mut self, _for_: &ast::For) -> Value {
        unimplemented!()
    }

    fn visit_loop(&mut self, _loop_: &ast::Loop) -> Value {
        unimplemented!()
    }
    
    fn visit_while(&mut self, _while_: &ast::While) -> Value {
        unimplemented!()
    }
}
