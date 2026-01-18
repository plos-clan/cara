use std::collections::HashMap;

use super::*;

#[derive(Debug, Clone)]
pub struct ConstExp {
    pub exp: ExpId,
}

impl ConstExp {
    pub fn span(&self) -> Span {
        self.exp.span()
    }
}

#[derive(Debug, Clone)]
pub enum Array {
    List(Vec<ExpId>, Span),
    Template(ExpId, ConstExp, Span),
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
    Exp(ExpId, Span),
    Type(Type),
    Number(Number),
    Var(Var),
    Str(String, Span),
    Unary(UnaryOp, ExpId, Span),
    Binary(BinaryOp, ExpId, ExpId, Span),
    GetAddr(GetAddr),
    Deref(Deref),
    Index(Index),
    Array(Array),
    Call(Call),
    Block(Block),
    Function(FunctionDef),
    Assign(Assign),
    Return(Return),
    IfExp(IfExp),
    For(For),
    Loop(Loop),
    While(While),
    ProtoDef(ProtoDef),
    Unit(Span),
    TypeCast(TypeCast),
    Structure(Structure),
    FieldAccess(FieldAccess),
}

impl Exp {
    pub fn span(&self) -> Span {
        match self {
            Self::Exp(_, span) => *span,
            Self::Type(type_) => type_.span,
            Self::Number(number) => number.span,
            Self::Var(var) => var.span,
            Self::Unary(_, _, span) => *span,
            Self::Binary(_, _, _, span) => *span,
            Self::GetAddr(get_addr) => get_addr.span,
            Self::Str(_, span) => *span,
            Self::Deref(deref) => deref.span,
            Self::Index(index) => index.span,
            Self::Array(array) => array.span(),
            Self::Call(call) => call.span,
            Self::Block(block) => block.span,
            Self::Function(func) => func.span,
            Self::Assign(assign) => assign.span,
            Self::Return(return_) => return_.span,
            Self::IfExp(if_exp) => if_exp.span,
            Self::Unit(span) => *span,
            Self::For(for_) => for_.span,
            Self::Loop(loop_) => loop_.span,
            Self::While(while_) => while_.span,
            Self::ProtoDef(proto_def) => proto_def.span,
            Self::TypeCast(type_cast) => type_cast.span,
            Self::Structure(structure) => structure.span,
            Self::FieldAccess(field_access) => field_access.span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeCast {
    pub exp: ExpId,
    pub ty: ExpId,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub func: ExpId,
    pub args: Vec<ExpId>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Var {
    pub path: Path,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Index {
    pub exp: ExpId,
    pub index: ExpId,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Deref {
    pub exp: ExpId,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct GetAddr {
    pub exp: ExpId,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Number {
    pub num: u64,
    pub ty: Option<TypeEnum>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub path: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub lhs: ExpId,
    pub rhs: ExpId,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Return {
    pub value: Option<ExpId>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfExp {
    pub condition: ExpId,
    pub then_branch: Block,
    pub else_branch: Option<Block>,
    pub else_if: Option<Box<IfExp>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Loop {
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct While {
    pub condition: ExpId,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct For {
    pub var: String,
    pub start: ExpId,
    pub end: ExpId,
    pub step: Option<ExpId>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Structure {
    pub ty: ExpId,
    pub fields: HashMap<String, ExpId>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FieldAccess {
    pub lhs: ExpId,
    pub field: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Pos,
    Neg,
    Not,
    Ptr,
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
