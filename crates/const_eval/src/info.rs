use std::sync::Arc;

use ast::{FunctionDef, ProtoDef};

pub enum Value {
    Int((bool, u32), i64),
    Function(Arc<FunctionDef>),
    Proto(Arc<ProtoDef>),
    Unit,
}

impl Value {
    pub fn apply(&self, mut f: impl FnMut(i64) -> i64) -> Value {
        match self {
            Value::Int(ty, value) => Value::Int(*ty, f(*value)),
            _ => unreachable!(),
        }
    }
}
