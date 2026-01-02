use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use crate::QueryContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProviderId(u64);

pub struct Provider<A, R> {
    pub(crate) f: Box<dyn Fn(Arc<QueryContext>, A) -> R + Send + Sync>,
    pub(crate) cache: RwLock<BTreeMap<A, R>>,
}

impl<A, R> Provider<A, R> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(Arc<QueryContext>, A) -> R + Send + Sync + 'static,
    {
        Self {
            f: Box::new(f),
            cache: RwLock::new(BTreeMap::new()),
        }
    }
}
