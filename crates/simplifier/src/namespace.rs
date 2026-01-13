use std::collections::HashSet;

pub struct NameSpaces {
    name_cache: Option<String>,
    stack: Vec<String>,
    symbols: Vec<HashSet<String>>,
}

impl NameSpaces {
    pub fn new_root() -> Self {
        Self {
            name_cache: None,
            stack: Vec::new(),
            symbols: Vec::new(),
        }
    }
}

impl NameSpaces {
    pub fn set_name_cache(&mut self, name: String) {
        self.name_cache = Some(name);
    }

    pub fn push_layer(&mut self) {
        self.stack.push(self.name_cache.clone().unwrap_or_default());
        self.symbols.push(HashSet::new());
    }

    pub fn pop_layer(&mut self) {
        self.stack.pop();
        self.symbols.pop();
    }

    pub fn add_symbol(&mut self, raw_name: String) {
        if let Some(symbols) = self.symbols.last_mut() {
            symbols.insert(raw_name);
        }
    }

    pub fn prefixed_name<S: AsRef<str>>(&self, raw_name: S) -> String {
        let mut name = raw_name.as_ref().to_string();
        for layer in self.stack.iter().rev() {
            name = format!("{}::{}", layer, name);
        }
        name
    }

    pub fn prefixes(&self) -> Vec<String> {
        self.stack.clone()
    }

    pub fn super_prefixes(&self) -> Vec<String> {
        let len = self.stack.len();
        self.stack.iter().cloned().take(len - 1).collect()
    }

    pub fn lookup_current(&self, name: &String) -> bool {
        self.symbols.last().unwrap().contains(name)
    }
}
