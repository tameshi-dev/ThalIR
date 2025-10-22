use crate::block::{BasicBlock, BlockId};
use crate::function::FunctionBody;
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    pub blocks: IndexMap<BlockId, BasicBlock>,
    pub edges: HashMap<BlockId, Vec<BlockId>>,
    pub reverse_edges: HashMap<BlockId, Vec<BlockId>>,
    pub entry: BlockId,
}

impl ControlFlowGraph {
    pub fn from_function(body: &FunctionBody) -> Self {
        let mut edges = HashMap::new();
        let mut reverse_edges = HashMap::new();

        for (block_id, block) in &body.blocks {
            let successors = block.terminator.successors();
            edges.insert(*block_id, successors.clone());

            for succ in successors {
                reverse_edges
                    .entry(succ)
                    .or_insert_with(Vec::new)
                    .push(*block_id);
            }
        }

        Self {
            blocks: body.blocks.clone(),
            edges,
            reverse_edges,
            entry: body.entry_block,
        }
    }

    pub fn predecessors(&self, block: BlockId) -> &[BlockId] {
        self.reverse_edges
            .get(&block)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn successors(&self, block: BlockId) -> &[BlockId] {
        self.edges.get(&block).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn is_reachable(&self, block: BlockId) -> bool {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(self.entry);

        while let Some(current) = queue.pop_front() {
            if current == block {
                return true;
            }

            if visited.insert(current) {
                for &succ in self.successors(current) {
                    queue.push_back(succ);
                }
            }
        }

        false
    }

    pub fn reachable_blocks(&self) -> HashSet<BlockId> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(self.entry);

        while let Some(current) = queue.pop_front() {
            if visited.insert(current) {
                for &succ in self.successors(current) {
                    queue.push_back(succ);
                }
            }
        }

        visited
    }
}

#[derive(Debug, Clone)]
pub struct DominatorTree {
    dominators: HashMap<BlockId, HashSet<BlockId>>,
    immediate_dominators: HashMap<BlockId, BlockId>,
}

impl DominatorTree {
    pub fn from_cfg(cfg: &ControlFlowGraph) -> Self {
        let mut dominators = HashMap::new();
        let reachable = cfg.reachable_blocks();

        dominators.insert(cfg.entry, HashSet::from([cfg.entry]));
        for &block in &reachable {
            if block != cfg.entry {
                dominators.insert(block, reachable.clone());
            }
        }

        let mut changed = true;
        while changed {
            changed = false;

            for &block in &reachable {
                if block == cfg.entry {
                    continue;
                }

                let mut new_doms = HashSet::from([block]);

                let preds = cfg.predecessors(block);
                if !preds.is_empty() {
                    let mut intersection = dominators.get(&preds[0]).cloned().unwrap_or_default();

                    for &pred in &preds[1..] {
                        if let Some(pred_doms) = dominators.get(&pred) {
                            intersection = intersection.intersection(pred_doms).cloned().collect();
                        }
                    }

                    new_doms.extend(intersection);
                }

                if dominators.get(&block) != Some(&new_doms) {
                    dominators.insert(block, new_doms);
                    changed = true;
                }
            }
        }

        let immediate_dominators = Self::compute_immediate_dominators(&dominators, cfg.entry);

        Self {
            dominators,
            immediate_dominators,
        }
    }

    fn compute_immediate_dominators(
        dominators: &HashMap<BlockId, HashSet<BlockId>>,
        entry: BlockId,
    ) -> HashMap<BlockId, BlockId> {
        let mut idoms = HashMap::new();

        for (&block, doms) in dominators {
            if block == entry {
                continue;
            }

            let mut candidates: Vec<_> = doms.iter().filter(|&&d| d != block).cloned().collect();

            candidates.sort_by_key(|&d| dominators.get(&d).map(|s| s.len()).unwrap_or(0));

            if let Some(&idom) = candidates.last() {
                idoms.insert(block, idom);
            }
        }

        idoms
    }

    pub fn dominates(&self, a: BlockId, b: BlockId) -> bool {
        self.dominators
            .get(&b)
            .map(|doms| doms.contains(&a))
            .unwrap_or(false)
    }

    pub fn immediate_dominator(&self, block: BlockId) -> Option<BlockId> {
        self.immediate_dominators.get(&block).cloned()
    }
}

#[derive(Debug, Clone)]
pub struct LoopAnalysis {
    pub loops: Vec<Loop>,
    pub loop_headers: HashSet<BlockId>,
    pub loop_depth: HashMap<BlockId, usize>,
}

#[derive(Debug, Clone)]
pub struct Loop {
    pub header: BlockId,
    pub blocks: HashSet<BlockId>,
    pub back_edges: Vec<(BlockId, BlockId)>,
    pub exits: HashSet<BlockId>,
    pub depth: usize,
}

impl LoopAnalysis {
    pub fn from_cfg(cfg: &ControlFlowGraph, dom_tree: &DominatorTree) -> Self {
        let mut loops = Vec::new();
        let mut loop_headers = HashSet::new();
        let mut back_edges = Vec::new();

        for (&block, successors) in &cfg.edges {
            for &succ in successors {
                if dom_tree.dominates(succ, block) {
                    back_edges.push((block, succ));
                    loop_headers.insert(succ);
                }
            }
        }

        for (tail, header) in back_edges {
            let mut loop_blocks = HashSet::from([header]);
            let mut queue = VecDeque::from([tail]);

            while let Some(block) = queue.pop_front() {
                if loop_blocks.insert(block) {
                    for &pred in cfg.predecessors(block) {
                        queue.push_back(pred);
                    }
                }
            }

            let mut exits = HashSet::new();
            for &block in &loop_blocks {
                for &succ in cfg.successors(block) {
                    if !loop_blocks.contains(&succ) {
                        exits.insert(block);
                    }
                }
            }

            loops.push(Loop {
                header,
                blocks: loop_blocks,
                back_edges: vec![(tail, header)],
                exits,
                depth: 0,
            });
        }

        let loop_depth = Self::compute_loop_depth(&loops);

        Self {
            loops,
            loop_headers,
            loop_depth,
        }
    }

    fn compute_loop_depth(loops: &[Loop]) -> HashMap<BlockId, usize> {
        let mut depth_map = HashMap::new();

        for loop_info in loops {
            for &block in &loop_info.blocks {
                *depth_map.entry(block).or_insert(0) += 1;
            }
        }

        depth_map
    }

    pub fn is_in_loop(&self, block: BlockId) -> bool {
        self.loop_depth.contains_key(&block)
    }

    pub fn get_loop_depth(&self, block: BlockId) -> usize {
        self.loop_depth.get(&block).cloned().unwrap_or(0)
    }
}
