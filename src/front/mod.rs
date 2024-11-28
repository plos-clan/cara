use crate::ast::*;
use pest::{iterators::Pair,pratt_parser::*};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "cara.pest"]
struct CaraParser;

pub struct CParser {
    code: String,
    file: String,
}

impl CParser {
    pub fn new(code: String, file: String) -> Self {
        Self { code, file }
    }

    fn get_span(&self, span: pest::Span<'_>) -> Span {
        let code = span.lines().next().unwrap();
        Span::new(
            span.start_pos().line_col(),
            span.end_pos().line_col(),
            code.into(),
            self.file.clone(),
        )
    }

    pub fn parse(&self) -> CompUnit {
        use pest::Parser;
        let rules = CaraParser::parse(Rule::comp_unit, &self.code);

        if let Err(err) = rules {
            panic!("{}", err);
        }

        let rules = rules.unwrap().next().unwrap();

        let mut items = Vec::new();
        let span = rules.as_span().clone();

        for line in rules.into_inner() {
            match line.as_rule() {
                Rule::const_decl => items.push(GlobalItem::ConstDecl(self.parse_const_decl(line))),
                Rule::soi | Rule::eoi => {}
                _ => unimplemented!(),
            }
        }

        CompUnit {
            global_items: items,
            span: self.get_span(span),
        }
    }

    pub fn parse_const_decl(&self, rules: Pair<Rule>) -> ConstDecl {
        let mut rules_iter = rules.clone().into_inner();

        let id_rule = rules_iter.next().unwrap();
        let id = self.parse_ident(id_rule);

        let initial_value = self.parse_const_initial_value(rules_iter.next().unwrap());

        ConstDecl {
            name: id,
            initial_value,
            span: self.get_span(rules.as_span().clone()),
        }
    }

    pub fn parse_ident(&self, rules: Pair<Rule>) -> String {
        let mut ident = rules.as_str().to_string();
        if ident.starts_with("@") {
            ident.remove(0);
            ident.remove(0);
            ident.pop();
        }
        println!("{}", ident);
        ident
    }

    pub fn parse_lval(&self, rules: Pair<Rule>) -> LVal {
        let mut rules_iter = rules.clone().into_inner();
        let id = rules_iter.next().unwrap();
        let span = self.get_span(rules.as_span().clone());
        LVal {
            ids: vec![self.parse_ident(id)],
            span,
            exp: None,
        }
    }
    
    pub fn parse_const_initial_value(&self, rules: Pair<Rule>) -> ConstInitialValue {
        let mut rules_iter = rules.clone().into_inner();

        let initial_value = rules_iter.next().unwrap();

        let value = match initial_value.as_rule() {
            Rule::function_def => {
                ConstInitialValueEnum::Function(self.parse_function_def(initial_value))
            }
            Rule::const_exp => ConstInitialValueEnum::Exp(self.parse_const_exp(initial_value)),
            _ => unimplemented!(),
        };

        ConstInitialValue {
            value,
            span: self.get_span(rules.as_span().clone()),
        }
    }

    pub fn parse_function_def(&self, rules: Pair<Rule>) -> FunctionDef {
        let mut rules_iter = rules.clone().into_inner();

        let mut params = Vec::new();

        let return_type = loop {
            let first = rules_iter.next().unwrap();

            if first.as_rule() == Rule::param {
                params.push(self.parse_param(first));
            } else {
                break self.parse_type(first);
            }
        };

        let block = self.parse_block(rules_iter.next().unwrap());

        FunctionDef {
            params,
            return_type,
            block,
            span: self.get_span(rules.as_span().clone()),
        }
    }

    pub fn parse_param(&self, rules: Pair<Rule>) -> Param {
        let mut rules_iter = rules.clone().into_inner();
        let name = rules_iter.next().unwrap().as_str().to_string();
        let param_type = self.parse_type(rules_iter.next().unwrap());
        let span = self.get_span(rules.as_span().clone());
        Param {
            name,
            param_type,
            span,
        }
    }

