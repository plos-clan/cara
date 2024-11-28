use inkwell::IntPredicate;

use comptime::ComptimeEvaluate;

use super::*;

impl<'gen> GenerateProgramOnce<'gen> for ConstExp {
    type Out = Value<'gen>;

    fn generate(&self, gen: Arc<RwLock<Generator<'gen>>>) -> Result<Self::Out> {
        self.exp.evaluate(gen)
    }
}

impl<'gen> GenerateProgramOnce<'gen> for Exp {
    type Out = Value<'gen>;

    fn generate(&self, gen: Arc<RwLock<Generator<'gen>>>) -> Result<Self::Out> {
        Ok(match self {
            Self::Exp(exp, _) => exp.generate(gen)?,
            Self::Unary(op, exp, span) => {
                let value = exp
                    .generate(gen.clone())?
                    .as_int(span.clone(), gen.read().unwrap().context)?;
                Value::new_int(match op {
                    UnaryOp::Negative => gen
                        .read()
                        .unwrap()
                        .builder
                        .build_int_neg(value, "")
                        .unwrap(),
                    UnaryOp::Not => gen.read().unwrap().builder.build_not(value, "").unwrap(),
                    UnaryOp::Positive => value,
                })
            }
            Self::Number(number) => number.generate(gen)?,
            Self::LVal(lval) => lval.generate(gen)?,
            Self::Binary(lhs, op, rhs, span) => {
                let ctx = gen.read().unwrap().context;

                let lhs = lhs.generate(gen.clone())?.as_int(span.clone(), ctx)?;
                let rhs = rhs.generate(gen.clone())?.as_int(span.clone(), ctx)?;

                Value::new_int(match op {
                    BinaryOp::Add => gen
                        .read()
                        .unwrap()
                        .builder
                        .build_int_add(lhs, rhs, "")
                        .unwrap(),
                    BinaryOp::Sub => gen
                        .read()
                        .unwrap()
                        .builder
                        .build_int_sub(lhs, rhs, "")
                        .unwrap(),
                    BinaryOp::Mul => gen
                        .read()
                        .unwrap()
                        .builder
                        .build_int_mul(lhs, rhs, "")
                        .unwrap(),
                    BinaryOp::Div => gen
                        .read()
                        .unwrap()
                        .builder
                        .build_int_unsigned_div(lhs, rhs, "")
                        .unwrap(),
                    BinaryOp::Mod => gen
                        .read()
                        .unwrap()
                        .builder
                        .build_int_unsigned_rem(lhs, rhs, "")
                        .unwrap(),
                    BinaryOp::Le => gen
                        .read()
                        .unwrap()
                        .builder
                        .build_int_compare(IntPredicate::SLE, lhs, rhs, "")
                        .unwrap(),
                    BinaryOp::Lt => gen
                        .read()
                        .unwrap()
                        .builder
                        .build_int_compare(IntPredicate::SLT, lhs, rhs, "")
                        .unwrap(),
                    BinaryOp::Ge => gen
                        .read()
                        .unwrap()
                        .builder
                        .build_int_compare(IntPredicate::SGE, lhs, rhs, "")
                        .unwrap(),
                    BinaryOp::Gt => gen
                        .read()
                        .unwrap()
                        .builder
                        .build_int_compare(IntPredicate::SGT, lhs, rhs, "")
                        .unwrap(),
                    BinaryOp::Eq => gen
                        .read()
                        .unwrap()
                        .builder
                        .build_int_compare(IntPredicate::EQ, lhs, rhs, "")
                        .unwrap(),
                    BinaryOp::Neq => gen
                        .read()
                        .unwrap()
                        .builder
                        .build_int_compare(IntPredicate::NE, lhs, rhs, "")
                        .unwrap(),
                })
            }
            _ => unimplemented!(),
        })
    }
}

impl<'gen> GenerateProgramOnce<'gen> for Number {
    type Out = Value<'gen>;

    fn generate(&self, _gen: Arc<RwLock<Generator<'gen>>>) -> Result<Self::Out> {
        Ok(Value::new_comp_int(self.num))
    }
}

impl<'gen> GenerateProgramOnce<'gen> for LVal {
    type Out = Value<'gen>;

    fn generate(&self, gen: Arc<RwLock<Generator<'gen>>>) -> Result<Self::Out> {
        let gen = gen.read().unwrap();
        
        let id = self.ids[0].clone();
        let symbol = gen
            .get_symbol(&id)
            .ok_or(CompileError::new_symbol_not_found(self.span.clone(),id))?;
        
        Ok(
            match symbol {
                Symbol::Const(name, value) => value.clone(),
                _ => unimplemented!(),
            }
        )
    }
}
