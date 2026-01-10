use crate::Value;

pub enum Symbol {
    Var(String, bool, Value),
}

impl symbol_table::Symbol for Symbol {
    type Key = String;

    fn key(&self) -> &Self::Key {
        self.name()
    }
}

impl Symbol {
    pub fn name(&self) -> &String {
        match self {
            Symbol::Var(name, _, _) => name,
        }
    }
}
