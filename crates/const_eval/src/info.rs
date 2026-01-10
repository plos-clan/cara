use std::sync::Arc;

use ast::{FunctionDef, ProtoDef, Span, Type, TypeEnum};

#[derive(Clone)]
pub enum ValueKind {
    Int(i64),
    Function(Arc<FunctionDef>),
    Proto(Arc<ProtoDef>),
    Unit,
}

#[derive(Clone)]
pub struct Value {
    kind: ValueKind,
    ty: Option<Arc<Type>>,
}

impl Value {
    pub fn new_int(value: i64) -> Self {
        Value {
            kind: ValueKind::Int(value),
            ty: None,
        }
    }

    pub fn new_function(func: Arc<FunctionDef>) -> Self {
        Value {
            kind: ValueKind::Function(func),
            ty: None,
        }
    }

    pub fn new_proto(proto: Arc<ProtoDef>) -> Self {
        Value {
            kind: ValueKind::Proto(proto),
            ty: None,
        }
    }

    pub fn new_unit() -> Self {
        Value {
            kind: ValueKind::Unit,
            ty: None,
        }
    }
}

impl Value {
    pub fn as_int(&self) -> i64 {
        match &self.kind {
            ValueKind::Int(value) => *value,
            _ => unreachable!(),
        }
    }

    pub fn as_function(&self) -> Arc<FunctionDef> {
        match &self.kind {
            ValueKind::Function(func) => func.clone(),
            _ => unreachable!(),
        }
    }

    pub fn as_proto(&self) -> Arc<ProtoDef> {
        match &self.kind {
            ValueKind::Proto(proto) => proto.clone(),
            _ => unreachable!(),
        }
    }

    pub fn as_unit(&self) {
        match &self.kind {
            ValueKind::Unit => (),
            _ => unreachable!(),
        }
    }
}

impl Value {
    pub fn kind(&self) -> ValueKind {
        self.kind.clone()
    }

    pub fn ty(&self) -> Arc<Type> {
        let span = Span::new(0, 0);
        self.ty.clone().unwrap_or({
            let kind = match self.kind {
                ValueKind::Int(_) => TypeEnum::Signed(32),
                ValueKind::Unit => TypeEnum::Unit,
                _ => unreachable!(),
            };
            Arc::new(Type {
                kind,
                ref_count: 0,
                span,
            })
        })
    }

    pub fn set_type(&mut self, ty: Arc<Type>) {
        self.ty = Some(ty);
    }
}
