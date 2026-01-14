use ast::{Block, BlockItem, Statement, VarDef};

use crate::SimplifierContext;

impl SimplifierContext<'_> {
    pub fn simp_block(&mut self, block: Block) -> Block {
        self.locals.push_scope();
        let items = block
            .items
            .into_iter()
            .map(|stmt| self.simp_block_item(stmt))
            .collect();
        let return_value = block.return_value.map(|ret| self.simp_exp(ret));
        self.locals.pop_scope();
        Block {
            items,
            return_value,
            span: block.span,
        }
    }

    fn simp_block_item(&mut self, item: BlockItem) -> BlockItem {
        match item {
            BlockItem::VarDef(var_def) => {
                let VarDef {
                    name,
                    var_type,
                    initial_value,
                    mutable,
                    span,
                } = var_def;
                let var_type = var_type.map(|ty| self.simp_exp(ty));
                let initial_value = self.simp_exp(initial_value);

                self.locals.push(name.clone());
                BlockItem::VarDef(VarDef {
                    name,
                    var_type,
                    initial_value,
                    mutable,
                    span,
                })
            }
            BlockItem::Statement(stmt) => BlockItem::Statement(match stmt {
                Statement::Exp(exp) => Statement::Exp(self.simp_exp(exp)),
                Statement::InlineAsm(asm) => Statement::InlineAsm(asm),
            }),
        }
    }
}
