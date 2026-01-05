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

        rule abi() -> Abi
        = "extern" __ abi: $("C" / "c") _ "[" _ name: identifier() _ "]" {
            match abi {
                "C" | "c" => Abi::CAbi(name),
                _ => unreachable!(),
            }
        }

        rule function_def() -> FunctionDef
        = l: position!() _ abi: abi()? _ "fn" _ "(" _ params: (param() ** ",") _ ")" return_type: (__ "->" _ t: type_() {t})? _ block: block() _ r: position!() {
            FunctionDef { abi: abi.unwrap_or(Abi::Cara), params, return_type, block, span: Span::new(l, r) }
        }

        rule const_def() -> ConstDef
        = l: position!() _ "const" __ name: identifier() _ "=" _ value: const_initial_value() _ ";" r: position!() {
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
                var_type: (":" _ t: type_() {t} )? _
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
        = l: position!() _ name: identifier() _ ":" _ ty: type_() _ r: position!() {
            Param { name, param_type: ty, span: Span::new(l, r) }
        }

        rule expr() -> Exp =
            f: function_def() { Exp::Function(Box::new(f)) } /
            e: expr_impl() { e }

        rule expr_impl() -> Exp = precedence!{
            l: position!() _ "return" __ rhs: @ {
                let span = Span::new(l, rhs.span().end());
                Exp::Return(Box::new(Return { value: Some(rhs), span }))
            }
            l: position!() _ "return" __ ";" {
                let span = Span::new(l, l);
                Exp::Return(Box::new(Return { value: None, span }))
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
            --
            l: (@) _ "<<" _ r: @ {
                binary_op_rule!(l, r, LShift)
            }
            l: (@) _ ">>" _ r: @ {
                binary_op_rule!(l, r, RShift)
            }
            --
            d: deref() { Exp::Deref(Box::new(d)) }
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
            l: (@) _ "(" _ args: (expr() ** ("," _)) ","? _ ")" r: position!() {
                let span = Span::new(l.span().start(), r);
                Exp::Call(Box::new(Call {
                    func: l,
                    args,
                    span
                }))
            }
            --
            i: if_exp() { Exp::IfExp(Box::new(i)) }
            lhs: (@) _ "=" _ rhs: @ {
                let span = Span::new(lhs.span().start(), rhs.span().end());
                Exp::Assign(Box::new(Assign { lhs, rhs, span }))
            }
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
            g: get_addr() { Exp::GetAddr(Box::new(g)) }
            b: block() { Exp::Block(Box::new(b)) }
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

        rule deref() -> Deref
              = l: position!() "*" _ e: expr() r: position!() {
            Deref {
                exp: e,
                span: Span::new(l, r)
            }
        }

        rule get_addr() -> GetAddr
              = l: position!() "*" _ e: expr() r: position!() {
            GetAddr{
                exp: e,
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
            = s: position!() i:identifier() e: position!() {
                Path{
                    path: vec![i],
                    span: Span::new(s, e)
                }
            }
            / s: position!() i:identifier() _ "." _ p: path() e: position!() {
                let mut path = vec![i];
                path.extend(p.path);
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
            }

        rule type_() -> Type
            = s: position!() ref_count: ("*")* kind: type_enum() e: position!() {
            Type {
                ref_count: ref_count.len(),
                kind,
                span: Span::new(s, e),
            }
        }

        rule identifier() -> String
          = n:$([ 'a'..='z' | 'A'..='Z' | '_']['a'..='z' | 'A'..='Z' | '0'..='9' | '_' ]*) {
              n.into()
          }

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
