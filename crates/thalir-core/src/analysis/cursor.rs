use crate::{
    block::{BasicBlock, BlockId},
    function::Function,
    instructions::Instruction,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorPosition {
    Nowhere,
    At(BlockId, usize),
    Before(BlockId),
    After(BlockId),
}

pub trait IRCursor {
    fn position(&self) -> CursorPosition;

    fn set_position(&mut self, pos: CursorPosition);

    fn current_block(&self) -> Option<BlockId>;

    fn current_inst(&self) -> Option<&Instruction>;

    fn goto_block(&mut self, block: BlockId);

    fn goto_first_inst(&mut self, block: BlockId);

    fn goto_last_inst(&mut self, block: BlockId);

    fn next_inst(&mut self) -> Option<&Instruction>;

    fn prev_inst(&mut self) -> Option<&Instruction>;

    fn next_block(&mut self) -> Option<BlockId>;

    fn prev_block(&mut self) -> Option<BlockId>;
}

pub struct ScannerCursor<'a> {
    function: &'a Function,
    position: CursorPosition,
    block_order: Vec<BlockId>,
    block_idx: Option<usize>,
    cache: HashMap<String, Box<dyn std::any::Any>>,
}

impl<'a> ScannerCursor<'a> {
    pub fn new(function: &'a Function) -> Self {
        let block_order: Vec<BlockId> = function.body.blocks.keys().cloned().collect();

        Self {
            function,
            position: CursorPosition::Nowhere,
            block_order,
            block_idx: None,
            cache: HashMap::new(),
        }
    }

    pub fn at_entry(function: &'a Function) -> Self {
        let mut cursor = Self::new(function);
        cursor.goto_block(function.body.entry_block);
        cursor
    }

    pub fn current_block_ref(&self) -> Option<&BasicBlock> {
        match self.position {
            CursorPosition::At(block_id, _)
            | CursorPosition::Before(block_id)
            | CursorPosition::After(block_id) => self.function.body.blocks.get(&block_id),
            CursorPosition::Nowhere => None,
        }
    }

    pub fn analyze_at<T>(&self, f: impl FnOnce(&Instruction) -> T) -> Option<T> {
        self.current_inst().map(f)
    }

    pub fn is_at_terminator(&self) -> bool {
        if let Some(block) = self.current_block_ref() {
            if let CursorPosition::At(_, idx) = self.position {
                return idx == block.instructions.len();
            }
        }
        false
    }

    pub fn traverse_dom_order(&mut self) -> DomTreeIterator<'a, '_> {
        DomTreeIterator::new(self)
    }

