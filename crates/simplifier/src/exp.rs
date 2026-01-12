use ast::{
    Array, Assign, BinaryOp, Call, Deref, Exp, FieldAccess, For, FunctionDef, GetAddr, IfExp,
    Index, Loop, Param, Path, ProtoDef, Return, Span, Structure, TypeCast, UnaryOp, Var, While,
};

use crate::SimplifierContext;

impl SimplifierContext {
    pub fn simp_exp(&mut self, exp: Exp) -> Exp {
        match exp {
            Exp::Type(ty) => {
                let ty = self.simp_type(ty);
                Exp::Type(ty)
            }
            Exp::Array(array) => self.simp_array(*array),
            Exp::Binary(op, lhs, rhs, span) => self.simp_binary(op, *lhs, *rhs, span),
            Exp::Assign(assign) => self.simp_assign(*assign),
            Exp::Block(block) => Exp::Block(Box::new(self.simp_block(*block))),
            Exp::Call(call) => self.simp_call(*call),
            Exp::Deref(deref) => self.simp_deref(*deref),
            Exp::FieldAccess(field_access) => self.simp_field_access(*field_access),
            Exp::For(for_loop) => self.simp_for(*for_loop),
            Exp::IfExp(if_exp) => self.simp_if(*if_exp),
            Exp::Function(func) => self.simp_function(*func),
            Exp::GetAddr(get_addr) => self.simp_get_addr(*get_addr),
            Exp::Index(index) => self.simp_index(*index),
            Exp::Loop(loop_exp) => self.simp_loop(*loop_exp),
            Exp::ProtoDef(proto_def) => self.simp_proto(*proto_def),
            Exp::Return(ret) => self.simp_return(*ret),
            Exp::Structure(structure) => self.simp_structure(*structure),
            Exp::TypeCast(type_cast) => self.simp_type_cast(*type_cast),
            Exp::Var(var) => self.simp_var(*var),
            Exp::While(while_exp) => self.simp_while(*while_exp),
            Exp::Exp(exp, _) => self.simp_exp(*exp),
            Exp::Unary(op, value, span) => self.simp_unary(op, *value, span),
            _ => exp,
        }
    }

    fn simp_array(&mut self, array: Array) -> Exp {
        match array {
            Array::List(values, span) => Exp::Array(Box::new(Array::List(
                values
                    .into_iter()
                    .map(|value| self.simp_exp(value))
                    .collect(),
                span,
            ))),
            _ => unreachable!(),
        }
    }

    fn simp_binary(&mut self, op: BinaryOp, lhs: Exp, rhs: Exp, span: Span) -> Exp {
        let lhs = self.simp_exp(lhs);
        let rhs = self.simp_exp(rhs);
        Exp::Binary(op, Box::new(lhs), Box::new(rhs), span)
    }

    fn simp_unary(&mut self, op: UnaryOp, value: Exp, span: Span) -> Exp {
        let value = self.simp_exp(value);
        Exp::Unary(op, Box::new(value), span)
    }

    fn simp_assign(&mut self, assign: Assign) -> Exp {
        let lhs = self.simp_exp(assign.lhs);
        let rhs = self.simp_exp(assign.rhs);
        Exp::Assign(Box::new(Assign {
            lhs,
            rhs,
            span: assign.span,
        }))
    }

    fn simp_call(&mut self, call: Call) -> Exp {
        let func = self.simp_exp(call.func);
        let args = call
            .args
            .into_iter()
            .map(|arg| self.simp_exp(arg))
            .collect();
        Exp::Call(Box::new(Call {
            func,
            args,
            span: call.span,
        }))
    }

    fn simp_deref(&mut self, deref: Deref) -> Exp {
        let Deref { exp, span } = deref;
        Exp::Deref(Box::new(Deref {
            exp: self.simp_exp(exp),
            span,
        }))
    }

    fn simp_field_access(&mut self, field: FieldAccess) -> Exp {
        let FieldAccess { lhs, field, span } = field;
        Exp::FieldAccess(Box::new(FieldAccess {
            lhs: self.simp_exp(lhs),
            field,
            span,
        }))
    }

    fn simp_for(&mut self, for_exp: For) -> Exp {
        let For {
            var,
            start,
            end,
            step,
            body,
            span,
        } = for_exp;
        let start = self.simp_exp(start);
        let end = self.simp_exp(end);
        let step = step.map(|step| self.simp_exp(step));

        self.locals.pre_push(var.clone());
        let body = self.simp_block(body);

        Exp::For(Box::new(For {
            var,
            start,
            end,
            step,
            body,
            span,
        }))
    }

