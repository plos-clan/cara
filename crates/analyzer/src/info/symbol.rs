use crate::Value;

pub enum Symbol {
    Const(String, Value),
    Var(String, bool, Value),
}

impl Symbol {
    pub fn name(&self) -> &str {
        match self {
            Symbol::Const(name, _) => name,
            Symbol::Var(name, _, _) => name,
        }
    }
}
