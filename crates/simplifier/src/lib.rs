use ast::{CompUnit, ConstDef, ConstExp, ConstInitialValue, GlobalItem, Type, TypeEnum};
use symbol_table::SymbolTable;

use crate::namespace::NameSpaces;

mod exp;
mod namespace;
mod stmt;

pub fn simplify(ast: CompUnit) -> CompUnit {
    let mut ctx = SimplifierContext::new();

    let CompUnit { global_items, span } = ast;
    let mut new_items = Vec::new();
    for item in global_items {
        let new_item = ctx.simp_item(item);
        new_items.push(new_item);
    }

    let SimplifierContext { extra_items, .. } = ctx;

    new_items.extend(extra_items);

    CompUnit {
        global_items: new_items,
        span,
    }
}

struct SimplifierContext {
    globals: NameSpaces,
    locals: SymbolTable<String>,
    extra_items: Vec<GlobalItem>,
}

impl SimplifierContext {
    fn new() -> Self {
        Self {
            globals: {
                let mut globals = NameSpaces::new_root();
                globals.push_layer();
                globals
            },
            locals: SymbolTable::new(),
            extra_items: Vec::new(),
        }
    }
}

impl SimplifierContext {
    fn simp_item(&mut self, item: GlobalItem) -> GlobalItem {
        let GlobalItem::ConstDef(const_def) = item;
        self.simp_const_def(const_def)
    }

    fn simp_const_def(&mut self, const_def: ConstDef) -> GlobalItem {
        let ConstDef {
            name: raw_name,
            initial_value,
            span,
        } = const_def;
        let name = self.globals.prefixed_name(&raw_name);
        self.globals.set_name_cache(raw_name.clone());
        match initial_value {
            ConstInitialValue::Exp(exp) => {
                let exp = self.simp_exp(exp.exp.clone());
                GlobalItem::ConstDef(ConstDef {
                    name,
                    initial_value: ConstInitialValue::Exp(ConstExp { exp }),
                    span,
                })
            }
        }
    }

    fn simp_type(&mut self, ty: Type) -> Type {
        if let TypeEnum::Structure(fields, items) = ty.kind {
            self.globals.push_layer();
            for item in &items {
                match item {
                    GlobalItem::ConstDef(ConstDef { name, .. }) => {
                        self.globals.add_symbol(name.clone());
                    }
                }
            }
            for item in items {
                let item = self.simp_item(item);
                self.extra_items.push(item);
            }
            self.globals.pop_layer();
            Type {
                kind: TypeEnum::Structure(fields, vec![]),
                span: ty.span,
            }
        } else {
            ty
        }
    }
}
