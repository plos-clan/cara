use super::*;

#[derive(Debug, Clone)]
pub struct ConstDef {
    pub name: String,
    pub initial_value: ConstInitialValue,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ConstInitialValue {
    Function(FunctionDef),
    Exp(ConstExp),
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub params: Vec<Param>,
    pub return_type: Type,
    pub block: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub param_type: Type,
    pub span: Span,
}