    pub fn cache_analysis<T: 'static>(&mut self, key: String, value: T) {
        self.cache.insert(key, Box::new(value));
    }

    pub fn get_cached<T: 'static>(&self, key: &str) -> Option<&T> {
        self.cache.get(key)?.downcast_ref()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl<'a> IRCursor for ScannerCursor<'a> {
    fn position(&self) -> CursorPosition {
        self.position
    }

    fn set_position(&mut self, pos: CursorPosition) {
        self.position = pos;

        match pos {
            CursorPosition::At(block_id, _)
            | CursorPosition::Before(block_id)
            | CursorPosition::After(block_id) => {
                self.block_idx = self.block_order.iter().position(|&b| b == block_id);
            }
            CursorPosition::Nowhere => {
                self.block_idx = None;
            }
        }
    }

    fn current_block(&self) -> Option<BlockId> {
        match self.position {
            CursorPosition::At(block, _)
            | CursorPosition::Before(block)
            | CursorPosition::After(block) => Some(block),
            CursorPosition::Nowhere => None,
        }
    }

    fn current_inst(&self) -> Option<&Instruction> {
        match self.position {
            CursorPosition::At(block_id, idx) => self
                .function
                .body
                .blocks
                .get(&block_id)
                .and_then(|block| block.instructions.get(idx)),
            _ => None,
        }
    }

    fn goto_block(&mut self, block: BlockId) {
        self.set_position(CursorPosition::Before(block));
    }

    fn goto_first_inst(&mut self, block: BlockId) {
        if let Some(block_ref) = self.function.body.blocks.get(&block) {
            if !block_ref.instructions.is_empty() {
                self.set_position(CursorPosition::At(block, 0));
            } else {
                self.set_position(CursorPosition::After(block));
            }
        }
    }

    fn goto_last_inst(&mut self, block: BlockId) {
        if let Some(block_ref) = self.function.body.blocks.get(&block) {
            let len = block_ref.instructions.len();
            if len > 0 {
                self.set_position(CursorPosition::At(block, len - 1));
            } else {
                self.set_position(CursorPosition::After(block));
            }
        }
    }

    fn next_inst(&mut self) -> Option<&Instruction> {
        match self.position {
            CursorPosition::Before(block) => {
                self.goto_first_inst(block);
                self.current_inst()
            }
            CursorPosition::At(block_id, idx) => {
                if let Some(block) = self.function.body.blocks.get(&block_id) {
                    let next_idx = idx + 1;
                    if next_idx < block.instructions.len() {
                        self.set_position(CursorPosition::At(block_id, next_idx));
                        return Some(&block.instructions[next_idx]);
                    } else {
                        self.set_position(CursorPosition::After(block_id));
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn prev_inst(&mut self) -> Option<&Instruction> {
        match self.position {
            CursorPosition::After(block) => {
                self.goto_last_inst(block);
                self.current_inst()
            }
            CursorPosition::At(block_id, idx) => {
                if idx > 0 {
                    let prev_idx = idx - 1;
                    self.set_position(CursorPosition::At(block_id, prev_idx));
                    if let Some(block) = self.function.body.blocks.get(&block_id) {
                        return Some(&block.instructions[prev_idx]);
                    }
                } else {
                    self.set_position(CursorPosition::Before(block_id));
                }
                None
            }
            _ => None,
        }
    }

    fn next_block(&mut self) -> Option<BlockId> {
        if let Some(idx) = self.block_idx {
            let next_idx = idx + 1;
            if next_idx < self.block_order.len() {
                let next_block = self.block_order[next_idx];
                self.goto_block(next_block);
                return Some(next_block);
            }
        }
        None
    }

    fn prev_block(&mut self) -> Option<BlockId> {
        if let Some(idx) = self.block_idx {
            if idx > 0 {
                let prev_idx = idx - 1;
                let prev_block = self.block_order[prev_idx];
                self.goto_block(prev_block);
                return Some(prev_block);
            }
        }
        None
    }
}

pub struct DomTreeIterator<'a, 'c> {
    cursor: &'c mut ScannerCursor<'a>,
    visited: Vec<BlockId>,
    stack: Vec<BlockId>,
}

impl<'a, 'c> DomTreeIterator<'a, 'c> {
    fn new(cursor: &'c mut ScannerCursor<'a>) -> Self {
        let entry = cursor.function.body.entry_block;
        Self {
            cursor,
            visited: Vec::new(),
            stack: vec![entry],
        }
    }
}

impl<'a, 'c> Iterator for DomTreeIterator<'a, 'c> {
    type Item = BlockId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(block) = self.stack.pop() {
            if !self.visited.contains(&block) {
                self.visited.push(block);

                if let Some(block_ref) = self.cursor.function.body.blocks.get(&block) {
                    match &block_ref.terminator {
                        crate::block::Terminator::Branch {
                            then_block,
                            else_block,
                            ..
                        } => {
                            self.stack.push(*else_block);
                            self.stack.push(*then_block);
                        }
                        crate::block::Terminator::Jump(target, _) => {
                            self.stack.push(*target);
                        }
                        _ => {}
                    }
                }

                self.cursor.goto_block(block);
                return Some(block);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /*
    Cursor navigation tests should verify:
    - Positioning at block start/end
    - Instruction-level navigation
    - Insertion point management
    - State consistency across moves
    */
    #[ignore]
    fn test_cursor_navigation() {}

    /*
    Cache operation tests should verify:
    - Hit/miss behavior
    - Invalidation on IR changes
    - LRU eviction policy
    - Size limit enforcement
    */
    #[test]
    #[ignore]
    fn test_cache_operations() {}
}
