use std::collections::VecDeque;

use super::Value;

#[derive(Debug, Clone)]
pub enum Symbol<'sym> {
    Const(String, Value<'sym>),
}


#[derive(Debug, Clone)]
pub struct SymbolTable<'sym> {
    stack: VecDeque<Symbol<'sym>>,
}

impl<'sym> SymbolTable<'sym> {
    pub fn new() -> Self {
        Self {
            stack: VecDeque::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn pop(&mut self) -> Option<Symbol<'sym>> {
        self.stack.pop_front()
    }

    pub fn push(&mut self, symbol: Symbol<'sym>) {
        self.stack.push_front(symbol);
    }

    pub fn get(&self, name: &str) -> Option<&Symbol<'sym>> {
        self.stack.iter().find(|symbol| match symbol {
            Symbol::Const(n, _) => n == name,
        })
    }
}
