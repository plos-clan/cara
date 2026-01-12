pub trait Symbol {
    type Key: Ord;
    fn key(&self) -> &Self::Key;
}

impl Symbol for String {
    type Key = String;
    fn key(&self) -> &Self::Key {
        self
    }
}

pub struct SymbolTable<S: Symbol> {
    symbols: Vec<Vec<S>>,
    cache: Vec<S>,
}

impl<S: Symbol> SymbolTable<S> {
    pub fn new() -> Self {
        SymbolTable {
            symbols: Vec::new(),
            cache: Vec::new(),
        }
    }
}

impl<S: Symbol> Default for SymbolTable<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Symbol> SymbolTable<S> {
    pub fn pre_push(&mut self, symbol: S) {
        self.cache.push(symbol);
    }

    pub fn push_scope(&mut self) {
        self.symbols.push(Vec::new());
        for symbol in self.cache.drain(..) {
            self.symbols.last_mut().unwrap().push(symbol);
        }
    }

    pub fn pop_scope(&mut self) {
        self.symbols.pop();
    }

    pub fn push(&mut self, symbol: S) {
        self.symbols.last_mut().unwrap().push(symbol);
    }

    pub fn lookup(&self, key: &S::Key) -> Option<&S> {
        self.symbols
            .iter()
            .rev()
            .find_map(|scope| scope.iter().rev().find(|symbol| symbol.key() == key))
    }

    pub fn lookup_current(&self, key: &S::Key) -> Option<&S> {
        self.symbols
            .last()
            .and_then(|scope| scope.iter().rev().find(|symbol| symbol.key() == key))
    }

    pub fn contains(&self, key: &S::Key) -> bool {
        self.symbols
            .iter()
            .rev()
            .any(|scope| scope.iter().any(|symbol| symbol.key() == key))
    }
}
