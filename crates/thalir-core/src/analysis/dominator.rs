use crate::{block::BlockId, function::Function};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct DominatorTree {
    idom: HashMap<BlockId, BlockId>,
    children: HashMap<BlockId, Vec<BlockId>>,
}

impl DominatorTree {
    pub fn build(function: &Function) -> Self {
        let entry = function.entry_block();
        let mut idom = HashMap::new();
        let mut children: HashMap<BlockId, Vec<BlockId>> = HashMap::new();

        let blocks = Self::reverse_postorder(function, entry);

        if blocks.len() <= 1 {
            return Self { idom, children };
        }

        let mut doms: HashMap<BlockId, HashSet<BlockId>> = HashMap::new();

        doms.insert(entry, HashSet::from([entry]));

        for &block in &blocks[1..] {
            doms.insert(block, blocks.iter().copied().collect());
        }

        let mut changed = true;
        while changed {
            changed = false;

            for &block in &blocks[1..] {
                let preds = function.body.blocks[&block].predecessors();

                if preds.is_empty() {
                    continue;
                }

                let mut new_dom = None;
                for pred in preds {
                    if let Some(pred_dom) = doms.get(&pred) {
                        if let Some(acc) = new_dom {
                            new_dom = Some(Self::intersect(&acc, pred_dom));
                        } else {
                            new_dom = Some(pred_dom.clone());
                        }
                    }
                }

                if let Some(mut new_dom_set) = new_dom {
                    new_dom_set.insert(block);

                    if doms[&block] != new_dom_set {
                        doms.insert(block, new_dom_set);
                        changed = true;
                    }
                }
            }
        }

        for &block in &blocks {
            if block == entry {
                continue;
            }

            let dominators = &doms[&block];

            for &candidate in dominators {
                if candidate == block {
                    continue;
                }

                let mut is_immediate = true;
                for &other in dominators {
                    if other == block || other == candidate {
                        continue;
                    }

                    if doms
                        .get(&candidate)
                        .map_or(false, |c_doms| c_doms.contains(&other))
                    {
                        is_immediate = false;
                        break;
                    }
                }

                if is_immediate {
                    idom.insert(block, candidate);
                    children.entry(candidate).or_default().push(block);
                    break;
                }
            }
        }

        Self { idom, children }
    }

    fn reverse_postorder(function: &Function, entry: BlockId) -> Vec<BlockId> {
        let mut visited = HashSet::new();
        let mut postorder = Vec::new();

        Self::dfs_postorder(function, entry, &mut visited, &mut postorder);

        postorder.reverse();
        postorder
    }

    fn dfs_postorder(
        function: &Function,
        block: BlockId,
        visited: &mut HashSet<BlockId>,
        postorder: &mut Vec<BlockId>,
    ) {
        if !visited.insert(block) {
            return;
        }

        if let Some(block_data) = function.body.blocks.get(&block) {
            for succ in block_data.successors() {
                Self::dfs_postorder(function, succ, visited, postorder);
            }
        }

        postorder.push(block);
    }

    fn intersect(a: &HashSet<BlockId>, b: &HashSet<BlockId>) -> HashSet<BlockId> {
        a.intersection(b).copied().collect()
    }

    pub fn dominates(&self, dominator: BlockId, dominated: BlockId) -> bool {
        if dominator == dominated {
            return true;
        }

        let mut current = dominated;
        while let Some(&idom) = self.idom.get(&current) {
            if idom == dominator {
                return true;
            }
            if idom == current {
                break;
            }
            current = idom;
        }

        false
    }

    pub fn idom(&self, block: BlockId) -> Option<BlockId> {
        self.idom.get(&block).copied()
    }

    pub fn children(&self, block: BlockId) -> &[BlockId] {
        self.children
            .get(&block)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn dominance_frontier(&self, block: BlockId, function: &Function) -> HashSet<BlockId> {
        let mut frontier = HashSet::new();

        let mut worklist = vec![block];
        let mut visited = HashSet::new();

        while let Some(current) = worklist.pop() {
            if !visited.insert(current) {
                continue;
            }

            if let Some(block_data) = function.body.blocks.get(&current) {
                for succ in block_data.successors() {
                    if !self.dominates(block, succ) || succ == block {
                        frontier.insert(succ);
                    }
                }
            }

            worklist.extend_from_slice(self.children(current));
        }

        frontier
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::IRBuilder;

    #[test]
    fn test_simple_dominance() {
        let mut builder = IRBuilder::new();
        let mut contract_builder = builder.contract("TestContract");

        let mut func_builder = contract_builder.function("test");

        let entry = {
            let entry_builder = func_builder.entry_block();
            entry_builder.block_id()
        };

        let b1 = func_builder.create_block_id();
        let b2 = func_builder.create_block_id();
        let end = func_builder.create_block_id();

        let mut entry_builder = func_builder.switch_to_block(entry).unwrap();
        let cond = entry_builder.constant_bool(true);
        entry_builder.branch(cond, b1, b2).unwrap();

        let mut b1_builder = func_builder.switch_to_block(b1).unwrap();
        b1_builder.jump(end).unwrap();

        let mut b2_builder = func_builder.switch_to_block(b2).unwrap();
        b2_builder.jump(end).unwrap();

        let mut end_builder = func_builder.switch_to_block(end).unwrap();
        end_builder.return_void().unwrap();

        let function = func_builder.build().unwrap();

        let dom_tree = DominatorTree::build(&function);

        assert!(dom_tree.dominates(entry, entry));
        assert!(dom_tree.dominates(entry, b1));
        assert!(dom_tree.dominates(entry, b2));
        assert!(dom_tree.dominates(entry, end));

        assert!(!dom_tree.dominates(b1, b2));
        assert!(!dom_tree.dominates(b2, b1));
        assert!(!dom_tree.dominates(b1, end));
        assert!(!dom_tree.dominates(b2, end));

        assert_eq!(dom_tree.idom(b1), Some(entry));
        assert_eq!(dom_tree.idom(b2), Some(entry));
        assert_eq!(dom_tree.idom(end), Some(entry));
    }
}
