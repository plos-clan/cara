use super::*;

impl<'gen> GenerateProgramOnce<'gen> for Statement {
    type Out = ();

    fn generate(&self, gen: Arc<RwLock<Generator<'gen>>>) -> Result<Self::Out> {
        match self {
            Self::Return(ret) => ret.generate(gen),
        }
    }
}

impl<'gen> GenerateProgramOnce<'gen> for Return {
    type Out = ();

    fn generate(&self, gen: Arc<RwLock<Generator<'gen>>>) -> Result<Self::Out> {
        if let Some(value_exp) = &self.value {
            let value = value_exp.generate(gen.clone())?;

            gen.read()
                .unwrap()
                .builder
                .build_return(Some(&value.as_basic_value_enum(
                    value_exp.get_span(),
                    gen.read().unwrap().context,
                )?))
                .unwrap();
        } else {
            gen.read().unwrap().builder.build_return(None).unwrap();
        }
        Ok(())
    }
}
