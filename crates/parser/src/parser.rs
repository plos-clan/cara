use std::sync::Arc;

use ast::*;
use peg::RuleResult;

use crate::lexer::{Token, TokenStream};

#[derive(Default)]
pub struct CaraParser;

impl CaraParser {
    pub fn new() -> Self {
        Self
    }
}

impl Parser for CaraParser {
    type Error = anyhow::Error;

    fn parse_content(
        &self,
        ctx: &ParseContext<'_>,
        content: Arc<String>,
    ) -> anyhow::Result<StructType> {
        cara_parser::module_root(&TokenStream::new(content)?, self, ctx).map_err(|e| e.into())
    }
}

macro_rules! binary_op_rule {
    ($parser: expr, $l: expr, $r: expr, $op_enum: ident) => {{
        let s = $parser.span($l.span().start(), $r.span().end());
        $parser.insert_exp(Exp::Binary(BinaryOp::$op_enum, $l, $r, s))
    }};
}

peg::parser! {
    grammar cara_parser<'s>(pself: &CaraParser, parser: &ParseContext<'_>) for TokenStream {
        pub rule module_root() -> StructType
        = _ i: struct_inner() _ {
            i
        }

        rule global_item() -> GlobalItem
        = c: const_def() {
            GlobalItem::ConstDef(Arc::new(c))
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

        rule expr() -> ExpId =
            p: proto_def() { parser.insert_exp(Exp::ProtoDef(p)) } /
            f: function_def() { parser.insert_exp(Exp::Function(f)) } /
            precedence!{
                l: position!() _ "return" __ rhs: @ {
                    let span = parser.span(l, rhs.span().end());
                    parser.insert_exp(Exp::Return(Return { value: Some(rhs), span }))
                }
                l: position!() _ "return" __ ";" {
                    let span = parser.span(l, l);
                    parser.insert_exp(Exp::Return(Return { value: None, span }))
                }
                lhs: (@) _ "=" _ rhs: @ {
                    let span = parser.span(lhs.span().start(), rhs.span().end());
                    parser.insert_exp(Exp::Assign(Assign { lhs, rhs, span }))
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
                    parser.insert_exp(Exp::Unary(UnaryOp::Pos, r, span))
                }
                s: position!() "-" _ r: (@) {
                    let span = parser.span(s, r.span().end());
                    parser.insert_exp(Exp::Unary(UnaryOp::Neg, r, span))
                }
                s: position!() "!" _ r: (@) {
                    let span = parser.span(s, r.span().end());
                    parser.insert_exp(Exp::Unary(UnaryOp::Not, r, span))
                }
                s: position!() "*" _ r: (@) {
                    let span = parser.span(s, r.span().end());
                    parser.insert_exp(Exp::Unary(UnaryOp::Ptr, r, span))
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
                    parser.insert_exp(Exp::TypeCast(TypeCast { exp: l, ty: t, span }))
                }
                --
                l: (@) _ "[" _ r: expr() _ "]" {
                    let span = parser.span(l.span().start(), r.span().end());
                    parser.insert_exp(Exp::Index(Index { exp: l, index: r, span }))
                }
                --
                l: position!() "&" _ e: (@) {
                    let span = parser.span(l, e.span().end());
                    parser.insert_exp(Exp::GetAddr(GetAddr { exp: e, span }))
                }
                --
                t: @ _ "{" _
                    fields: ((name: identifier() _ ":" _ value: expr() {(name, value)}) ** ("," _)) ","?
                _ "}" r: position!() {
                    let span = parser.span(t.span().start(), r);
                    parser.insert_exp(Exp::Structure(Structure {
                        ty: t,
                        fields: fields.into_iter().collect(),
                        span
                    }))
                }
                --
                l: @ "." "*" r: position!() {
                    let span = parser.span(l.span().start(), r);
                    parser.insert_exp(Exp::Deref(Deref {
                        exp: l,
                        span
                    }))
                }
                l: @ "." n: identifier() r: position!() {
                    let span = parser.span(l.span().start(), r);
                    parser.insert_exp(Exp::FieldAccess(FieldAccess {
                        lhs: l,
                        field: n,
                        span
                    }))
                }
                --
                l: (@) _ "(" _ args: (expr() ** ("," _)) ","? _ ")" r: position!() {
                    let span = parser.span(l.span().start(), r);
                    parser.insert_exp(Exp::Call(Call {
                        func: l,
                        args,
                        span
                    }))
                }
                --
                m: module() {
                    let (span, m) = m;
                    parser.insert_exp(Exp::Type(Type { kind: TypeEnum::Structure(m), span }))
                }
                t: type_() { parser.insert_exp(Exp::Type(t)) }
                f: for_exp() { parser.insert_exp(Exp::For(f)) }
                l: loop_exp() { parser.insert_exp(Exp::Loop(l)) }
                w: while_exp() { parser.insert_exp(Exp::While(w)) }
                i: if_exp() { parser.insert_exp(Exp::IfExp(i)) }
                l: position!() "(" _ ")" r: position!() {
                    let span = parser.span(l, r);
                    parser.insert_exp(Exp::Unit(span))
                }
                "(" _ e: expr() _ ")" {
                    e
                }
                n: number() { parser.insert_exp(n) }
                s: string_wrapper() { parser.insert_exp(s) }
                v: var() { parser.insert_exp(Exp::Var(v)) }
                b: block() { parser.insert_exp(Exp::Block(b)) }
                a: array() { parser.insert_exp(Exp::Array(a)) }
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

        rule module() -> (Span, StructType)
             = s: position!() "mod" __ path: string() e: position!() {
                 let Ok(file) = parser.file_table().register_file(path.clone()) else {
                     panic!("Failed to read {}!", path);
                 };
                 (
                     parser.span(s, e),
                     parser.parse_module(pself, file).unwrap()
                 )
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
                        let ty = if width == "size" {
                            if p == "u" {
                                TypeEnum::Usize
                            } else {
                                TypeEnum::Isize
                            }
                        } else {
                            let Ok(width) = width.parse() else {
                                return Err(());
                            };
                            if p == "u" {
                                TypeEnum::Unsigned(width)
                            } else {
                                TypeEnum::Signed(width)
                            }
                        };
                        Ok((num, ty))
                    };

                    let (num, ty) = if num_str.contains("u") {
                        let Ok((num, ty)) = num_width_getter("u") else {
                            return RuleResult::Failed;
                        };
                        (num, Some(ty))
                    } else if num_str.contains("i") {
                        let Ok((num, ty)) = num_width_getter("i") else {
                            return RuleResult::Failed;
                        };
                        (num, Some(ty))
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

                        let signed = if prefix == "i" {
                            true
                        } else if prefix == "u" {
                            false
                        } else {
                            return RuleResult::Failed;
                        };

                        if width == "size" {
                            let result = if signed {
                                TypeEnum::Isize
                            } else {
                                TypeEnum::Usize
                            };
                            return RuleResult::Matched(pos+1, result);
                        }

                        let Ok(width) = width.parse() else {
                            return RuleResult::Failed;
                        };
                        let kind = if signed {
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
                TypeEnum::Array(inner, len as u32)
            } / "struct" _ "{" _ i: struct_inner() _ "}" {
                TypeEnum::Structure(i)
            }

        rule struct_inner() -> StructType
             = l: position!() _
             fields: (name: identifier() _ ":" _ ty: expr() { (name, ty) }) ** ("," _) ","? _
             items: (global_item() ** _) _
             r: position!() {
                 StructType {
                     fields: fields.into_iter().collect(),
                     members: items,
                     span: parser.span(l, r),
                 }
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
