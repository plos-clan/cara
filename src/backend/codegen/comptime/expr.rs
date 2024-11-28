use super::*;
use inkwell::{IntPredicate, values::IntValue};

impl<'gen> ComptimeEvaluate<'gen> for Exp {
    type Out = Value<'gen>;

    fn evaluate(&self, gen: Arc<RwLock<Generator<'gen>>>) -> Result<Self::Out> {
        Ok(match self {
            Self::Exp(exp, _) => exp.evaluate(gen)?,
            Self::Unary(op, rhs, span) => {
                let ctx = gen.read().unwrap().context;
                let rhs = rhs.evaluate(gen)?.as_int(span.clone(), &ctx)?;
                Value::new_int(match op {
                    UnaryOp::Negative => rhs.const_neg(),
                    UnaryOp::Not => rhs.const_not(),
                    UnaryOp::Positive => rhs,
                })
            }
            Self::Binary(lhs, op, rhs, span) => {
                let ctx = gen.read().unwrap().context;
                let lhs = lhs.evaluate(gen.clone())?.as_int(span.clone(), &ctx)?;
                let rhs = rhs.evaluate(gen.clone())?.as_int(span.clone(), &ctx)?;
                Value::new_int(match op {
                    BinaryOp::Add => lhs.const_add(rhs),
                    BinaryOp::Sub => lhs.const_sub(rhs),
                    BinaryOp::Mul => lhs.const_mul(rhs),
                    BinaryOp::Div => const_div(ctx, lhs, rhs),
                    BinaryOp::Mod => {
                        let t = const_div(ctx, lhs, rhs);
                        lhs.const_nsw_sub(t.const_nsw_mul(rhs))
                    }
                    BinaryOp::Eq => lhs.const_int_compare(IntPredicate::EQ, rhs),
                    BinaryOp::Neq => lhs.const_int_compare(IntPredicate::NE, rhs),
                    BinaryOp::Lt => lhs.const_int_compare(IntPredicate::ULT, rhs),
                    BinaryOp::Gt => lhs.const_int_compare(IntPredicate::UGT, rhs),
                    BinaryOp::Le => lhs.const_int_compare(IntPredicate::ULE, rhs),
                    BinaryOp::Ge => lhs.const_int_compare(IntPredicate::UGE, rhs),
                })
            }
            Self::Number(number) => number.evaluate(gen)?,
            Self::LVal(lval) => lval.evaluate(gen)?,
            _ => unimplemented!(),
        })
    }
}

fn const_div<'a>(ctx: &'a Context, lhs: IntValue<'a>, rhs: IntValue<'a>) -> IntValue<'a> {
    let lhs = lhs.get_sign_extended_constant().unwrap();
    let rhs = rhs.get_sign_extended_constant().unwrap();
    let lhs_sign = lhs < 0;
    let rhs_sign = rhs < 0;
    let mut lhs = ctx.i64_type().const_int(lhs.abs() as u64, false);
    let rhs = ctx.i64_type().const_int(rhs.abs() as u64, false);
    let mut ans = ctx.i64_type().const_zero();
    while lhs.const_nsw_sub(rhs).get_sign_extended_constant().unwrap() >= 0 {
        lhs = lhs.const_nsw_sub(rhs);
        ans = ans.const_add(ctx.i64_type().const_int(1, false));
    }
    let sign = (lhs_sign || rhs_sign) && !(lhs_sign && rhs_sign);
    if sign {
        ans.const_neg()
    } else {
        ans
    }
}

impl<'gen> ComptimeEvaluate<'gen> for Number {
    type Out = Value<'gen>;

    fn evaluate(&self, _gen: Arc<RwLock<Generator<'gen>>>) -> Result<Self::Out> {
        Ok(Value::new_comp_int(self.num))
    }
}

impl<'gen> ComptimeEvaluate<'gen> for LVal {
    type Out = Value<'gen>;
    
    fn evaluate(&self, gen: Arc<RwLock<Generator<'gen>>>) -> Result<Self::Out> {
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
