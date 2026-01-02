use crate::*;

pub trait CompUnitVisitor: DefVisitor {
    fn visit_comp_unit(&mut self, comp_unit: &CompUnit) {
        for item in comp_unit.global_items.iter() {
            match item {
                GlobalItem::ConstDef(const_def) => self.visit_const_def(const_def),
            }
        }
    }
}

pub trait DefVisitor {
    fn visit_const_def(&mut self, const_def: &ConstDef);
    fn visit_function_def(&mut self, func_def: &FunctionDef);
}

pub trait ExpVisitor<V> {
    fn visit_exp(&mut self, exp: &Exp) -> V {
        match exp {
            Exp::Array(array) => self.visit_array(array),
            Exp::Binary(op, lhs, rhs, _) => {
                let lhs = self.visit_exp(lhs);
                let rhs = self.visit_exp(rhs);
                self.visit_binary(op, lhs, rhs)
            }
            Exp::Block(block) => self.visit_block(block),
            Exp::Call(call) => self.visit_call(call),
            Exp::Deref(deref) => self.visit_deref(deref),
            Exp::Exp(exp, _) => self.visit_exp(exp),
            Exp::GetAddr(get_addr) => self.visit_get_addr(get_addr),
            Exp::Index(index) => self.visit_index(index),
            Exp::LVal(lval) => self.visit_lval(lval),
            Exp::Number(number) => self.visit_number(number),
            Exp::Str(string, _) => self.visit_str(string),
            Exp::Unary(op, value, _) => {
                let value = self.visit_exp(value);
                self.visit_unary(op, value)
            }
            Exp::Function(func) => self.visit_function(func),
        }
    }
    fn visit_array(&mut self, array: &Array) -> V;
    fn visit_binary(&mut self, op: &BinaryOp, lhs: V, rhs: V) -> V;
    fn visit_unary(&mut self, op: &UnaryOp, value: V) -> V;
    fn visit_call(&mut self, call: &Call) -> V;
    fn visit_deref(&mut self, deref: &Deref) -> V;
    fn visit_get_addr(&mut self, get_addr: &GetAddr) -> V;
    fn visit_index(&mut self, index: &Index) -> V;
    fn visit_lval(&mut self, lval: &LVal) -> V;
    fn visit_number(&mut self, number: &Number) -> V;
    fn visit_str(&mut self, string: &str) -> V;
    fn visit_block(&mut self, block: &Block) -> V;
    fn visit_function(&mut self, func: &FunctionDef) -> V;
}

pub trait BlockVisitor<V>: ExpVisitor<V> {
    fn on_enter_block(&mut self);
    fn on_leave_block(&mut self);
    fn visit_block(&mut self, block: &Block) -> Option<V> {
        self.on_enter_block();
        for item in block.items.iter() {
            if let Some(return_value) = match item {
                BlockItem::Statement(stmt) => self.visit_statement(stmt),
            } {
                return Some(return_value);
            }
        }
        self.on_leave_block();
        block.return_value.as_ref().map(|e| self.visit_exp(e))
    }

    fn visit_statement(&mut self, stmt: &Statement) -> Option<V> {
        match stmt {
            Statement::Exp(exp) => {
                self.visit_exp(exp);
                None
            }
            Statement::Return(r#return) => self.visit_return(r#return),
        }
    }
    /// If this returns `Some`, the function returns the value.
    fn visit_return(&mut self, return_stmt: &Return) -> Option<V>;
}
