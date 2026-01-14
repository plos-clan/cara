use std::collections::HashMap;

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
    Type(Type),
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
    Assign(Box<Assign>),
    Return(Box<Return>),
    IfExp(Box<IfExp>),
    For(Box<For>),
    Loop(Box<Loop>),
    While(Box<While>),
    ProtoDef(Box<ProtoDef>),
    Unit(Span),
    TypeCast(Box<TypeCast>),
    Structure(Box<Structure>),
    FieldAccess(Box<FieldAccess>),
    Module(Module),
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
            Self::Module(module) => module.span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeCast {
    pub exp: Exp,
    pub ty: Exp,
    pub span: Span,
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
    pub ty: Option<(bool, u32)>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub path: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub lhs: Exp,
    pub rhs: Exp,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Return {
    pub value: Option<Exp>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfExp {
    pub condition: Exp,
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
    pub condition: Exp,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct For {
    pub var: String,
    pub start: Exp,
    pub end: Exp,
    pub step: Option<Exp>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Structure {
    pub ty: Box<Exp>,
    pub fields: HashMap<String, Exp>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FieldAccess {
    pub lhs: Exp,
    pub field: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub path: String,
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
