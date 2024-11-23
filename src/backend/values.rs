use std::clone;

use inkwell::values::FunctionValue;

use crate::ast::Span;

use super::*;

#[derive(Debug, Clone)]
pub struct Function<'gen> {
    value: FunctionValue<'gen>,
}

impl<'gen> Function<'gen> {
    pub fn new(value: FunctionValue<'gen>) -> Self {
        Self {
            value,
        }
    }
}

impl<'gen> std::ops::Deref for Function<'gen> {
    type Target = FunctionValue<'gen>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Debug, Clone)]
enum ValueEnum<'gen> {
    Type(BackendType<'gen>),
    Function(Function<'gen>),
}


#[derive(Debug, Clone)]
pub struct Value<'gen> {
    value_enum: ValueEnum<'gen>,
    value_type: BackendType<'gen>,
}

impl<'gen> Value<'gen> {
    pub fn new_type(ty: BackendType<'gen>) -> Self {
        Self {
            value_enum: ValueEnum::Type(ty),
            value_type: BackendType::new_type(),
        }
    }

    pub fn new_function(func: Function<'gen>) -> Self {
        Self {
            value_enum: ValueEnum::Function(func.clone()),
            value_type: BackendType::new_function(func.get_type()),
        }
    }
}

impl<'gen> Value<'gen> {
    pub fn as_function(&self,span: Span) -> Result<Function> {
        match &self.value_enum {
            ValueEnum::Function(func) => Ok(func.clone()),
            _ => Err(Error(ErrorTypes::UseVoidValue,span))
        }
    }

    pub fn as_type(&self, span: Span) -> Result<BackendType<'gen>> {
        match &self.value_enum {
            ValueEnum::Type(ty) => Ok(ty.clone()),
            _ => Err(Error(ErrorTypes::UseVoidValue, span))
        }
    }
}

