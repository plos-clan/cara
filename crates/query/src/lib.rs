use std::{collections::HashMap, hash::Hash, sync::Arc};

use ast::{AstContext, ConstDef, GlobalItem};
use bon::bon;
pub use defs::*;
pub use provider::*;
use rayon::{ThreadPool, ThreadPoolBuilder};
use targets::spec::Target;

mod defs;
mod provider;

pub struct QueryContext {
    crate_name: String,
    target: Target,
    ast_ctx: Arc<AstContext>,
    consts: HashMap<DefId, Arc<ConstDef>>,
    thread_pool: ThreadPool,
}

#[bon]
impl QueryContext {
    #[builder]
    pub fn new(crate_name: String, target: Target, ast: Arc<AstContext>) -> Arc<Self> {
        let mut consts = HashMap::new();
        for GlobalItem::ConstDef(const_def) in ast.root.members.iter() {
            let id = DefId(consts.len());
            consts.insert(id, const_def.clone());
        }
        Arc::new(Self {
            crate_name,
            target,
            ast_ctx: ast,
            consts,
            thread_pool: ThreadPoolBuilder::new().build().unwrap(),
        })
    }
}

impl QueryContext {
    pub fn crate_name(&self) -> String {
        self.crate_name.clone()
    }

    pub fn main_fn_id(&self) -> DefId {
        self.lookup_def_id(format!("::{}::main", self.crate_name))
            .unwrap()
    }

    pub fn ast_ctx(&self) -> Arc<AstContext> {
        self.ast_ctx.clone()
    }

    pub fn target(&self) -> &Target {
        &self.target
    }
}

impl QueryContext {
    pub fn query<A: Send + Sync, R: Send + Sync>(
        self: &Arc<Self>,
        provider: &Provider<A, R>,
        arg: A,
    ) -> Option<R> {
        Some(self.thread_pool.install(|| (provider.f)(self.clone(), arg)))
    }

    pub fn query_cached<A: Hash + Eq + Send + Sync + Clone, R: Send + Sync + Clone>(
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

impl QueryContext {
    pub fn lookup_def_id<S: AsRef<str>>(&self, name: S) -> Option<DefId> {
        self.consts
            .keys()
            .find(|&&id| self.consts.get(&id).unwrap().name == name.as_ref())
            .copied()
    }

    pub fn get_def(&self, def_id: DefId) -> Option<&ConstDef> {
        self.consts.get(&def_id).map(|d| d.as_ref())
    }

    pub fn def_ids(&self) -> Vec<DefId> {
        self.consts.keys().copied().collect()
    }
}
