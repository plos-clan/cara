#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProviderId(pub(crate) u64);

pub type Provider<A, V> = Box<dyn Fn(A) -> V + Send + Sync>;
