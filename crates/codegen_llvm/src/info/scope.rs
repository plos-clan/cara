use crate::info::Value;

#[derive(Debug)]
pub enum Symbol<'s> {
    Var(String, Value<'s>),
}

impl symbol_table::Symbol for Symbol<'_> {
    type Key = String;

    fn key(&self) -> &Self::Key {
        match self {
            Self::Var(key, _) => key,
        }
    }
}
