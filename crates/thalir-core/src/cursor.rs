use crate::{
    block::{BasicBlock, BlockId, Terminator},
    function::Function,
    inst_builder::InstBuilder,
    instructions::Instruction,
    IrError, Result,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorPosition {
    Nowhere,
    Before(BlockId),
    After(BlockId),
    At(BlockId, usize),
}

impl Default for CursorPosition {
    fn default() -> Self {
        CursorPosition::Nowhere
    }
}

pub struct FuncCursor<'a> {
    position: CursorPosition,
    function: &'a mut Function,
}

impl<'a> FuncCursor<'a> {
    pub fn new(function: &'a mut Function) -> Self {
        Self {
            position: CursorPosition::Nowhere,
            function,
        }
    }

    pub fn position(&self) -> CursorPosition {
        self.position
    }

    pub fn set_position(&mut self, pos: CursorPosition) {
        self.position = pos;
    }

    pub fn goto_top(&mut self, block: BlockId) {
        self.position = CursorPosition::Before(block);
    }

    pub fn goto_bottom(&mut self, block: BlockId) {
        self.position = CursorPosition::After(block);
    }

    pub fn goto_inst(&mut self, block: BlockId, index: usize) {
        self.position = CursorPosition::At(block, index);
    }

    pub fn next_inst(&mut self) -> Option<()> {
        match self.position {
            CursorPosition::Nowhere => None,
            CursorPosition::Before(block) => {
                if self.get_block(block)?.instructions.is_empty() {
                    self.position = CursorPosition::After(block);
                } else {
                    self.position = CursorPosition::At(block, 0);
                }
                Some(())
            }
            CursorPosition::At(block, idx) => {
                let block_data = self.get_block(block)?;
                if idx + 1 < block_data.instructions.len() {
                    self.position = CursorPosition::At(block, idx + 1);
                } else {
                    self.position = CursorPosition::After(block);
                }
                Some(())
            }
            CursorPosition::After(_) => None,
        }
    }

    pub fn prev_inst(&mut self) -> Option<()> {
        match self.position {
            CursorPosition::Nowhere => None,
            CursorPosition::Before(_) => None,
            CursorPosition::At(block, 0) => {
                self.position = CursorPosition::Before(block);
                Some(())
            }
            CursorPosition::At(block, idx) => {
                self.position = CursorPosition::At(block, idx - 1);
                Some(())
            }
            CursorPosition::After(block) => {
                let block_data = self.get_block(block)?;
                if block_data.instructions.is_empty() {
                    self.position = CursorPosition::Before(block);
                } else {
                    self.position = CursorPosition::At(block, block_data.instructions.len() - 1);
                }
                Some(())
            }
        }
    }

    pub fn insert_inst(&mut self, inst: Instruction) -> Result<()> {
        match self.position {
            CursorPosition::Nowhere => {
                return Err(IrError::BuilderError(
                    "Cannot insert at Nowhere position".into(),
                ));
            }
            CursorPosition::Before(block) => {
                let block_data = self
                    .get_block_mut(block)
                    .ok_or_else(|| IrError::BuilderError(format!("Block {:?} not found", block)))?;
                block_data.instructions.insert(0, inst);

                self.position = CursorPosition::At(block, 0);
            }
            CursorPosition::At(block, idx) => {
                let block_data = self
                    .get_block_mut(block)
                    .ok_or_else(|| IrError::BuilderError(format!("Block {:?} not found", block)))?;
                block_data.instructions.insert(idx, inst);
            }
            CursorPosition::After(block) => {
                let block_data = self
                    .get_block_mut(block)
                    .ok_or_else(|| IrError::BuilderError(format!("Block {:?} not found", block)))?;
                let new_idx = block_data.instructions.len();
                block_data.instructions.push(inst);

                self.position = CursorPosition::At(block, new_idx);
            }
        }
        Ok(())
    }

    pub fn set_terminator(&mut self, term: Terminator) -> Result<()> {
        let block_id = match self.position {
            CursorPosition::Before(b) | CursorPosition::At(b, _) | CursorPosition::After(b) => b,
            CursorPosition::Nowhere => {
                return Err(IrError::BuilderError("No current block".into()));
            }
        };

        let block = self
            .get_block_mut(block_id)
            .ok_or_else(|| IrError::BuilderError(format!("Block {:?} not found", block_id)))?;

        if !matches!(block.terminator, Terminator::Invalid) {
            return Err(IrError::BuilderError(format!(
                "Block {:?} already has terminator",
                block_id
            )));
        }

        block.terminator = term;
        Ok(())
    }

    pub fn is_terminated(&self) -> bool {
        let block_id = match self.position {
            CursorPosition::Before(b) | CursorPosition::At(b, _) | CursorPosition::After(b) => b,
            CursorPosition::Nowhere => return false,
        };

        self.get_block(block_id)
            .map(|b| b.is_terminated())
            .unwrap_or(false)
    }

    fn get_block(&self, block_id: BlockId) -> Option<&BasicBlock> {
        self.function.body.blocks.get(&block_id)
    }

    fn get_block_mut(&mut self, block_id: BlockId) -> Option<&mut BasicBlock> {
        self.function.body.blocks.get_mut(&block_id)
    }

    pub fn ins(&mut self) -> InstBuilder<'_, 'a> {
        InstBuilder::new(self)
    }
}

impl<'a> FuncCursor<'a> {
    pub fn at_bottom(mut self, block: BlockId) -> Self {
        self.goto_bottom(block);
        self
    }

    pub fn at_top(mut self, block: BlockId) -> Self {
        self.goto_top(block);
        self
    }
}
