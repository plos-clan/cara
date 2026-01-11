use ast::*;

pub fn parse(input: &str) -> anyhow::Result<CompUnit> {
    cara_parser::comp_unit(input).map_err(|e| e.into())
}

macro_rules! binary_op_rule {
    ($l: expr, $r: expr, $op_enum: ident) => {{
        let s = Span::new($l.span().start(), $r.span().end());
        Exp::Binary(BinaryOp::$op_enum, Box::new($l), Box::new($r), s)
    }};
}

peg::parser! {
    grammar cara_parser() for str {
        pub rule comp_unit() -> CompUnit
        = l: position!() _ i: (global_item() ** _) _ r: position!() {
            CompUnit { global_items: i, span: Span::new(l, r) }
        }

        rule global_item() -> GlobalItem
        = c: const_def() {
            GlobalItem::ConstDef(c)
        }

        rule abi_kind() -> Abi
        = abi: $("C" / "c") _ "[" _ name: identifier() _ "]" {
            match abi {
                "C" | "c" => Abi::CAbi(name),
                _ => unreachable!(),
            }
        }

        rule proto_def() -> ProtoDef
        = l: position!() "proto" __ abi: abi_kind() _ "fn" _ "(" _ params: (param() ** ("," _)) _ ","? _ ")" return_type: (__ "->" _ t: expr() {t})? _ r: position!() {
            ProtoDef { abi: abi, params, return_type, span: Span::new(l, r) }
        }

        rule function_def() -> FunctionDef
        = l: position!() abi: ("extern" __ a: abi_kind() {a})? _ "fn" _ "(" _ params: (param() ** ("," _)) _ ","? _ ")" return_type: (__ "->" _ t: expr() {t})? _ block: block() _ r: position!() {
            FunctionDef { abi: abi.unwrap_or(Abi::Cara), params, return_type, block, span: Span::new(l, r) }
        }

        rule const_def() -> ConstDef
        = l: position!() "const" __ name: identifier() _ "=" _ value: const_initial_value() _ ";" r: position!() {
            ConstDef { name, initial_value: value, span: Span::new(l, r) }
        }

        rule const_initial_value() -> ConstInitialValue
        = e: expr() {
            ConstInitialValue::Exp(ConstExp { exp: e })
        }

        rule block() -> Block
        = l: position!() _ "{" _ items: (block_item() ** _) _ return_value: expr()? _ "}" _ r: position!() {
            Block { items, return_value, span: Span::new(l, r) }
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
            VarDef { name, var_type, initial_value: value, mutable: mutable.is_some(), span: Span::new(l, r) }
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
            InlineAsm { asm, span: Span::new(l, r) }
        }

        rule param() -> Param
        = l: position!() name: identifier() _ ":" _ ty: expr() r: position!() {
            Param { name, param_type: ty, span: Span::new(l, r) }
        }

        rule expr() -> Exp =
            p: proto_def() { Exp::ProtoDef(Box::new(p)) } /
            f: function_def() { Exp::Function(Box::new(f)) } /
            precedence!{
                l: position!() _ "return" __ rhs: @ {
                    let span = Span::new(l, rhs.span().end());
                    Exp::Return(Box::new(Return { value: Some(rhs), span }))
                }
                l: position!() _ "return" __ ";" {
                    let span = Span::new(l, l);
                    Exp::Return(Box::new(Return { value: None, span }))
                }
                lhs: (@) _ "=" _ rhs: @ {
                    let span = Span::new(lhs.span().start(), rhs.span().end());
                    Exp::Assign(Box::new(Assign { lhs, rhs, span }))
                }
                --
                l: (@) _ "<" _ r: @ {
                    binary_op_rule!(l, r, Lt)
                }
                l: (@) _ ">" _ r: @ {
                    binary_op_rule!(l, r, Gt)
                }
                l: (@) _ "<=" _ r: @ {
                    binary_op_rule!(l, r, Le)
                }
                l: (@) _ ">=" _ r: @ {
                    binary_op_rule!(l, r, Ge)
                }
                l: (@) _ "==" _ r: @ {
                    binary_op_rule!(l, r, Eq)
                }
                l: (@) _ "!=" _ r: @ {
                    binary_op_rule!(l, r, Ne)
                }
                --
                l: (@) _ "&&" _ r: @ {
                    binary_op_rule!(l, r, And)
                }
                l: (@) _ "||" _ r: @ {
                    binary_op_rule!(l, r, Or)
                }
                --
                l: (@) _ "+" _ r: @ {
                    binary_op_rule!(l, r, Add)
                }
                l: (@) _ "-" _ r: @ {
                    binary_op_rule!(l, r, Sub)
                }
                --
                l: (@) _ "*" _ r: @ {
                    binary_op_rule!(l, r, Mul)
                }
                l: (@) _ "/" _ r: @ {
                    binary_op_rule!(l, r, Div)
                }
                l: (@) _ "%" _ r: @ {
                    binary_op_rule!(l, r, Mod)
                }
                --
                s: position!() "+" _ r: (@) {
                    let span = Span::new(s, r.span().end());
                    Exp::Unary(UnaryOp::Pos, Box::new(r), span)
                }
                s: position!() "-" _ r: (@) {
                    let span = Span::new(s, r.span().end());
                    Exp::Unary(UnaryOp::Neg, Box::new(r), span)
                }
                s: position!() "!" _ r: (@) {
                    let span = Span::new(s, r.span().end());
                    Exp::Unary(UnaryOp::Not, Box::new(r), span)
                }
                s: position!() "*" _ r: (@) {
                    let span = Span::new(s, r.span().end());
                    Exp::Unary(UnaryOp::Ptr, Box::new(r), span)
                }
                --
                l: (@) _ "<<" _ r: @ {
                    binary_op_rule!(l, r, LShift)
                }
                l: (@) _ ">>" _ r: @ {
                    binary_op_rule!(l, r, RShift)
                }
                --
                l: (@) __ "as" __ t: @ {
                    let span = Span::new(l.span().start(), t.span().end());
                    Exp::TypeCast(Box::new(TypeCast { exp: l, ty: t, span }))
                }
                --
                l: (@) _ "[" _ r: expr() _ "]" {
                    let span = Span::new(l.span().start(), r.span().end());
                    Exp::Index(Box::new(Index {
                        exp: l,
                        index: r,
                        span
                    }))
                }
                --
                l: position!() "&" _ e: (@) {
                    let span = Span::new(l, e.span().end());
                    Exp::GetAddr(Box::new(GetAddr { exp: e, span }))
                }
                --
                t: @ _ "{" _
                    fields: ((name: identifier() _ ":" _ value: expr() {(name, value)}) ** ("," _)) ","?
                _ "}" r: position!() {
                    let span = Span::new(t.span().start(), r);
                    Exp::Structure(Box::new(Structure {
                        ty: Box::new(t),
                        fields: fields.into_iter().collect(),
                        span
                    }))
                }
                --
                l: @ "." "*" r: position!() {
                    let span = Span::new(l.span().start(), r);
                    Exp::Deref(Box::new(Deref {
                        exp: l,
                        span
                    }))
                }
                l: @ "." n: identifier() r: position!() {
                    let span = Span::new(l.span().start(), r);
                    Exp::FieldAccess(Box::new(FieldAccess {
                        lhs: l,
                        field: n,
                        span
                    }))
                }
                --
                l: (@) _ "(" _ args: (expr() ** ("," _)) ","? _ ")" r: position!() {
                    let span = Span::new(l.span().start(), r);
                    Exp::Call(Box::new(Call {
                        func: l,
                        args,
                        span
                    }))
                }
                --
                t: type_() { Exp::Type(t) }
                f: for_exp() { Exp::For(Box::new(f)) }
                l: loop_exp() { Exp::Loop(Box::new(l)) }
                w: while_exp() { Exp::While(Box::new(w)) }
                i: if_exp() { Exp::IfExp(Box::new(i)) }
                l: position!() "(" _ ")" r: position!() {
                    let span = Span::new(l, r);
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
                    span: Span::new(l, r)
                }
            }

        rule loop_exp() -> Loop
            = l: position!() "loop" _ b: block() r: position!() {
                Loop {
                    body: b,
                    span: Span::new(l, r)
                }
            }

        rule while_exp() -> While
            = l: position!() "while" __ c: expr() _ b: block() r: position!() {
                While {
                    condition: c,
                    body: b,
                    span: Span::new(l, r)
                }
            }

        rule array() -> Array
            = l: position!() "[" _ values: (expr() ** ("," _)) _ "]" r: position!() {
                Array::List(values, Span::new(l, r))
            }

        rule if_exp() -> IfExp
             = l: position!() "if" __ c: expr() _ t: block() _
                e: ("else" _ b: block() {b})? r: position!() {
                IfExp {
                    condition: c,
                    then_branch: t,
                    else_branch: e,
                    else_if: None,
                    span: Span::new(l, r)
                }
            } / l: position!() "if" __ c: expr() _ t: block() _
                i: ("else" __ i: if_exp() {i})? r: position!() {
                IfExp {
                    condition: c,
                    then_branch: t,
                    else_branch: None,
                    else_if: i.map(Box::new),
                    span: Span::new(l, r)
                }
            }

        rule string_wrapper() -> Exp
             = s: position!() string: string() e: position!() {
            Exp::Str(string, Span::new(s, e))
        }

        rule var() -> Var
              = s: position!() p: path() e: position!() {
            Var { path: p, span: Span::new(s, e) }
        }

        rule number() -> Exp
            = l: position!() n:$(['0'..='9']+) w: ("i" w: width() {w})? r: position!() {
                Exp::Number(Number { num: n.parse().unwrap(), ty: w.map(|w| (true, w)), span: Span::new(l, r) })
        }

        rule width() -> u32
             = n: $(['0'..='9']+) {
                n.parse().unwrap()
            }

        rule path() -> Path
            = s: position!() path:(identifier() ++ "::") e: position!() {
                Path{
                    path,
                    span: Span::new(s, e)
                }
            }

        rule type_enum() -> TypeEnum
            = "i" n:$(['0'..='9']+) {
                let width = n.parse().unwrap();
                TypeEnum::Signed(width)
            } / "u" n:$(['0'..='9']+) {
                let width = n.parse().unwrap();
                TypeEnum::Unsigned(width)
            } / "(" _ ")" {
                TypeEnum::Unit
            } / "[" _ inner: expr() _ ";" _ len: width() _ "]" {
                TypeEnum::Array(Box::new(inner), len)
            } / "struct" _ "{"
                _ fields: (name: identifier() _ ":" _ ty: expr() { (name, ty) }) ** ("," _) ","? _
                items: (global_item() ** _) _
             "}" {
                TypeEnum::Structure(fields.into_iter().collect(), items)
            }

        rule type_() -> Type
            = s: position!() kind: type_enum() e: position!() {
            Type {
                kind,
                span: Span::new(s, e),
            }
        }

        rule identifier() -> String
          = n:$(!keyword() ['a'..='z' | 'A'..='Z' | '_']['a'..='z' | 'A'..='Z' | '0'..='9' | '_' ]*) {
              n.into()
          }

        rule keyword()
          = ("const" / "fn" / "extern" / "mut" / "proto" / "let" / "struct"
            / "if" / "while" / "loop" / "for" / "in" / "else"
            / "i" n:$(['0'..='9']+) / "u" n:$(['0'..='9']+)) __

        rule string() -> String
        = "\""  s: string_character()* "\"" { s.into_iter().collect() }

        rule string_character() -> char
            = [^ '"' | '\\' | '\r' | '\n' ]
            / "\\n" { '\n' }
            / "\\u{" value:$(['0'..='9' | 'a'..='f' | 'A'..='F']+) "}" {?
                u32::from_str_radix(value, 16).ok().and_then(char::from_u32).ok_or("valid unicode code point")
            }
            / expected!("valid escape sequence")

        rule whitespace() = quiet!{[' ' | '\n' | '\t']}
        rule _() = whitespace()*
        rule __() = whitespace()+
    }
}
