use std::sync::{Arc, RwLock};

use inkwell::{
    builder::Builder, context::Context, module::Module, passes::PassBuilderOptions, targets::TargetMachine, types::BasicType, values::PointerValue
};

use super::*;
use crate::ast::*;
use super::error::{Error, Result};

mod decl;
mod program;

pub struct Generator<'gen> {
    pub name: String,
    pub context: &'gen Context,
    pub module: Module<'gen>,
    pub builder: Builder<'gen>,
    pub errors: Vec<Error>,
    pub local: SymbolTable<'gen>,
    pub global: SymbolTable<'gen>,
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
        }
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

