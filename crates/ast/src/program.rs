use super::*;

#[derive(Debug)]
pub struct CompUnit {
    pub global_items: Vec<GlobalItem>,
    pub span: Span,
}

#[derive(Debug)]
pub enum GlobalItem {
    ConstDef(ConstDef),
}

#[derive(Debug)]
pub struct Block {
    pub items: Vec<BlockItem>,
    pub return_value: Option<Exp>,
    pub span: Span,
}

#[derive(Debug)]
pub enum BlockItem {
    Statement(Statement),
}

#[derive(Debug)]
pub enum Statement {
    Return(Return),
}

#[derive(Debug)]
pub struct Return {
    pub value: Option<Exp>,
    pub span: Span,
}
