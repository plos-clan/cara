use crate::info::Value;

pub struct SymbolStack<'s> {
    cache: Vec<Symbol<'s>>,
    symbols: Vec<Vec<Symbol<'s>>>,
}

#[derive(Debug)]
pub enum Symbol<'s> {
    Var(String, Value<'s>),
}

impl<'s> SymbolStack<'s> {
    pub fn new() -> Self {
        SymbolStack {
            symbols: Vec::new(),
            cache: Vec::new(),
        }
    }

    pub fn push_scope(&mut self) {
        let mut scope = Vec::new();
        for symbol in self.cache.drain(..) {
            scope.push(symbol);
        }
        self.symbols.push(scope);
    }

    pub fn pop_scope(&mut self) {
        self.symbols.pop();
    }

    pub fn pre_push(&mut self, symbol: Symbol<'s>) {
        self.cache.push(symbol);
    }

    pub fn push(&mut self, symbol: Symbol<'s>) {
        let len = self.symbols.len();
        self.symbols[len - 1].push(symbol);
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol<'s>> {
        for table in self.symbols.iter().rev() {
            for symbol in table.iter().rev() {
                match symbol {
                    Symbol::Var(symbol_name, _) => {
                        if symbol_name == name {
                            return Some(symbol);
                        }
                    }
                }
            }
        }
        None
    }
}
