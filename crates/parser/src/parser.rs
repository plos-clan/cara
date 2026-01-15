use std::sync::Arc;

use ast::*;
use peg::RuleResult;

use crate::lexer::{Token, TokenStream};

pub struct CaraParser<'p> {
    file_table: &'p FileTable,
    current: usize,
}

impl<'p> CaraParser<'p> {
    pub fn new(file_table: &'p FileTable) -> Self {
        Self {
            file_table,
            current: 0,
        }
    }
}

impl<'p> Parser for CaraParser<'p> {
    type Error = anyhow::Error;

    fn file_table(&self) -> &FileTable {
        self.file_table
    }

    fn current_file(&self) -> usize {
        self.current
    }

    fn set_current_file(&mut self, file: usize) {
        self.current = file;
    }

    fn parse_content(&self, content: Arc<String>) -> anyhow::Result<Type> {
        cara_parser::module_root(&TokenStream::new(content)?, self).map_err(|e| e.into())
    }
}

macro_rules! binary_op_rule {
    ($parser: expr, $l: expr, $r: expr, $op_enum: ident) => {{
        let s = $parser.span($l.span().start(), $r.span().end());
        Exp::Binary(BinaryOp::$op_enum, Box::new($l), Box::new($r), s)
    }};
}

peg::parser! {
    grammar cara_parser<'s>(parser: &CaraParser) for TokenStream {
        pub rule module_root() -> Type
        = l: position!() _ i: struct_inner() _ r: position!() {
            Type { kind: i, span: parser.span(l, r) }
        }

        rule global_item() -> GlobalItem
        = c: const_def() {
            GlobalItem::ConstDef(c)
        }

        rule abi_kind() -> Abi
        = abi: ("C" { "C" } / "c" { "c" }) _ "[" _ name: identifier() _ "]" {
            match abi {
                "C" | "c" => Abi::CAbi(name),
                _ => unreachable!(),
            }
        }

        rule proto_def() -> ProtoDef
        = l: position!() "proto" __ abi: abi_kind() _ "fn" _ "(" _ params: (param() ** ("," _)) _ ","? _ ")" return_type: (__ "-" ">" _ t: expr() {t})? _ r: position!() {
            ProtoDef { abi, params, return_type, span: parser.span(l, r) }
        }

        rule function_def() -> FunctionDef
        = l: position!() abi: ("extern" __ a: abi_kind() {a})? _ "fn" _ "(" _ params: (param() ** ("," _)) _ ","? _ ")" return_type: (__ "-" ">" _ t: expr() {t})? _ block: block() _ r: position!() {
            FunctionDef { abi: abi.unwrap_or(Abi::Cara), params, return_type, block, span: parser.span(l, r) }
        }

        rule const_def() -> ConstDef
        = l: position!() "const" __ name: identifier() _ "=" _ value: const_initial_value() _ ";" r: position!() {
            ConstDef { name, initial_value: value, span: parser.span(l, r) }
        }

        rule const_initial_value() -> ConstInitialValue
        = e: expr() {
            ConstInitialValue::Exp(ConstExp { exp: e })
        }

        rule block() -> Block
        = l: position!() _ "{" _ items: (block_item() ** _) _ return_value: expr()? _ "}" _ r: position!() {
            Block { items, return_value, span: parser.span(l, r) }
        }

        rule block_item() -> BlockItem
        = v: var_def() {
            BlockItem::VarDef(v)
        } / s: statement() {
            BlockItem::Statement(s)
        }

        rule var_def() -> VarDef
        = l: position!() _ "let" __ mutable: ("mut"?) _ name: identifier() _
                var_type: (":" _ t: expr() {t} )? _
                "=" _ value: expr() _ ";" r: position!() {
            VarDef { name, var_type, initial_value: value, mutable: mutable.is_some(), span: parser.span(l, r) }
        }

        rule statement() -> Statement
        = s: statement_impl() ";" {
            s
        }

        rule statement_impl() -> Statement
        = i: inline_asm() {
            Statement::InlineAsm(i)
        } / e: expr() {
            Statement::Exp(e)
        }

        rule inline_asm() -> InlineAsm
        = l: position!() _ "asm" _ "{" _ asm: (string() ** ("," _)) _ ","? _ "}" _ r: position!() {
            InlineAsm { asm, span: parser.span(l, r) }
        }

        rule param() -> Param
        = l: position!() name: identifier() _ ":" _ ty: expr() r: position!() {
            Param { name, param_type: ty, span: parser.span(l, r) }
        }

        rule expr() -> Exp =
            p: proto_def() { Exp::ProtoDef(Box::new(p)) } /
            f: function_def() { Exp::Function(Box::new(f)) } /
            precedence!{
                l: position!() _ "return" __ rhs: @ {
                    let span = parser.span(l, rhs.span().end());
                    Exp::Return(Box::new(Return { value: Some(rhs), span }))
                }
                l: position!() _ "return" __ ";" {
                    let span = parser.span(l, l);
                    Exp::Return(Box::new(Return { value: None, span }))
                }
                lhs: (@) _ "=" _ rhs: @ {
                    let span = parser.span(lhs.span().start(), rhs.span().end());
                    Exp::Assign(Box::new(Assign { lhs, rhs, span }))
                }
                --
                l: (@) _ "<" _ r: @ {
                    binary_op_rule!(parser, l, r, Lt)
                }
                l: (@) _ ">" _ r: @ {
                    binary_op_rule!(parser, l, r, Gt)
                }
                l: (@) _ "<" "=" _ r: @ {
                    binary_op_rule!(parser, l, r, Le)
                }
                l: (@) _ ">" "=" _ r: @ {
                    binary_op_rule!(parser, l, r, Ge)
                }
                l: (@) _ "=" "=" _ r: @ {
                    binary_op_rule!(parser, l, r, Eq)
                }
                l: (@) _ "!" "=" _ r: @ {
                    binary_op_rule!(parser, l, r, Ne)
                }
                --
                l: (@) _ "&" "&" _ r: @ {
                    binary_op_rule!(parser, l, r, And)
                }
                l: (@) _ "|" "|" _ r: @ {
                    binary_op_rule!(parser, l, r, Or)
                }
                --
                l: (@) _ "+" _ r: @ {
                    binary_op_rule!(parser, l, r, Add)
                }
                l: (@) _ "-" _ r: @ {
                    binary_op_rule!(parser, l, r, Sub)
                }
                --
                l: (@) _ "*" _ r: @ {
                    binary_op_rule!(parser, l, r, Mul)
                }
                l: (@) _ "/" _ r: @ {
                    binary_op_rule!(parser, l, r, Div)
                }
                l: (@) _ "%" _ r: @ {
                    binary_op_rule!(parser, l, r, Mod)
                }
                --
                s: position!() "+" _ r: (@) {
                    let span = parser.span(s, r.span().end());
                    Exp::Unary(UnaryOp::Pos, Box::new(r), span)
                }
                s: position!() "-" _ r: (@) {
                    let span = parser.span(s, r.span().end());
                    Exp::Unary(UnaryOp::Neg, Box::new(r), span)
                }
                s: position!() "!" _ r: (@) {
                    let span = parser.span(s, r.span().end());
                    Exp::Unary(UnaryOp::Not, Box::new(r), span)
                }
                s: position!() "*" _ r: (@) {
                    let span = parser.span(s, r.span().end());
                    Exp::Unary(UnaryOp::Ptr, Box::new(r), span)
                }
                --
                l: (@) _ "<" "<" _ r: @ {
                    binary_op_rule!(parser, l, r, LShift)
                }
                l: (@) _ ">" ">" _ r: @ {
                    binary_op_rule!(parser, l, r, RShift)
                }
                --
                l: (@) __ "as" __ t: @ {
                    let span = parser.span(l.span().start(), t.span().end());
                    Exp::TypeCast(Box::new(TypeCast { exp: l, ty: t, span }))
                }
                --
                l: (@) _ "[" _ r: expr() _ "]" {
                    let span = parser.span(l.span().start(), r.span().end());
                    Exp::Index(Box::new(Index {
                        exp: l,
                        index: r,
                        span
                    }))
                }
                --
                l: position!() "&" _ e: (@) {
                    let span = parser.span(l, e.span().end());
                    Exp::GetAddr(Box::new(GetAddr { exp: e, span }))
                }
                --
                t: @ _ "{" _
                    fields: ((name: identifier() _ ":" _ value: expr() {(name, value)}) ** ("," _)) ","?
                _ "}" r: position!() {
                    let span = parser.span(t.span().start(), r);
                    Exp::Structure(Box::new(Structure {
                        ty: Box::new(t),
                        fields: fields.into_iter().collect(),
                        span
                    }))
                }
                --
                l: @ "." "*" r: position!() {
                    let span = parser.span(l.span().start(), r);
                    Exp::Deref(Box::new(Deref {
                        exp: l,
                        span
                    }))
                }
                l: @ "." n: identifier() r: position!() {
                    let span = parser.span(l.span().start(), r);
                    Exp::FieldAccess(Box::new(FieldAccess {
                        lhs: l,
                        field: n,
                        span
                    }))
                }
                --
                l: (@) _ "(" _ args: (expr() ** ("," _)) ","? _ ")" r: position!() {
                    let span = parser.span(l.span().start(), r);
                    Exp::Call(Box::new(Call {
                        func: l,
                        args,
                        span
                    }))
                }
                --
                m: module() { Exp::Module(m) }
                t: type_() { Exp::Type(t) }
                f: for_exp() { Exp::For(Box::new(f)) }
                l: loop_exp() { Exp::Loop(Box::new(l)) }
                w: while_exp() { Exp::While(Box::new(w)) }
                i: if_exp() { Exp::IfExp(Box::new(i)) }
                l: position!() "(" _ ")" r: position!() {
                    let span = parser.span(l, r);
                    Exp::Unit(span)
                }
                "(" _ e: expr() _ ")" {
                    e
                }
                n: number() { n }
                s: string_wrapper() { s }
                v: var() { Exp::Var(Box::new(v)) }
                b: block() { Exp::Block(Box::new(b)) }
                a: array() { Exp::Array(Box::new(a)) }
            }

        rule for_exp() -> For
            = l: position!() "for" __ v: identifier() _ "in" _
                "(" _ s: expr() _ "," _ e: expr() _ step: ("," _ step: expr() {step})? _ ","? _ ")" _
                b: block() r: position!() {
                For {
                    var: v,
                    start: s,
                    end: e,
                    step,
                    body: b,
                    span: parser.span(l, r)
                }
            }

        rule loop_exp() -> Loop
            = l: position!() "loop" _ b: block() r: position!() {
                Loop {
                    body: b,
                    span: parser.span(l, r)
                }
            }

        rule while_exp() -> While
            = l: position!() "while" __ c: expr() _ b: block() r: position!() {
                While {
                    condition: c,
                    body: b,
                    span: parser.span(l, r)
                }
            }

        rule array() -> Array
            = l: position!() "[" _ values: (expr() ** ("," _)) _ "]" r: position!() {
                Array::List(values, parser.span(l, r))
            }

        rule if_exp() -> IfExp
             = l: position!() "if" __ c: expr() _ t: block() _
                e: ("else" _ b: block() {b})? r: position!() {
                IfExp {
                    condition: c,
                    then_branch: t,
                    else_branch: e,
                    else_if: None,
                    span: parser.span(l, r)
                }
            } / l: position!() "if" __ c: expr() _ t: block() _
                i: ("else" __ i: if_exp() {i})? r: position!() {
                IfExp {
                    condition: c,
                    then_branch: t,
                    else_branch: None,
                    else_if: i.map(Box::new),
                    span: parser.span(l, r)
                }
            }

        rule module() -> Module
             = s: position!() "mod" __ path: string() e: position!() {
                 Module { path, span: parser.span(s, e) }
        }

        rule string_wrapper() -> Exp
             = s: position!() string: string() e: position!() {
            Exp::Str(string, parser.span(s, e))
        }

        rule var() -> Var
              = s: position!() p: path() e: position!() {
            Var { path: p, span: parser.span(s, e) }
        }

        rule number() -> Exp
            = #{ |input, pos| match input.get(pos) {
                Some((Token::Number(num_str), span)) => {
                    let num_width_getter = |p| {
                        let (num, width) = num_str.rsplit_once(p).unwrap();
                        let Ok(num) = num.parse() else {
                            return Err(());
                        };
                        let Ok(width) = width.parse() else {
                            return Err(());
                        };
                        Ok((num, width))
                    };

                    let (num, ty) = if num_str.contains("i") {
                        let Ok((num, width)) = num_width_getter("i") else {
                            return RuleResult::Failed;
                        };
                        (num, Some((true, width)))
                    } else if num_str.contains("u") {
                        let Ok((num, width)) = num_width_getter("u") else {
                            return RuleResult::Failed;
                        };
                        (num, Some((false, width)))
                    } else {
                        let Ok(num) = num_str.parse() else {
                            return RuleResult::Failed;
                        };
                        (num, None)
                    };

                    RuleResult::Matched(pos+1, Exp::Number(Number { num, ty, span: parser.span(span.start, span.end) }))
                }
                _ => RuleResult::Failed
            }}

        rule digit() -> u64
            = #{ |input, pos| match input.get(pos) {
                Some((Token::Number(num_str), span)) => {
                    let Ok(num) = num_str.parse() else {
                        return RuleResult::Failed;
                    };
                    RuleResult::Matched(pos+1, num)
                }
                _ => RuleResult::Failed
            }}

        rule path() -> Path
            = s: position!() path:(identifier() ++ (":" ":")) e: position!() {
                Path{
                    path,
                    span: parser.span(s, e)
                }
            }

        rule type_enum() -> TypeEnum
            = #{ |input, pos| match input.get(pos) {
                Some((Token::Ident(ident), span)) => {
                    if ident.len() > 1 {
                        let (prefix, width) = ident.split_at(1);
                        if prefix != "i" && prefix != "u" {
                            return RuleResult::Failed;
                        }
                        let Ok(width) = width.parse() else {
                            return RuleResult::Failed;
                        };
                        let kind = if prefix == "i" {
                            TypeEnum::Signed(width)
                        } else {
                            TypeEnum::Unsigned(width)
                        };
                        RuleResult::Matched(pos+1, kind)
                    } else {
                        RuleResult::Failed
                    }
                }
                _ => RuleResult::Failed
            }} / "(" _ ")" {
                TypeEnum::Unit
            } / "[" _ inner: expr() _ ";" _ len: digit() _ "]" {
                TypeEnum::Array(Box::new(inner), len as u32)
            } / "struct" _ "{" _ i: struct_inner() _ "}" {
                i
            }

        rule struct_inner() -> TypeEnum
             = fields: (name: identifier() _ ":" _ ty: expr() { (name, ty) }) ** ("," _) ","? _
             items: (global_item() ** _) {
                 TypeEnum::Structure(fields.into_iter().collect(), items)
             }

        rule type_() -> Type
            = s: position!() kind: type_enum() e: position!() {
            Type {
                kind,
                span: parser.span(s, e),
            }
        }

        rule identifier() -> String
          = #{ |input, pos| match input.get(pos) {
              Some((Token::Ident(ident), span)) => {
                  RuleResult::Matched(pos+1, ident.into())
              }
              _ => RuleResult::Failed
          }}

        rule keyword()
          = ("const" / "fn" / "extern" / "mut" / "proto" / "let" / "struct" / "mod"
            / "if" / "while" / "loop" / "for" / "in" / "else"
            / "i" n: digit() / "u" n: digit()) __

        rule string() -> String
        = #{ |input, pos| match input.get(pos) {
            Some((Token::String(string), span)) => {
                RuleResult::Matched(pos+1, string.into())
            }
            _ => RuleResult::Failed
        }}

        rule _() =
        rule __() =
    }
}
