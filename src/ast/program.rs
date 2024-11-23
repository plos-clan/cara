use super::*;

#[derive(Debug)]
pub struct CompUnit {
    pub global_items: Vec<GlobalItem>,
    pub span: Span,
}

#[derive(Debug)]
pub enum GlobalItem {
    ConstDecl(ConstDecl),
}

#[derive(Debug)]
pub struct Block {
    pub span: Span,
}

