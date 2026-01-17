use std::{
    collections::HashSet,
    sync::{Arc, LazyLock},
};

use ast::visitor::ExpVisitor;
use const_eval::{ValueKind, queries::CONST_EVAL_PROVIDER};
use query::{DefId, Provider, QueryContext};
use symbol_table::SymbolTable;

use crate::MonomorphizeContext;

pub static COLLECT_CODEGEN_UNITS: LazyLock<Provider<(), Vec<DefId>>> =
    LazyLock::new(|| Provider::new(collect_codegen_units));

fn collect_codegen_units(ctx: Arc<QueryContext>, (): ()) -> Vec<DefId> {
    let initial = ctx.main_fn_id();
    let mut required = HashSet::new();

    required.insert(initial);

    let mut new_ones = vec![initial];

    while !new_ones.is_empty() {
        let mut new_new_ones = Vec::new();
        for new_one in new_ones.iter() {
            let collected = collect_required_items(ctx.clone(), *new_one);
            for def_id in collected {
                if !required.contains(&def_id) {
                    required.insert(def_id);
                    new_new_ones.push(def_id);
                }
            }
        }

        new_ones = new_new_ones;
    }

    required.iter().copied().collect()
}

fn collect_required_items(ctx: Arc<QueryContext>, def_id: DefId) -> Vec<DefId> {
    let Some(ValueKind::Function(func_def)) = ctx
        .query_cached(&CONST_EVAL_PROVIDER, def_id)
        .map(|v| v.kind())
    else {
        return Vec::new();
    };

    let mut visitor_ctx = MonomorphizeContext {
        ctx,
        locals: SymbolTable::new(),
        required_items: Vec::new(),
    };

    for param in &func_def.params {
        visitor_ctx.locals.pre_push(param.name.clone());
    }

    visitor_ctx.visit_block(&func_def.block);

    visitor_ctx.required_items
}
