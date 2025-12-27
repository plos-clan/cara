use crate::Span;

#[derive(Debug)]
pub enum TypeEnum {
    I8,
    I16,
    I32,
    I64,

    U8,
    U16,
    U32,
    U64,
}

#[derive(Debug)]
pub struct Type {
    pub kind: TypeEnum,
    pub ref_count: usize,
    pub span: Span,
}
