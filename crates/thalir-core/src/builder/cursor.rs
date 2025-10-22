use super::{BlockBuilder, IRContext};
use crate::{
    block::{BasicBlock, BlockId, Terminator},
    instructions::Instruction,
    IrError, Result,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum CursorPosition {
    None,
    BlockStart(BlockId),
    BlockEnd(BlockId),
    After(BlockId, usize),
    Before(BlockId, usize),
}

pub struct FunctionCursor<'a> {
    position: CursorPosition,
    context: &'a mut IRContext,
    blocks: &'a mut HashMap<BlockId, BasicBlock>,
}

impl<'a> FunctionCursor<'a> {
    pub fn new(context: &'a mut IRContext, blocks: &'a mut HashMap<BlockId, BasicBlock>) -> Self {
        Self {
            position: CursorPosition::None,
            context,
            blocks,
        }
    }

    pub fn goto_block_start(&mut self, block: BlockId) {
        self.position = CursorPosition::BlockStart(block);
        self.context.set_current_block(block);
    }

    pub fn goto_block_end(&mut self, block: BlockId) {
        self.position = CursorPosition::BlockEnd(block);
        self.context.set_current_block(block);
    }

    pub fn goto_after_inst(&mut self, block: BlockId, inst_index: usize) {
        self.position = CursorPosition::After(block, inst_index);
        self.context.set_current_block(block);
    }

    pub fn goto_before_inst(&mut self, block: BlockId, inst_index: usize) {
        self.position = CursorPosition::Before(block, inst_index);
        self.context.set_current_block(block);
    }

    pub fn current_block(&self) -> Option<BlockId> {
        match self.position {
            CursorPosition::None => None,
            CursorPosition::BlockStart(b)
            | CursorPosition::BlockEnd(b)
            | CursorPosition::After(b, _)
            | CursorPosition::Before(b, _) => Some(b),
        }
    }

    pub fn insert_inst(&mut self, inst: Instruction) -> Result<()> {
        let block_id = self
            .current_block()
            .ok_or_else(|| IrError::BuilderError("Cursor not positioned".to_string()))?;

        let block = self
            .blocks
            .get_mut(&block_id)
            .ok_or_else(|| IrError::BuilderError(format!("Block {:?} not found", block_id)))?;

        match self.position {
            CursorPosition::BlockStart(_) => {
                block.instructions.insert(0, inst);
                self.position = CursorPosition::After(block_id, 0);
            }
            CursorPosition::BlockEnd(_) => {
                let index = block.instructions.len();
                block.instructions.push(inst);
                self.position = CursorPosition::After(block_id, index);
            }
            CursorPosition::After(_, index) => {
                let insert_at = index + 1;
                if insert_at <= block.instructions.len() {
                    block.instructions.insert(insert_at, inst);
                    self.position = CursorPosition::After(block_id, insert_at);
                } else {
                    block.instructions.push(inst);
                    self.position = CursorPosition::After(block_id, block.instructions.len() - 1);
                }
            }
            CursorPosition::Before(_, index) => {
                block.instructions.insert(index, inst);
                self.position = CursorPosition::After(block_id, index);
            }
            CursorPosition::None => {
                return Err(IrError::BuilderError("Cursor not positioned".to_string()));
            }
        }

        Ok(())
    }

    pub fn set_terminator(&mut self, term: Terminator) -> Result<()> {
        let block_id = self
            .current_block()
            .ok_or_else(|| IrError::BuilderError("Cursor not positioned".to_string()))?;

        let block = self
            .blocks
            .get_mut(&block_id)
            .ok_or_else(|| IrError::BuilderError(format!("Block {:?} not found", block_id)))?;

        block.terminator = term;
        Ok(())
    }

    pub fn create_block(&mut self, _name: String) -> BlockId {
        let block_id = BlockId(self.context.next_id() as u32);
        let block = BasicBlock::new(block_id);

        self.blocks.insert(block_id, block);
        self.goto_block_start(block_id);

        block_id
    }

    pub fn split_block(&mut self, _new_block_name: String) -> Result<BlockId> {
        let (block_id, split_at) = match self.position {
            CursorPosition::After(b, i) => (b, i + 1),
            CursorPosition::Before(b, i) => (b, i),
            _ => {
                return Err(IrError::BuilderError(
                    "Can only split block when cursor is at an instruction".to_string(),
                ))
            }
        };

        let new_block_id = BlockId(self.context.next_id() as u32);
        let mut new_block = BasicBlock::new(new_block_id);

        let old_block = self
            .blocks
            .get_mut(&block_id)
            .ok_or_else(|| IrError::BuilderError(format!("Block {:?} not found", block_id)))?;

        if split_at < old_block.instructions.len() {
            new_block.instructions = old_block.instructions.split_off(split_at);
        }

        new_block.terminator = old_block.terminator.clone();
        old_block.terminator = Terminator::Jump(new_block_id, Vec::new());

        self.blocks.insert(new_block_id, new_block);

        self.goto_block_start(new_block_id);

        Ok(new_block_id)
    }

    pub fn is_terminated(&self) -> Result<bool> {
        let block_id = self
            .current_block()
            .ok_or_else(|| IrError::BuilderError("Cursor not positioned".to_string()))?;

        let block = self
            .blocks
            .get(&block_id)
            .ok_or_else(|| IrError::BuilderError(format!("Block {:?} not found", block_id)))?;

        Ok(!matches!(block.terminator, Terminator::Invalid))
    }

    pub fn next_inst_index(&self) -> Option<usize> {
        match self.position {
            CursorPosition::BlockStart(_) => Some(0),
            CursorPosition::After(_, i) => Some(i + 1),
            CursorPosition::Before(_, i) => Some(i),
            _ => None,
        }
    }
}

impl<'a> BlockBuilder<'a> {
    pub fn cursor_at_end(&mut self) -> CursorPosition {
        CursorPosition::BlockEnd(self.block_id)
    }

    pub fn cursor_at_start(&mut self) -> CursorPosition {
        CursorPosition::BlockStart(self.block_id)
    }
}
