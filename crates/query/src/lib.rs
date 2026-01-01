#![feature(tuple_trait)]

use std::{collections::BTreeMap, marker::Tuple};

use ast::{CompUnit, ConstDef, GlobalItem};
pub use defs::*;
pub use provider::*;
use rayon::{ThreadPool, ThreadPoolBuilder};

mod defs;
mod provider;

pub struct QueryContext<'d> {
    consts: BTreeMap<DefId, &'d ConstDef>,
    thread_pool: ThreadPool,
}

impl<'d> QueryContext<'d> {
    pub fn new(ast: &'d CompUnit) -> Self {
        let mut consts = BTreeMap::new();
        for GlobalItem::ConstDef(const_def) in &ast.global_items {
            let id = DefId(consts.len());
            consts.insert(id, const_def);
        }
        Self {
            consts,
            thread_pool: ThreadPoolBuilder::new().build().unwrap(),
        }
    }
}

impl<'d> QueryContext<'d> {
    pub fn query<A: Tuple + Send + Sync, R: Send + Sync>(
        &self,
        providers: &Providers<A, R>,
        provider: ProviderId,
        arg: A,
    ) -> Option<R> {
        let provider = providers
            .providers
            .get(&provider)
            .map(|provider| provider)?;

        Some(self.thread_pool.install(|| provider(self, arg)))
    }

    pub fn query_cached<A: Tuple + Ord + Send + Sync + Clone, R: Send + Sync + Clone>(
        &self,
        providers: &Providers<A, R>,
        provider: ProviderId,
        arg: A,
    ) -> Option<R> {
        if let Some(cache) = providers.cache.read().unwrap().get(&provider) {
            if let Some(value) = cache.get(&arg) {
                return Some(value.clone());
            }
        }

        let result = self.query(providers, provider, arg.clone());
        if let Some(result) = result.clone() {
            providers
                .cache
                .write()
                .unwrap()
                .entry(provider)
                .or_default()
                .insert(arg, result.clone());
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
}
