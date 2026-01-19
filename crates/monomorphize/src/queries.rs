use std::{
    collections::HashSet,
    sync::{Arc, LazyLock},
};

use ast::{FunctionDef, visitor::ExpVisitor};
use const_eval::queries::CONST_EVAL_PROVIDER;
use query::{Provider, QueryContext};
use symbol_table::SymbolTable;

use crate::{CodegenItem, MonomorphizeContext};

pub static COLLECT_CODEGEN_UNITS: LazyLock<Provider<(), Vec<CodegenItem>>> =
    LazyLock::new(|| Provider::new(collect_codegen_units));

fn collect_codegen_units(ctx: Arc<QueryContext>, (): ()) -> Vec<CodegenItem> {
    let initial = ctx.main_fn_id();
    let const_eval::ValueKind::Function(initial) = ctx
        .query_cached(&CONST_EVAL_PROVIDER, initial)
        .unwrap()
        .kind()
    else {
        return Vec::new();
    };
    let initial = CodegenItem::Func(initial);

    let mut required = HashSet::new();

    required.insert(initial.clone());

    let mut new_ones = vec![initial];

    while !new_ones.is_empty() {
        let mut new_new_ones = Vec::new();
        for new_one in new_ones.iter() {
            let CodegenItem::Func(new_one) = new_one else {
                continue;
            };
            let collected = collect_required_items(ctx.clone(), new_one.clone());
            for item in collected {
                if !required.contains(&item) {
                    required.insert(item.clone());
                    new_new_ones.push(item);
                }
            }
        }

        new_ones = new_new_ones;
    }

    required.into_iter().collect()
}

fn collect_required_items(
    ctx: Arc<QueryContext>,
    func_def: Arc<FunctionDef>,
) -> HashSet<CodegenItem> {
    let mut visitor_ctx = MonomorphizeContext {
        ctx,
        locals: SymbolTable::new(),
        required_items: HashSet::new(),
    };

    for param in &func_def.params {
        visitor_ctx.locals.pre_push(param.name.clone());
    }

    visitor_ctx.visit_block(&func_def.block);

    visitor_ctx.required_items
}
