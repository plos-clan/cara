use std::collections::BTreeMap;

use ast::{CompUnit, ConstDef, GlobalItem};

pub struct ConstDefTable<'a> {
    defs: BTreeMap<String, &'a ConstDef>,
}

impl<'a> ConstDefTable<'a> {
    pub fn new(unit: &'a CompUnit) -> Self {
        let mut defs = BTreeMap::new();
        for item in &unit.global_items {
            let GlobalItem::ConstDef(def) = item;
            defs.insert(def.name.clone(), def);
        }
        ConstDefTable { defs }
    }

    pub fn get(&self, name: &str) -> Option<ConstDef> {
        self.defs.get(name).cloned().cloned()
    }
}
