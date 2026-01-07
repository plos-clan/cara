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
    Assign(Box<Assign>),
    Return(Box<Return>),
    IfExp(Box<IfExp>),
    For(Box<For>),
    Loop(Box<Loop>),
    While(Box<While>),
    ProtoDef(Box<ProtoDef>),
    Unit(Span),
    TypeCast(Box<TypeCast>),
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
            Exp::Assign(assign) => assign.span,
            Exp::Return(return_) => return_.span,
            Exp::IfExp(if_exp) => if_exp.span,
            Exp::Unit(span) => *span,
            Exp::For(for_) => for_.span,
            Exp::Loop(loop_) => loop_.span,
            Exp::While(while_) => while_.span,
            Exp::ProtoDef(proto_def) => proto_def.span,
            Exp::TypeCast(type_cast) => type_cast.span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeCast {
    pub exp: Exp,
    pub ty: Type,
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
