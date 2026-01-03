use super::*;

#[derive(Debug, Clone)]
pub struct ConstExp {
    pub exp: Exp,
}

impl ConstExp {
    pub fn span(&self) -> Span {
        self.exp.span()
    }
}

#[derive(Debug, Clone)]
pub enum Array {
    List(Vec<Exp>, Span),
    Template(Exp, ConstExp, Span),
}

impl Array {
    pub fn span(&self) -> Span {
        match self {
            Array::List(_, span) => *span,
            Array::Template(_, _, span) => *span,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Exp {
    Exp(Box<Exp>, Span),
    Number(Number),
    Var(Box<Var>),
    Str(String, Span),
    Unary(UnaryOp, Box<Exp>, Span),
    Binary(BinaryOp, Box<Exp>, Box<Exp>, Span),
    GetAddr(Box<GetAddr>),
    Deref(Box<Deref>),
    Index(Box<Index>),
    Array(Box<Array>),
    Call(Box<Call>),
    Block(Box<Block>),
    Function(Box<FunctionDef>),
}

impl Exp {
    pub fn span(&self) -> Span {
        match self {
            Exp::Exp(_, span) => *span,
            Exp::Number(number) => number.span,
            Exp::Var(var) => var.span,
            Exp::Unary(_, _, span) => *span,
            Exp::Binary(_, _, _, span) => *span,
            Exp::GetAddr(get_addr) => get_addr.span,
            Exp::Str(_, span) => *span,
            Exp::Deref(deref) => deref.span,
            Exp::Index(index) => index.span,
            Exp::Array(array) => array.span(),
            Exp::Call(call) => call.span,
            Exp::Block(block) => block.span,
            Exp::Function(func) => func.span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Call {
    pub func: Exp,
    pub args: Vec<Exp>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Var {
    pub path: Path,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Index {
    pub exp: Exp,
    pub index: Exp,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Deref {
    pub exp: Exp,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct GetAddr {
    pub exp: Exp,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Number {
    pub num: u64,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub path: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Pos,
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy)]
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
