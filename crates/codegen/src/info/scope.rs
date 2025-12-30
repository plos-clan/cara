use crate::info::Value;

pub struct SymbolStack<'s> {
    symbols: Vec<Vec<Symbol<'s>>>,
}

pub enum Symbol<'s> {
    Const(String, Value<'s>),
}

impl<'s> Symbol<'s> {
    pub fn value(&self) -> &Value<'s> {
        match self {
            Symbol::Const(_, value) => value,
        }
    }
}

impl<'s> SymbolStack<'s> {
    pub fn new() -> Self {
        SymbolStack {
            symbols: Vec::new(),
        }
    }

    pub fn push_scope(&mut self) {
        self.symbols.push(Vec::new());
    }

    pub fn pop_scope(&mut self) {
        self.symbols.pop();
    }

    pub fn push(&mut self, symbol: Symbol<'s>) {
        let len = self.symbols.len();
        self.symbols[len - 1].push(symbol);
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol<'s>> {
        for table in self.symbols.iter().rev() {
            for symbol in table.iter().rev() {
                match symbol {
                    Symbol::Const(symbol_name, _) => {
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
