use std::sync::{Arc, LazyLock};

use ast::{ConstDef, ConstExp, ConstInitialValue, visitor::ExpVisitor};
use query::{DefId, Provider, QueryContext};

use crate::{ConstEvalContext, info::Value};

pub static CONST_EVAL_PROVIDER: LazyLock<Provider<DefId, Value>> =
    LazyLock::new(|| Provider::new(const_eval_provider));

fn const_eval_provider(ctx: Arc<QueryContext<'_>>, def_id: DefId) -> Value {
    let mut eval_ctx = ConstEvalContext { ctx: ctx.clone() };

    let ConstDef {
        initial_value: ConstInitialValue::Exp(ConstExp { exp }),
        ..
    } = ctx.get_def(def_id).unwrap();

    eval_ctx.visit_exp(exp)
}
