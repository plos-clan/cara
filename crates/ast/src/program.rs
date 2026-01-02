use super::*;

#[derive(Debug, Clone)]
pub struct CompUnit {
    pub global_items: Vec<GlobalItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum GlobalItem {
    ConstDef(ConstDef),
}

#[derive(Debug, Clone)]
pub struct Block {
    pub items: Vec<BlockItem>,
    pub return_value: Option<Exp>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum BlockItem {
    Statement(Statement),
    VarDef(VarDef),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Return(Return),
    Exp(Exp),
}

#[derive(Debug, Clone)]
pub struct Return {
    pub value: Option<Exp>,
    pub span: Span,
}
