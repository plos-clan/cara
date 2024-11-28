use inkwell::{llvm_sys::core::LLVMSetValueName2, values::AsValueRef};

use super::*;

impl<'gen> GenerateProgramTwice<'gen> for ConstDecl {
    type Out = ();

    fn decl(
        &self,
        gen: std::sync::Arc<std::sync::RwLock<super::Generator<'gen>>>,
    ) -> Result<Self::Out> {
        gen.write().unwrap().current_const_name = Some(self.name.clone());
        let value = self.initial_value.decl(gen.clone())?;
        gen.write().unwrap().current_const_name = None;

        if let Ok(value) = value.as_function(self.span.clone()) {
            unsafe {
                LLVMSetValueName2(
                    value.as_value_ref(),
                    self.name.as_ptr() as *const i8,
                    self.name.len(),
                );
            }
        }

        gen.write()
            .unwrap()
            .global
            .push(Symbol::Const(self.name.clone(), value));

        Ok(())
    }

    fn implement(
        &self,
        gen: std::sync::Arc<std::sync::RwLock<super::Generator<'gen>>>,
    ) -> Result<Self::Out> {
        gen.write().unwrap().current_const_name = Some(self.name.clone());
        let value = self.initial_value.implement(gen.clone())?;
        gen.write().unwrap().current_const_name = None;

        if let Err(_) = value.as_function(self.span.clone()) {
            gen.write()
                .unwrap()
                .local
                .push(Symbol::Const(self.name.clone(), value));
        }

        Ok(())
    }
}

impl<'gen> GenerateProgramTwice<'gen> for ConstInitialValue {
    type Out = Value<'gen>;

    fn decl(
        &self,
        gen: std::sync::Arc<std::sync::RwLock<super::Generator<'gen>>>,
    ) -> Result<Self::Out> {
        match &self.value {
            ConstInitialValueEnum::Function(func) => func.decl(gen),
            ConstInitialValueEnum::Exp(exp) => exp.generate(gen),
        }
    }

    fn implement(
        &self,
        gen: std::sync::Arc<std::sync::RwLock<super::Generator<'gen>>>,
    ) -> Result<Self::Out> {
        match &self.value {
            ConstInitialValueEnum::Function(func) => func.implement(gen),
            ConstInitialValueEnum::Exp(exp) => exp.generate(gen),
        }
    }
}

impl<'gen> GenerateProgramTwice<'gen> for FunctionDef {
    type Out = Value<'gen>;

    fn decl(
        &self,
        gen: std::sync::Arc<std::sync::RwLock<super::Generator<'gen>>>,
    ) -> Result<Self::Out> {
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
    ) -> Result<Self::Out> {
        let func_name = gen.read().unwrap().current_const_name.clone().unwrap();
        let binding = gen.read().unwrap();
        let func = match binding.global.get(&func_name).unwrap() {
            Symbol::Const(_, value) => value.as_function(self.span.clone()).unwrap(),
            _ => unreachable!(),
        }.clone();
        
        //let func = &func;

        let entry = gen.read().unwrap().context.append_basic_block(*func, "entry");

        gen.read().unwrap().builder.position_at_end(entry);
        
        //let mut paramters = Vec::new();
        
        drop(binding);

        for (idx, arg) in self.params.iter().enumerate() {
            let func_name = gen.read().unwrap().current_const_name.clone().unwrap();
            let binding = gen.read().unwrap();
            let func = match binding.global.get(&func_name).unwrap() {
                Symbol::Const(_, value) => value.as_function(self.span.clone()).unwrap(),
                _ => unreachable!(),
            }.clone();
            
            let ty = arg.param_type.generate(gen.clone())?.as_type(self.span.clone())?;

            let ptr = gen.read().unwrap().create_entry_block_alloca(
                arg.name.as_str(),
                ty.as_basic_type_enum(self.span.clone())?,
                self.span.clone(),
            ).clone();

            gen.read().unwrap()
                .builder
                .build_store(ptr, func.get_nth_param(idx as u32).unwrap())
                .unwrap();
            
            gen.write().unwrap()
                .local
                .push(Symbol::Varible(arg.name.clone(), ptr.clone(), ty.clone()));
            
            //paramters.push((arg.name.clone(), ptr.clone(), ty.clone()));
        }

        self.block.generate(gen.clone())?;
        
        let func_name = gen.read().unwrap().current_const_name.clone().unwrap();
        let binding = gen.read().unwrap();
        let func = match binding.global.get(&func_name).unwrap() {
            Symbol::Const(_, value) => value.as_function(self.span.clone()).unwrap(),
            _ => unimplemented!(),
        }.clone();

        for bb in func.get_basic_block_iter() {
            if let Some(ins) = bb.get_last_instruction() {
                if !ins.is_terminator() {
                    gen.read().unwrap().builder.position_at_end(bb.clone());
                    gen.read().unwrap().builder.build_return(None).unwrap();
                }
            } else {
                gen.read().unwrap().builder.position_at_end(bb.clone());
                gen.read().unwrap().builder.build_return(None).unwrap();
            }
        }

        for _ in 0..self.params.len() {
            gen.write().unwrap().local.pop().unwrap();
        }

        Ok(Value::new_void())
    }
}
