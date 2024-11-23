use super::*;

#[derive(Debug)]
pub enum TypeEnum {
    U64,
}

#[derive(Debug)]
pub struct Type {
    pub ty: TypeEnum,
    pub star: usize,
    pub span: Span,
}
