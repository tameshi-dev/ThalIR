use crate::{block::BlockId, function::Function};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    entry: BlockId,
    exits: Vec<BlockId>,
    predecessors: HashMap<BlockId, Vec<BlockId>>,
    successors: HashMap<BlockId, Vec<BlockId>>,
    loops: Vec<Loop>,
    back_edges: Vec<(BlockId, BlockId)>,
}

#[derive(Debug, Clone)]
pub struct Loop {
    pub header: BlockId,
    pub blocks: HashSet<BlockId>,
    pub back_edges: Vec<BlockId>,
    pub exits: Vec<BlockId>,
    pub depth: usize,
}

impl ControlFlowGraph {
    pub fn build(function: &Function) -> Self {
        let entry = function.entry_block();
        let mut predecessors = HashMap::new();
        let mut successors = HashMap::new();
        let mut exits = Vec::new();

        for (&block_id, block) in &function.body.blocks {
            let succs = block.successors();
            successors.insert(block_id, succs.clone());

            if succs.is_empty() {
                exits.push(block_id);
            }

            for succ in succs {
                predecessors
                    .entry(succ)
                    .or_insert_with(Vec::new)
                    .push(block_id);
            }
        }

        let (loops, back_edges) = Self::find_loops(function, entry, &predecessors);

        Self {
            entry,
            exits,
            predecessors,
            successors,
            loops,
            back_edges,
        }
    }

    fn find_loops(
        function: &Function,
        entry: BlockId,
        predecessors: &HashMap<BlockId, Vec<BlockId>>,
    ) -> (Vec<Loop>, Vec<(BlockId, BlockId)>) {
        let mut loops = Vec::new();
        let mut back_edges = Vec::new();

        let mut visited = HashSet::new();
        let mut on_stack = HashSet::new();
        let mut stack = vec![(entry, false)];

        while let Some((block, processed)) = stack.pop() {
            if processed {
                on_stack.remove(&block);
                continue;
            }

            if !visited.insert(block) {
                continue;
            }

            on_stack.insert(block);
            stack.push((block, true));

            if let Some(block_data) = function.body.blocks.get(&block) {
                for succ in block_data.successors() {
                    if on_stack.contains(&succ) {
                        back_edges.push((block, succ));

                        let loop_blocks = Self::find_loop_blocks(succ, block, predecessors);
                        let loop_exits = Self::find_loop_exits(&loop_blocks, &function.body.blocks);

                        loops.push(Loop {
                            header: succ,
                            blocks: loop_blocks,
                            back_edges: vec![block],
                            exits: loop_exits,
                            depth: 0,
                        });
                    } else if !visited.contains(&succ) {
                        stack.push((succ, false));
                    }
                }
            }
        }

        for i in 0..loops.len() {
            let mut depth = 0;
            for j in 0..loops.len() {
                if i != j && loops[j].blocks.contains(&loops[i].header) {
                    depth += 1;
                }
            }
            loops[i].depth = depth;
        }

        (loops, back_edges)
    }

    fn find_loop_blocks(
        header: BlockId,
        back_edge_source: BlockId,
        predecessors: &HashMap<BlockId, Vec<BlockId>>,
    ) -> HashSet<BlockId> {
        let mut blocks = HashSet::new();
        blocks.insert(header);

        if back_edge_source == header {
            return blocks;
        }

        blocks.insert(back_edge_source);

        let mut worklist = vec![back_edge_source];

        while let Some(block) = worklist.pop() {
            if let Some(preds) = predecessors.get(&block) {
                for &pred in preds {
                    if pred != header && blocks.insert(pred) {
                        worklist.push(pred);
                    }
                }
            }
        }

        blocks
    }

    fn find_loop_exits(
        loop_blocks: &HashSet<BlockId>,
        blocks: &indexmap::IndexMap<BlockId, crate::block::BasicBlock>,
    ) -> Vec<BlockId> {
        let mut exits = Vec::new();

        for &block in loop_blocks {
            if let Some(block_data) = blocks.get(&block) {
                for succ in block_data.successors() {
                    if !loop_blocks.contains(&succ) {
                        exits.push(succ);
                    }
                }
            }
        }

        exits.sort_unstable();
        exits.dedup();
        exits
    }

    pub fn entry(&self) -> BlockId {
        self.entry
    }

    pub fn exits(&self) -> &[BlockId] {
        &self.exits
    }

    pub fn predecessors(&self, block: BlockId) -> &[BlockId] {
        self.predecessors
            .get(&block)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn successors(&self, block: BlockId) -> &[BlockId] {
        self.successors
            .get(&block)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn is_back_edge(&self, from: BlockId, to: BlockId) -> bool {
        self.back_edges.contains(&(from, to))
    }

    pub fn loops(&self) -> &[Loop] {
        &self.loops
    }

    pub fn find_loop(&self, block: BlockId) -> Option<&Loop> {
        self.loops.iter().find(|l| l.blocks.contains(&block))
    }

    pub fn is_loop_header(&self, block: BlockId) -> bool {
        self.loops.iter().any(|l| l.header == block)
    }

    pub fn post_dominators(&self, _function: &Function) -> HashMap<BlockId, HashSet<BlockId>> {
        HashMap::new()
    }

    pub fn has_path(&self, from: BlockId, to: BlockId) -> bool {
        if from == to {
            return true;
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(from);

        while let Some(block) = queue.pop_front() {
            if !visited.insert(block) {
                continue;
            }

            for &succ in self.successors(block) {
                if succ == to {
                    return true;
                }
                queue.push_back(succ);
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::IRBuilder;

    #[test]
    fn test_loop_detection() {
        let mut builder = IRBuilder::new();
        let mut contract_builder = builder.contract("TestContract");

        let mut func_builder = contract_builder.function("test");

        let entry = {
            let entry_builder = func_builder.entry_block();
            entry_builder.block_id()
        };

        let loop_header = func_builder.create_block_id();
        let loop_body = func_builder.create_block_id();
        let exit = func_builder.create_block_id();

        let mut entry_builder = func_builder.switch_to_block(entry).unwrap();
        entry_builder.jump(loop_header).unwrap();

        let mut header_builder = func_builder.switch_to_block(loop_header).unwrap();
        let cond = header_builder.constant_bool(true);
        header_builder.branch(cond, loop_body, exit).unwrap();

        let mut body_builder = func_builder.switch_to_block(loop_body).unwrap();
        body_builder.jump(loop_header).unwrap();

        let mut exit_builder = func_builder.switch_to_block(exit).unwrap();
        exit_builder.return_void().unwrap();

        let function = func_builder.build().unwrap();
        let cfg = ControlFlowGraph::build(&function);

        assert_eq!(cfg.loops().len(), 1);

        let loop_info = &cfg.loops()[0];
        assert_eq!(loop_info.header, loop_header);
        assert!(loop_info.blocks.contains(&loop_header));
        assert!(loop_info.blocks.contains(&loop_body));
        assert_eq!(loop_info.exits, vec![exit]);

        assert!(cfg.is_back_edge(loop_body, loop_header));
    }
}
