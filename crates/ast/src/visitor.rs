use crate::*;

pub trait CompUnitVisitor {
    fn visit_comp_unit(&mut self, comp_unit: &CompUnit) {
        for item in comp_unit.global_items.iter() {
            match item {
                GlobalItem::ConstDef(const_def) => self.visit_const_def(const_def),
            }
        }
    }

    fn visit_const_def(&mut self, const_def: &ConstDef);
}

pub trait ExpVisitor<V>: StatementVisitor<V> {
    fn get_right_value(&self, left_value: V) -> V;
    /// This function shouldn't generate instructions like load.
    fn pass_left_value_as_right_value(&self, left_value: V) -> V;

    fn visit_left_value(&mut self, exp: &Exp) -> V {
        match exp {
            Exp::Array(array) => self.visit_array(array),
            Exp::Binary(op, lhs, rhs, span) => {
                let lhs = self.visit_right_value(lhs);
                let rhs = self.visit_right_value(rhs);
                self.visit_binary(op, lhs, rhs, span)
            }
            Exp::Block(block) => self.visit_block(block),
            Exp::Call(call) => self.visit_call(call),
            Exp::Deref(deref) => self.visit_deref(deref),
            Exp::Exp(exp, _) => self.visit_left_value(exp),
            Exp::GetAddr(get_addr) => self.visit_get_addr(get_addr),
            Exp::Index(index) => self.visit_index(index),
            Exp::Var(var) => self.visit_var(var),
            Exp::Number(number) => self.visit_number(number),
            Exp::Str(string, _) => self.visit_str(string),
            Exp::Unary(op, value, span) => {
                let value = self.visit_right_value(value);
                self.visit_unary(op, value, span)
            }
            Exp::ProtoDef(proto_def) => self.visit_proto(proto_def),
            Exp::Function(func) => self.visit_function(func),
            Exp::Assign(assign) => self.visit_assign(assign),
            Exp::Return(return_) => self.visit_return(return_),
            Exp::IfExp(if_exp) => self.visit_if_exp(if_exp),
            Exp::Unit(_) => self.visit_unit(),
            Exp::For(for_) => self.visit_for(for_),
            Exp::Loop(loop_) => self.visit_loop(loop_),
            Exp::While(while_) => self.visit_while(while_),
            Exp::TypeCast(type_cast) => self.visit_type_cast(type_cast),
            Exp::Structure(structure) => self.visit_structure(structure),
            Exp::FieldAccess(field_access) => self.visit_field_access(field_access),
        }
    }

    fn visit_right_value(&mut self, exp: &Exp) -> V {
        let left_value = self.visit_left_value(exp);
        self.get_right_value(left_value)
    }

    fn visit_get_addr(&mut self, get_addr: &GetAddr) -> V {
        let exp = self.visit_left_value(&get_addr.exp);
        self.pass_left_value_as_right_value(exp)
    }

    fn visit_array(&mut self, array: &Array) -> V;
    fn visit_binary(&mut self, op: &BinaryOp, lhs: V, rhs: V, span: &Span) -> V;
    fn visit_unary(&mut self, op: &UnaryOp, value: V, span: &Span) -> V;
    fn visit_call(&mut self, call: &Call) -> V;
    fn visit_deref(&mut self, deref: &Deref) -> V;
    fn visit_index(&mut self, index: &Index) -> V;
    fn visit_var(&mut self, var: &Var) -> V;
    fn visit_number(&mut self, number: &Number) -> V;
    fn visit_str(&mut self, string: &str) -> V;
    fn visit_block(&mut self, block: &Block) -> V;
    fn visit_proto(&mut self, proto_def: &ProtoDef) -> V;
    fn visit_function(&mut self, func: &FunctionDef) -> V;
    fn visit_unit(&mut self) -> V;
    fn visit_type_cast(&mut self, type_cast: &TypeCast) -> V;
    fn visit_structure(&mut self, structure: &Structure) -> V;
    fn visit_field_access(&mut self, field_access: &FieldAccess) -> V;
}

pub trait StatementVisitor<V> {
    fn visit_assign(&mut self, assign: &Assign) -> V;
    fn visit_if_exp(&mut self, if_exp: &IfExp) -> V;
    /// If this returns `Some`, the function returns the value.
    fn visit_return(&mut self, return_stmt: &Return) -> V;
    fn visit_for(&mut self, for_: &For) -> V;
    fn visit_loop(&mut self, loop_: &Loop) -> V;
    fn visit_while(&mut self, while_: &While) -> V;
}

pub trait BlockVisitor<V>: ExpVisitor<V> {
    fn on_enter_block(&mut self);
    fn on_leave_block(&mut self);
    fn visit_block(&mut self, block: &Block) -> Option<V> {
        self.on_enter_block();
        for item in block.items.iter() {
            if let Some(return_value) = match item {
                BlockItem::Statement(stmt) => self.visit_statement(stmt),
                BlockItem::VarDef(var_def) => {
                    self.visit_var_def(var_def);
                    None
                }
            } {
                return Some(return_value);
            }
        }
        let result = block
            .return_value
            .as_ref()
            .map(|e| self.visit_right_value(e));
        self.on_leave_block();
        result
    }

    fn visit_statement(&mut self, stmt: &Statement) -> Option<V> {
        match stmt {
            Statement::Exp(exp) => {
                self.visit_right_value(exp);
                None
            }
            Statement::InlineAsm(inline_asm) => {
                self.visit_inline_asm(inline_asm);
                None
            }
        }
    }
    fn visit_var_def(&mut self, var_def: &VarDef);
    fn visit_inline_asm(&mut self, inline_asm: &InlineAsm);
}
