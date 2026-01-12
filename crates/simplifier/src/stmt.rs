use ast::{Block, BlockItem, Statement};

use crate::SimplifierContext;

impl SimplifierContext {
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
                self.locals.push(var_def.name.clone());
                BlockItem::VarDef(var_def)
            }
            BlockItem::Statement(stmt) => BlockItem::Statement(match stmt {
                Statement::Exp(exp) => Statement::Exp(self.simp_exp(exp)),
                Statement::InlineAsm(asm) => Statement::InlineAsm(asm),
            }),
        }
    }
}
