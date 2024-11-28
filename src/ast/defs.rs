use super::*;

#[derive(Debug)]
pub struct ConstDecl {
    pub name: String,
    pub initial_value: ConstInitialValue,
    pub span: Span,
}

#[derive(Debug)]
pub enum ConstInitialValueEnum {
    Function(FunctionDef),
    Exp(ConstExp),
}

#[derive(Debug)]
pub struct ConstInitialValue {
    pub value: ConstInitialValueEnum,
    pub span: Span,
}

#[derive(Debug)]
pub struct FunctionDef {
    pub params: Vec<Param>,
    pub return_type: Type,
    pub block: Block,
    pub span: Span,
}

#[derive(Debug)]
pub struct Param {
    pub name: String,
    pub param_type: Type,
    pub span: Span,
}
