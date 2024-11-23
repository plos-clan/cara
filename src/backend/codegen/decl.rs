use inkwell::{llvm_sys::core::LLVMSetValueName2, values::AsValueRef};

use super::{
    BackendType, ConstDecl, ConstInitialValue, ConstInitialValueEnum, Function, FunctionDef,
    GenerateProgramOnce, GenerateProgramTwice, Value,
};

impl<'gen> GenerateProgramTwice<'gen> for ConstDecl {
    type Out = ();

    fn decl(
        &self,
        gen: std::sync::Arc<std::sync::RwLock<super::Generator<'gen>>>,
    ) -> super::error::Result<Self::Out> {
        let value = self.initial_value.decl(gen.clone())?;

        if let Ok(value) = value.as_function(self.span.clone()) {
            unsafe {
                LLVMSetValueName2(value.as_value_ref(), self.name.as_ptr() as *const i8, self.name.len());
            }
        }

        Ok(())
    }

    fn implement(
        &self,
        gen: std::sync::Arc<std::sync::RwLock<super::Generator<'gen>>>,
    ) -> super::error::Result<Self::Out> {
        let value = self.initial_value.implement(gen.clone())?;

        Ok(())
    }
}

impl<'gen> GenerateProgramTwice<'gen> for ConstInitialValue {
    type Out = Value<'gen>;

    fn decl(
        &self,
        gen: std::sync::Arc<std::sync::RwLock<super::Generator<'gen>>>,
    ) -> super::error::Result<Self::Out> {
        match &self.value {
            ConstInitialValueEnum::Function(func) => func.decl(gen),
        }
    }

    fn implement(
        &self,
        gen: std::sync::Arc<std::sync::RwLock<super::Generator<'gen>>>,
    ) -> super::error::Result<Self::Out> {
        match &self.value {
            ConstInitialValueEnum::Function(func) => func.implement(gen),
        }
    }
}

impl<'gen> GenerateProgramTwice<'gen> for FunctionDef {
    type Out = Value<'gen>;

    fn decl(
        &self,
        gen: std::sync::Arc<std::sync::RwLock<super::Generator<'gen>>>,
    ) -> super::error::Result<Self::Out> {
        let return_type = self.return_type.generate(gen.clone())?;

        let mut param_types = Vec::new();

        for param in self.params.iter() {
            param_types.push(
                param
                    .param_type
                    .generate(gen.clone())?
                    .as_type(param.span.clone())?,
            );
        }

        let function_type = return_type
            .as_type(self.span.clone())?
            .function_type(param_types, self.span.clone())?;

        let func = gen.read().unwrap().module.add_function(
            "",
            function_type.clone().as_function(self.span.clone())?,
            None,
        );

        for (idx, arg) in func.get_param_iter().enumerate() {
            arg.set_name(self.params[idx].name.as_str());
        }

        Ok(Value::new_function(Function::new(func)))
    }

    fn implement(
        &self,
        gen: std::sync::Arc<std::sync::RwLock<super::Generator<'gen>>>,
    ) -> super::error::Result<Self::Out> {
        Ok(Value::new_type(BackendType::new_type()))
    }
}
