use super::*;

#[derive(Debug)]
pub struct ConstExp {
    pub exp: Exp,
}

impl ConstExp {
    pub fn span(&self) -> Span {
        self.exp.span()
    }
}

#[derive(Debug)]
pub enum Array {
    List(Vec<Exp>, Span),
    Template(Exp, ConstExp, Span),
}

impl Array {
    pub fn get_span(&self) -> Span {
        match self {
            Array::List(_, span) => span.clone(),
            Array::Template(_, _, span) => span.clone(),
        }
    }
}

#[derive(Debug)]
pub enum Exp {
    Exp(Box<Exp>, Span),
    Number(Number),
    LVal(Box<LVal>),
    Str(String, Span),
    Unary(UnaryOp, Box<Exp>, Span),
    Binary(BinaryOp, Box<Exp>, Box<Exp>, Span),
    GetAddr(Box<GetAddr>),
    Deref(Box<Deref>),
    Index(Box<Index>),
    Array(Box<Array>),
}

impl Exp {
    pub fn span(&self) -> Span {
        match self {
            Exp::Exp(_, span) => span.clone(),
            Exp::Number(number) => number.span.clone(),
            Exp::LVal(lval) => lval.span.clone(),
            Exp::Unary(_, _, span) => span.clone(),
            Exp::Binary(_, _, _, span) => span.clone(),
            Exp::GetAddr(get_addr) => get_addr.span.clone(),
            Exp::Str(_, span) => span.clone(),
            Exp::Deref(deref) => deref.span.clone(),
            Exp::Index(index) => index.span.clone(),
            Exp::Array(array) => array.get_span(),
        }
    }
}

#[derive(Debug)]
pub struct LVal {
    pub path: Path,
    pub span: Span,
}

#[derive(Debug)]
pub struct Index {
    pub exp: Exp,
    pub index: Exp,
    pub span: Span,
}

#[derive(Debug)]
pub struct Deref {
    pub exp: Exp,
    pub span: Span,
}

#[derive(Debug)]
pub struct GetAddr {
    pub exp: Exp,
    pub span: Span,
}

#[derive(Debug)]
pub struct Number {
    pub num: u64,
    pub span: Span,
}

#[derive(Debug)]
pub struct Path {
    pub path: Vec<String>,
    pub span: Span,
}

#[derive(Debug)]
pub enum UnaryOp {
    Pos,
    Neg,
    Not,
}

#[derive(Debug)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,

    And,
    Or,

    LShift,
    RShift,
}
