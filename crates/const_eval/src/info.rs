use std::{collections::HashMap, sync::Arc};

use ast::{FunctionDef, ProtoDef, Span, Type, TypeEnum};

#[derive(Debug, Clone)]
pub enum ValueKind {
    Int(i64),
    Function(Arc<FunctionDef>),
    Proto(Arc<ProtoDef>),
    Structure(Arc<TypeKind>, HashMap<String, Value>),
    Type(Arc<TypeKind>),
    Unit,
}

#[derive(Debug, Clone)]
pub struct Value {
    kind: ValueKind,
    ty: Option<Arc<TypeKind>>,
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

    pub fn new_structure(ty: Arc<TypeKind>, structure: HashMap<String, Value>) -> Self {
        Value {
            kind: ValueKind::Structure(ty, structure),
            ty: None,
        }
    }

    pub fn new_type(ty: Arc<TypeKind>) -> Self {
        Value {
            kind: ValueKind::Type(ty),
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

    pub fn as_type(&self) -> Arc<TypeKind> {
        match &self.kind {
            ValueKind::Type(ty) => ty.clone(),
            _ => unreachable!(),
        }
    }
}

impl Value {
    pub fn kind(&self) -> ValueKind {
        self.kind.clone()
    }

    pub fn ty(&self) -> Arc<TypeKind> {
        let span = Span::default();
        self.ty.clone().unwrap_or({
            let kind = match self.kind {
                ValueKind::Int(_) => TypeEnum::Signed(32),
                ValueKind::Unit => TypeEnum::Unit,
                _ => unreachable!(),
            };
            TypeKind::new(Arc::new(Type { kind, span }))
        })
    }

    pub fn set_type(&mut self, ty: Arc<TypeKind>) {
        self.ty = Some(ty);
    }
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Primary(Arc<Type>),
    Ptr(Arc<Self>),
}

impl TypeKind {
    pub fn new(ty: Arc<Type>) -> Arc<Self> {
        Arc::new(TypeKind::Primary(ty))
    }

    pub fn new_ptr(self: &Arc<Self>) -> Arc<Self> {
        Arc::new(TypeKind::Ptr(self.clone()))
    }
}