    pub fn parse_block(&self, rules: Pair<Rule>) -> Block {
        let mut rules_iter = rules.clone().into_inner();
        let mut items = Vec::new();
        while let Some(rule) = rules_iter.next() {
            match rule.as_rule() {
                Rule::stmt => items.push(BlockItem::Statement(self.parse_statement(rule))),
                _ => unimplemented!(),
            }
        }

        Block {
            items,
            span: self.get_span(rules.as_span().clone()),
        }
    }

    pub fn parse_statement(&self, rules: Pair<Rule>) -> Statement {
        let mut rules_iter = rules.clone().into_inner();
        let statement = match rules_iter.next().unwrap().as_rule() {
            Rule::r#return => Statement::Return(self.parse_return(rules.clone())),
            _ => unimplemented!(),
        };
        statement
    }

    pub fn parse_return(&self, rules: Pair<Rule>) -> Return {
        let mut rules_iter = rules.clone().into_inner();
        let exp = rules_iter.next().map(|rule| self.parse_expr(rule));
        Return {
            value: exp,
            span: self.get_span(rules.as_span().clone()),
        }
    }
    
    pub fn parse_const_exp(&self, rules: Pair<Rule>) -> ConstExp {
        let mut rules_iter = rules.clone().into_inner();
        let exp = rules_iter.next().unwrap();
        ConstExp {
            exp: self.parse_expr(exp),
        }
    }

    pub fn parse_expr(&self, rules: Pair<Rule>) -> Exp {
        let pratt = PrattParser::new()
            .op(Op::infix(Rule::eq, Assoc::Left) | Op::infix(Rule::neq, Assoc::Left))
            .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
            .op(Op::infix(Rule::mul, Assoc::Left)
                | Op::infix(Rule::div, Assoc::Left)
                | Op::infix(Rule::r#mod, Assoc::Left))
            .op(Op::prefix(Rule::neg) | Op::prefix(Rule::pos));

        pratt
            .map_primary(|primary| match primary.as_rule() {
                Rule::exp => self.parse_expr(primary),
                Rule::number => Exp::Number(Number {
                    num: primary.as_str().parse().unwrap(),
                    span: self.get_span(primary.as_span()),
                }),
                Rule::lval => Exp::LVal(Box::new(self.parse_lval(primary))),

                _ => panic!("Unkown primary {}!", primary),
            })
            .map_prefix(|op, rhs| match op.as_rule() {
                Rule::neg => Exp::Unary(UnaryOp::Negative, Box::new(rhs), self.get_span(op.as_span())),
                Rule::pos => Exp::Unary(UnaryOp::Positive, Box::new(rhs), self.get_span(op.as_span())),
                _ => unimplemented!(),
            })
            .map_postfix(|_lhs, _op| unimplemented!())
            .map_infix(|lhs, op, rhs| {
                let lhs = Box::new(lhs);
                let rhs = Box::new(rhs);

                Exp::Binary(
                    lhs,
                    match op.as_rule() {
                        Rule::eq => BinaryOp::Eq,
                        Rule::neq => BinaryOp::Neq,
                        Rule::add => BinaryOp::Add,
                        Rule::sub => BinaryOp::Sub,
                        Rule::mul => BinaryOp::Mul,
                        Rule::div => BinaryOp::Div,
                        Rule::r#mod => BinaryOp::Mod,
                        _ => unimplemented!(),
                    },
                    rhs,
                    self.get_span(op.as_span()),
                )
            })
            .parse(rules.into_inner())
    }

    pub fn parse_type(&self, rules: Pair<Rule>) -> Type {
        let mut rules_iter = rules.clone().into_inner();

        let vtype_enum = rules_iter.next().unwrap();

        let vty_enum = match vtype_enum.as_str() {
            "u64" => TypeEnum::U64,
            _ => panic!("Unkown type {}!", vtype_enum.as_str()),
        };

        let mut star_cnt = 0usize;
        while let Some(_) = rules_iter.next() {
            star_cnt += 1;
        }
        Type {
            ty: vty_enum,
            star: star_cnt,
            span: self.get_span(rules.as_span().clone()),
        }
    }
}