    fn simp_while(&mut self, while_exp: While) -> Exp {
        let While {
            condition,
            body,
            span,
        } = while_exp;

        let condition = self.simp_exp(condition);
        let body = self.simp_block(body);

        Exp::While(Box::new(While {
            condition,
            body,
            span,
        }))
    }

    fn simp_if(&mut self, if_exp: IfExp) -> Exp {
        let IfExp {
            condition,
            then_branch,
            else_branch,
            else_if,
            span,
        } = if_exp;

        let condition = self.simp_exp(condition);
        let then_branch = self.simp_block(then_branch);
        let else_branch = else_branch.map(|exp| self.simp_block(exp));
        let else_if = else_if.map(|exp| self.simp_if(*exp)).map(|exp| match exp {
            Exp::IfExp(v) => v,
            _ => unreachable!(),
        });

        Exp::IfExp(Box::new(IfExp {
            condition,
            then_branch,
            else_branch,
            else_if,
            span,
        }))
    }

    fn simp_function(&mut self, func: FunctionDef) -> Exp {
        let FunctionDef {
            abi,
            params,
            return_type,
            block,
            span,
        } = func;
        let params = params
            .into_iter()
            .map(|param| {
                let param = self.simp_param(param);
                self.locals.pre_push(param.name.clone());
                param
            })
            .collect();
        let return_type = return_type.map(|ty| self.simp_exp(ty));
        let block = self.simp_block(block);
        Exp::Function(Box::new(FunctionDef {
            abi,
            params,
            return_type,
            block,
            span,
        }))
    }

    fn simp_proto(&mut self, proto: ProtoDef) -> Exp {
        let ProtoDef {
            abi,
            params,
            return_type,
            span,
        } = proto;
        let params = params
            .into_iter()
            .map(|param| self.simp_param(param))
            .collect();
        let return_type = return_type.map(|ty| self.simp_exp(ty));
        Exp::ProtoDef(Box::new(ProtoDef {
            abi,
            params,
            return_type,
            span,
        }))
    }

    fn simp_param(&mut self, param: Param) -> Param {
        let Param {
            name,
            param_type,
            span,
        } = param;
        let param_type = self.simp_exp(param_type);
        Param {
            name,
            param_type,
            span,
        }
    }

    fn simp_get_addr(&mut self, get_addr: GetAddr) -> Exp {
        let GetAddr { exp, span } = get_addr;
        let exp = self.simp_exp(exp);
        Exp::GetAddr(Box::new(GetAddr { exp, span }))
    }

    fn simp_index(&mut self, index: Index) -> Exp {
        let Index { exp, index, span } = index;
        let exp = self.simp_exp(exp);
        let index = self.simp_exp(index);
        Exp::Index(Box::new(Index { exp, index, span }))
    }

    fn simp_loop(&mut self, loop_exp: Loop) -> Exp {
        let Loop { body, span } = loop_exp;
        let body = self.simp_block(body);
        Exp::Loop(Box::new(Loop { body, span }))
    }

    fn simp_return(&mut self, return_exp: Return) -> Exp {
        let Return { value, span } = return_exp;
        let value = value.map(|value| self.simp_exp(value));
        Exp::Return(Box::new(Return { value, span }))
    }

    fn simp_structure(&mut self, structure: Structure) -> Exp {
        let Structure { ty, fields, span } = structure;
        let ty = Box::new(self.simp_exp(*ty));
        let fields = fields
            .into_iter()
            .map(|(name, value)| {
                let value = self.simp_exp(value);
                (name, value)
            })
            .collect();
        Exp::Structure(Box::new(Structure { ty, fields, span }))
    }

    fn simp_type_cast(&mut self, type_cast: TypeCast) -> Exp {
        let TypeCast { exp, ty, span } = type_cast;
        let exp = self.simp_exp(exp);
        let ty = self.simp_exp(ty);
        Exp::TypeCast(Box::new(TypeCast { exp, ty, span }))
    }

    fn simp_var(&mut self, var: Var) -> Exp {
        let Var { path, span } = var;
        let start = path.path[0].clone();
        let mut new_path = if self.globals.lookup_current(start) {
            self.globals.prefixes()
        } else {
            vec![]
        };
        new_path.extend(path.path);
        Exp::Var(Box::new(Var {
            path: Path {
                path: new_path,
                span: path.span,
            },
            span,
        }))
    }
}
