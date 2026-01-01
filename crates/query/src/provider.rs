use std::{collections::BTreeMap, marker::Tuple, sync::RwLock};

use crate::QueryContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProviderId(u64);

type Provider<A, R> = Box<dyn Fn(&QueryContext, A) -> R + Send + Sync>;

pub struct Providers<A: Tuple, R> {
    pub(crate) providers: BTreeMap<ProviderId, Provider<A, R>>,
    pub(crate) cache: RwLock<BTreeMap<ProviderId, BTreeMap<A, R>>>,
}

impl<A: Tuple, R> Providers<A, R> {
    pub fn new() -> Self {
        Self {
            providers: BTreeMap::new(),
            cache: RwLock::new(BTreeMap::new()),
        }
    }
}

impl<A: Tuple, R> Default for Providers<A, R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Tuple, R> Providers<A, R> {
    pub fn register(&mut self, provider: Provider<A, R>) -> ProviderId {
        let id = ProviderId(self.providers.len() as u64);
        self.providers.insert(id, provider);
        self.cache.write().unwrap().insert(id, Default::default());
        id
    }
}
