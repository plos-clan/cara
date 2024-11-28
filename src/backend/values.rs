use inkwell::{context::Context, values::{BasicValueEnum, FunctionValue, IntValue}};

use crate::ast::Span;

use super::*;
use anyhow::Result;

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
    CompInt(u64),
    Int(IntValue<'gen>),
    Void,
}


#[derive(Debug, Clone)]
pub struct Value<'gen> {
    value_enum: ValueEnum<'gen>,
    value_type: BackendType<'gen>,
}

impl<'gen> Value<'gen> {
    pub fn new_void() -> Self {
        Self {
            value_enum: ValueEnum::Void,
            value_type: BackendType::new_void(),
        }
    }
    
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
    
    pub fn new_comp_int(value: u64) -> Self {
        Self {
            value_enum: ValueEnum::CompInt(value),
            value_type: BackendType::new_comp_int(),
        }
    }
    
    pub fn new_int(value: IntValue<'gen>) -> Self {
        Self {
            value_enum: ValueEnum::Int(value),
            value_type: BackendType::new_int(value.get_type()),
        }
    }
}

impl<'gen> Value<'gen> {
    pub fn as_function(&self,span: Span) -> Result<Function> {
        match &self.value_enum {
            ValueEnum::Function(func) => Ok(func.clone()),
            _ => Err(CompileError::new_invalid_type_cast( span,self.value_type.get_name(), "function".to_string() ).into())
        }
    }

    pub fn as_type(&self, span: Span) -> Result<BackendType<'gen>> {
        match &self.value_enum {
            ValueEnum::Type(ty) => Ok(ty.clone()),
            _ => Err(CompileError::new_invalid_type_cast(span, self.value_type.get_name(), "type".to_string() ).into())
        }
    }
    
    pub fn as_comp_int(&self, span: Span) -> Result<u64> {
        match &self.value_enum {
            ValueEnum::CompInt(value) => Ok(*value),
            _ => Err(CompileError::new_invalid_type_cast(span, self.value_type.get_name(), "comptime_int".to_string() ).into())
        }
    }
    
    pub fn as_int(&self, span: Span, ctx: &'gen Context) -> Result<IntValue<'gen>> {
        match &self.value_enum {
            ValueEnum::CompInt(val) => Ok(ctx.i64_type().const_int(*val as u64, false)),
            ValueEnum::Int(val) => Ok(val.clone()),
            _ => Err(CompileError::new_invalid_type_cast(span, self.value_type.get_name(), "int".to_string()).into())
        }
    }
    
    pub fn as_basic_value_enum(&self,span: Span,context: &'gen Context) -> Result<BasicValueEnum<'gen>> {
        Ok(
            match &self.value_enum {
                ValueEnum::CompInt(_) | ValueEnum::Int(_) => BasicValueEnum::IntValue(self.as_int(span, context)?),
                _ => return Err(CompileError::new_non_comptime_value(span, self.value_type.get_name()).into())
            }
        )
    }
}

