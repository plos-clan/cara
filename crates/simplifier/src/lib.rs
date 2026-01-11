use ast::{CompUnit, ConstDef, ConstExp, ConstInitialValue, Exp, GlobalItem, Type, TypeEnum};

pub fn simplify(ast: CompUnit) -> CompUnit {
    let mut ctx = SimplifierContext {
        parents: Vec::new(),
    };

    let CompUnit { global_items, span } = ast;
    let mut new_items = Vec::new();
    for item in global_items {
        let some_items = ctx.simp_item(item);
        new_items.extend(some_items);
    }

    CompUnit {
        global_items: new_items,
        span,
    }
}

struct SimplifierContext {
    parents: Vec<String>,
}

impl SimplifierContext {
    fn simp_item(&mut self, item: GlobalItem) -> Vec<GlobalItem> {
        let GlobalItem::ConstDef(const_def) = item;
        self.simp_const_def(const_def)
    }

    fn simp_const_def(&mut self, const_def: ConstDef) -> Vec<GlobalItem> {
        let ConstDef {
            name: raw_name,
            initial_value,
            span,
        } = const_def;
        let name = if self.parents.is_empty() {
            raw_name.clone()
        } else {
            format!("{}::{}", self.parents.join("::"), raw_name)
        };
        match initial_value {
            ConstInitialValue::Exp(ConstExp { exp: Exp::Type(ty) }) => {
                self.parents.push(raw_name.clone());
                let (ty, mut extra) = self.simp_type(ty);
                self.parents.pop();
                extra.push(GlobalItem::ConstDef(ConstDef {
                    name,
                    initial_value: ConstInitialValue::Exp(ConstExp { exp: Exp::Type(ty) }),
                    span,
                }));
                extra
            }
            ConstInitialValue::Exp(exp) => {
                vec![GlobalItem::ConstDef(ConstDef {
                    name,
                    initial_value: ConstInitialValue::Exp(exp),
                    span,
                })]
            }
        }
    }

    fn simp_type(&mut self, ty: Type) -> (Type, Vec<GlobalItem>) {
        if let TypeEnum::Structure(fields, items) = ty.kind {
            let mut new_items = Vec::new();
            for item in items {
                new_items.extend(self.simp_item(item));
            }
            (
                Type {
                    kind: TypeEnum::Structure(fields, vec![]),
                    span: ty.span,
                },
                new_items,
            )
        } else {
            (ty, vec![])
        }
    }
}
