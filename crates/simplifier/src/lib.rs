use std::{collections::HashMap, sync::Arc};

use ast::{
    AstContext, ConstDef, ConstExp, ConstInitialValue, Exp, ExpId, GlobalItem, StructType, Type,
    TypeEnum,
};
use symbol_table::SymbolTable;

use crate::namespace::NameSpaces;

mod exp;
mod namespace;
mod stmt;

pub fn simplify(crate_name: String, ast: AstContext) -> AstContext {
    let (exps, root) = ast.into_tuple();
    let mut ctx = SimplifierContext::new(crate_name, exps);

    let span = ctx.simp_struct_ty(root).span;

    let SimplifierContext {
        extra_items, exps, ..
    } = ctx;

    AstContext::new(
        exps,
        StructType {
            fields: HashMap::new(),
            members: extra_items,
            span,
        },
    )
}

struct SimplifierContext {
    origin_exps: Vec<HashMap<ExpId, Exp>>,
    crate_name: String,
    globals: NameSpaces,
    locals: SymbolTable<String>,
    extra_items: Vec<GlobalItem>,
    exps: HashMap<ExpId, Exp>,
}

impl SimplifierContext {
    fn new(crate_name: String, origin_exps: HashMap<ExpId, Exp>) -> Self {
        Self {
            origin_exps: vec![origin_exps],
            crate_name: crate_name.clone(),
            globals: {
                let mut globals = NameSpaces::new_root();
                globals.push_layer();
                globals.set_name_cache(crate_name);
                globals
            },
            locals: SymbolTable::new(),
            extra_items: Vec::new(),
            exps: HashMap::new(),
        }
    }

    fn crate_name(&self) -> String {
        self.crate_name.clone()
    }

    fn insert_exp(&mut self, exp: Exp) -> ExpId {
        let id = self.exps.len() as u64;
        let id = ExpId::new(id, exp.span());
        self.exps.insert(id, exp);
        id
    }

    fn get_exp(&mut self, id: ExpId) -> Option<Exp> {
        self.origin_exps.last_mut().unwrap().remove(&id)
    }
}

impl SimplifierContext {
    fn simp_item(&mut self, item: GlobalItem) -> GlobalItem {
        let GlobalItem::ConstDef(const_def) = item;
        self.simp_const_def(const_def)
    }

    fn simp_const_def(&mut self, const_def: Arc<ConstDef>) -> GlobalItem {
        let ConstDef {
            name: raw_name,
            initial_value,
            span,
        } = const_def.as_ref();
        let name = self.globals.prefixed_name(raw_name);
        self.globals.set_name_cache(raw_name.clone());
        match initial_value {
            ConstInitialValue::Exp(exp) => {
                let exp = self.simp_exp(exp.exp);
                GlobalItem::ConstDef(Arc::new(ConstDef {
                    name,
                    initial_value: ConstInitialValue::Exp(ConstExp { exp }),
                    span: *span,
                }))
            }
        }
    }

    fn simp_type(&mut self, ty: Type) -> Type {
        if let TypeEnum::Structure(struct_ty) = ty.kind {
            let struct_ty = self.simp_struct_ty(struct_ty);
            Type {
                kind: TypeEnum::Structure(struct_ty),
                span: ty.span,
            }
        } else {
            ty
        }
    }

    fn simp_struct_ty(&mut self, struct_ty: StructType) -> StructType {
        let StructType {
            fields,
            members,
            span,
        } = struct_ty;
        let fields = fields
            .into_iter()
            .map(|(name, ty)| {
                let ty = self.simp_exp(ty);
                (name, ty)
            })
            .collect();
        self.globals.push_layer();
        for item in &members {
            match item {
                GlobalItem::ConstDef(const_def) => {
                    self.globals.add_symbol(const_def.name.clone());
                }
            }
        }
        for item in members {
            let item = self.simp_item(item);
            self.extra_items.push(item);
        }
        self.globals.pop_layer();
        StructType {
            fields,
            members: vec![],
            span,
        }
    }
}
