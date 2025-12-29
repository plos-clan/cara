use std::collections::BTreeMap;

pub use provider::*;
use rayon::{ThreadPool, ThreadPoolBuilder};

mod provider;

pub struct QuerySystem<A, V> {
    providers: BTreeMap<ProviderId, Provider<A, V>>,
    thread_pool: ThreadPool,
}

impl<A, V> QuerySystem<A, V> {
    pub fn new() -> Self {
        Self {
            providers: BTreeMap::new(),
            thread_pool: ThreadPoolBuilder::new().build().unwrap(),
        }
    }

    pub fn register_provider(&mut self, provider: Provider<A, V>) -> ProviderId {
        let id = ProviderId(self.providers.len() as u64);
        self.providers.insert(id, provider);
        id
    }
}

impl<A: Send + Sync, V: Send + Sync> QuerySystem<A, V> {
    pub fn query(&self, provider: ProviderId, arg: A) -> V {
        self.thread_pool
            .install(|| self.providers.get(&provider).unwrap()(arg))
    }
}
