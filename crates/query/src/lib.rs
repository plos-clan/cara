use std::{collections::BTreeMap, sync::Arc};

use ast::{CompUnit, ConstDef, GlobalItem};
pub use defs::*;
pub use provider::*;
use rayon::{ThreadPool, ThreadPoolBuilder};

mod defs;
mod provider;

pub struct QueryContext<'d> {
    crate_name: String,
    consts: BTreeMap<DefId, &'d ConstDef>,
    thread_pool: ThreadPool,
}

impl<'d> QueryContext<'d> {
    pub fn new(crate_name: String, ast: &'d CompUnit) -> Arc<Self> {
        let mut consts = BTreeMap::new();
        for GlobalItem::ConstDef(const_def) in &ast.global_items {
            let id = DefId(consts.len());
            consts.insert(id, const_def);
        }
        Arc::new(Self {
            crate_name,
            consts,
            thread_pool: ThreadPoolBuilder::new().build().unwrap(),
        })
    }
}

impl<'d> QueryContext<'d> {
    pub fn crate_name(&self) -> String {
        self.crate_name.clone()
    }

    pub fn main_fn_id(&self) -> DefId {
        self.lookup_def_id(format!("::{}::main", self.crate_name))
            .unwrap()
    }
}

impl<'d> QueryContext<'d> {
    pub fn query<A: Send + Sync, R: Send + Sync>(
        self: &Arc<Self>,
        provider: &Provider<A, R>,
        arg: A,
    ) -> Option<R> {
        Some(self.thread_pool.install(|| (provider.f)(self.clone(), arg)))
    }

    pub fn query_cached<A: Ord + Send + Sync + Clone, R: Send + Sync + Clone>(
        self: &Arc<Self>,
        provider: &Provider<A, R>,
        arg: A,
    ) -> Option<R> {
        if let Some(value) = provider.cache.read().unwrap().get(&arg) {
            return Some(value.clone());
        }

        let result = self.query(provider, arg.clone());
        if let Some(result) = result.clone() {
            provider.cache.write().unwrap().insert(arg, result.clone());
        }
        result
    }
}

impl<'d> QueryContext<'d> {
    pub fn lookup_def_id<S: AsRef<str>>(&self, name: S) -> Option<DefId> {
        self.consts
            .keys()
            .find(|&&id| self.consts.get(&id).unwrap().name == name.as_ref())
            .copied()
    }

    pub fn get_def(&self, def_id: DefId) -> Option<&&ConstDef> {
        self.consts.get(&def_id)
    }

    pub fn def_ids(&self) -> Vec<DefId> {
        self.consts.keys().copied().collect()
    }
}
