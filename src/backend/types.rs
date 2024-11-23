use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType, IntType};

use crate::ast::Span;

use super::*;

#[derive(Debug,Clone)]
enum TypeEnum<'gen> {
    Type,
    Function(FunctionType<'gen>),
    Int(IntType<'gen>),
}

#[derive(Debug, Clone)]
pub struct BackendType<'gen> {
    type_enum: TypeEnum<'gen>,
    symbols: SymbolTable<'gen>,
}

impl<'gen> BackendType<'gen> {
    pub fn new_type() -> Self {
        Self {
            type_enum: TypeEnum::Type,
            symbols: SymbolTable::new(),
        }
    }

    pub fn new_function(function_type: FunctionType<'gen>) -> Self {
        Self {
            type_enum: TypeEnum::Function(function_type),
            symbols: SymbolTable::new(),
        }
    }

    pub fn new_int(int_type: IntType<'gen>) -> Self {
        Self {
            type_enum: TypeEnum::Int(int_type),
            symbols: SymbolTable::new(),
        }
    }
}

impl<'gen> BackendType<'gen> {
    pub fn as_function(&self, span: Span) -> Result<FunctionType<'gen>> {
        match &self.type_enum {
            TypeEnum::Function(func_type) => Ok(func_type.clone()),
            _ => Err(Error(ErrorTypes::UseVoidValue, span)),
        }
    }

    pub fn as_basic_type_enum(&self, span: Span) -> Result<BasicTypeEnum> {
        let e = match &self.type_enum {
            TypeEnum::Int(int_type) => int_type.as_basic_type_enum(),
            _ => return Err(Error(ErrorTypes::UseVoidValue, span)),
        };
        Ok(e)
    }

    pub fn as_basic_metadata_type_enum(&self, span: Span) -> Result<BasicMetadataTypeEnum> {
        Ok(BasicMetadataTypeEnum::from(self.as_basic_type_enum(span)?))
    }

}

impl<'gen> BackendType<'gen> {
    pub fn function_type(&self, paramter_types: Vec<BackendType<'gen>>, span: Span) -> Result<Self> {
        let mut paramter_types_list = Vec::new();

        for param_type in paramter_types.leak().iter() {
            paramter_types_list.push(param_type.as_basic_metadata_type_enum(span.clone())?);
        }

        let value = match &self.type_enum{
            TypeEnum::Int(int_type) => int_type.fn_type(paramter_types_list.leak(), false),
            _ => return Err(Error(ErrorTypes::UseVoidValue, span))
        };

        Ok(
            Self::new_function(value.clone())
        )
    }
}
