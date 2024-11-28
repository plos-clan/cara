use std::sync::{Arc, RwLock};

use inkwell::{
    builder::Builder, context::Context, module::Module, passes::PassBuilderOptions,
    targets::TargetMachine,
    values::*,
    types::*,
};

use anyhow::Result;
use super::*;
use crate::ast::*;

mod comptime;
mod decl;
mod expr;
mod program;
mod stmt;

pub struct Generator<'gen> {
    pub name: String,
    pub context: &'gen Context,
    pub module: Module<'gen>,
    pub builder: Builder<'gen>,
    pub errors: Vec<CompileError>,
    pub local: SymbolTable<'gen>,
    pub global: SymbolTable<'gen>,
    pub(self) current_const_name: Option<String>,
}

impl<'gen> Generator<'gen> {
    pub fn new(
        name: String,
        context: &'gen Context,
        module: Module<'gen>,
        builder: Builder<'gen>,
    ) -> Self {
        Self {
            name,
            context,
            module,
            builder,
            errors: Vec::new(),
            local: SymbolTable::new(),
            global: SymbolTable::new(),
            current_const_name: None,
        }
    }

    pub(self) fn current_function<'a>(&'a self, span: Span) -> Option<Function<'a>> {
        if let Some(func_name) = self.current_const_name.clone() {
            let function = match self.global.get(&func_name).unwrap() {
                Symbol::Const(_, value) => value.as_function(span.clone()).ok()?,
                _ => return None,
            };
            return Some(function);
        }
        None
    }

    fn create_entry_block_alloca<T: BasicType<'gen>>(
        &self,
        name: &str,
        ty: T,
        span: Span,
    ) -> PointerValue<'gen> {
        let builder = &self.builder;

        let entry = self
            .current_function(span)
            .unwrap()
            .get_first_basic_block()
            .unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(ty, name).unwrap()
    }
    
    pub fn get_symbol(&self, name: &str) -> Option<&Symbol<'gen>> {
        if let Some(symbol) = self.global.get(name) {
            return Some(symbol);
        } else if let Some(symbol) = self.local.get(name) {
            return Some(symbol);
        }
        None
    }

    pub fn prepare(&self, target_machine: &TargetMachine) {
        let passes: &[&str] = &["codegenprepare"];

        self.module
            .run_passes(
                passes.join(",").as_str(),
                target_machine,
                PassBuilderOptions::create(),
            )
            .unwrap();
    }

    pub fn run_passes(&self, target_machine: &TargetMachine) {
        let passes: &[&str] = &[
            "instcombine",
            "reassociate",
            "gvn",
            "simplifycfg",
            //"basic-aa",
            "mem2reg",
            "dce",
            "dse",
        ];

        self.module
            .run_passes(
                passes.join(",").as_str(),
                target_machine,
                PassBuilderOptions::create(),
            )
            .unwrap();
    }
}

/// Trait for generating IR program.
pub trait GenerateProgramTwice<'gen> {
    type Out;

    fn decl(&self, gen: Arc<RwLock<Generator<'gen>>>) -> Result<Self::Out>;
    fn implement(&self, gen: Arc<RwLock<Generator<'gen>>>) -> Result<Self::Out>;
}

pub trait GenerateProgramOnce<'gen> {
    type Out;

    fn generate(&self, gen: Arc<RwLock<Generator<'gen>>>) -> Result<Self::Out>;
}
