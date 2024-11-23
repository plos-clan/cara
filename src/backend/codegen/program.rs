use super::*;

impl<'gen> GenerateProgramOnce<'gen> for Type {
    type Out = Value<'gen>;

    fn generate(
        &self,
        gen: Arc<RwLock<Generator<'gen>>>,
    ) -> Result<Self::Out> {
        let ty = match &self.ty {
            TypeEnum::U64 => BackendType::new_int(gen.read().unwrap().context.i64_type()),
        };
        Ok(Value::new_type(ty))
    }
}

impl<'gen> GenerateProgramTwice<'gen> for CompUnit {
    type Out = ();

    fn decl(
        &self,
        gen: Arc<RwLock<Generator<'gen>>>,
    ) -> Result<Self::Out> {
        for global_item in self.global_items.iter() {
            global_item.decl(gen.clone())?;
        }

        Ok(())
    }

    fn implement(
        &self,
        gen: Arc<RwLock<Generator<'gen>>>,
    ) -> Result<Self::Out> {
        for global_item in self.global_items.iter() {
            if let Err(error) = global_item.implement(gen.clone()) {
                gen.write().unwrap().errors.push(error);
            }
        }

        Ok(())
    }
}

impl<'gen> GenerateProgramTwice<'gen> for GlobalItem {
    type Out = ();

    fn decl(
        &self,
        gen: Arc<RwLock<Generator<'gen>>>,
    ) -> Result<Self::Out> {
        match self {
            GlobalItem::ConstDecl(decl) => decl.decl(gen),
        }
    }

    fn implement(
        &self,
        gen: Arc<RwLock<Generator<'gen>>>,
    ) -> Result<Self::Out> {
        match self {
            GlobalItem::ConstDecl(decl) => decl.implement(gen),
        }
    }
}

impl<'gen> GenerateProgramOnce<'gen> for Block {
    type Out = ();

    fn generate(
        &self,
        gen: Arc<RwLock<Generator<'gen>>>,
    ) -> Result<Self::Out> {
        /*let stack_len = gen.read().unwrap().local.len();

        for item in self.items.iter() {
            if let Err(err) = item.generate(gen.clone()) {
                if !(err.0 == ErrorTypes::Terminated) {
                    gen.write().unwrap().errors.push(err);
                    continue;
                }
                break;
            }
        }

        let mut gen = gen.write().unwrap();
        while gen.local.len() > stack_len {
            gen.local.pop();
        }*/

        Ok(())
    }
}

/*impl<'gen> GenerateProgramOnce<'gen> for BlockItem {
    type Out = ();

    fn generate(
        &self,
        _gen: Arc<RwLock<Generator<'gen>>>,
    ) -> Result<Self::Out> {
        Ok(())
    }
}*/
