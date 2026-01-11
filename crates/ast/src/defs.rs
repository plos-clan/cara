use super::*;

#[derive(Debug, Clone)]
pub struct VarDef {
    pub name: String,
    pub var_type: Option<Exp>,
    pub initial_value: Exp,
    pub mutable: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ConstDef {
    pub name: String,
    pub initial_value: ConstInitialValue,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ConstInitialValue {
    Exp(ConstExp),
}

#[derive(Debug, Clone)]
pub enum Abi {
    Cara,
    CAbi(String),
}

#[derive(Debug, Clone)]
pub struct ProtoDef {
    pub abi: Abi,
    pub params: Vec<Param>,
    pub return_type: Option<Exp>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub abi: Abi,
    pub params: Vec<Param>,
    pub return_type: Option<Exp>,
    pub block: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub param_type: Exp,
    pub span: Span,
}
