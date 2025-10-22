use crate::{block::BlockId, function::FunctionBody, instructions::Instruction, values::Value};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct DataFlowAnalysis {
    pub def_use: DefUseChains,
    pub live_vars: LiveVariables,
    pub reaching_defs: ReachingDefinitions,
}

impl DataFlowAnalysis {
    pub fn analyze(body: &FunctionBody) -> Self {
        let def_use = DefUseChains::compute(body);
        let live_vars = LiveVariables::compute(body, &def_use);
        let reaching_defs = ReachingDefinitions::compute(body);

        Self {
            def_use,
            live_vars,
            reaching_defs,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DefUseChains {
    pub definitions: HashMap<Value, Location>,
    pub uses: HashMap<Value, HashSet<Location>>,
    pub defs_at: HashMap<Location, HashSet<Value>>,
    pub uses_at: HashMap<Location, HashSet<Value>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Location {
    pub block: BlockId,
    pub instruction: usize,
}

impl DefUseChains {
    pub fn compute(body: &FunctionBody) -> Self {
        let mut definitions = HashMap::new();
        let mut uses = HashMap::new();
        let mut defs_at = HashMap::new();
        let mut uses_at = HashMap::new();

        for (block_id, block) in &body.blocks {
            for (inst_idx, inst) in block.instructions.iter().enumerate() {
                let loc = Location {
                    block: *block_id,
                    instruction: inst_idx,
                };

                if let Some(result) = inst.result() {
                    definitions.insert(result.clone(), loc.clone());
                    defs_at
                        .entry(loc.clone())
                        .or_insert_with(HashSet::new)
                        .insert(result.clone());
                }

                for used_value in Self::get_used_values(inst) {
                    uses.entry(used_value.clone())
                        .or_insert_with(HashSet::new)
                        .insert(loc.clone());
                    uses_at
                        .entry(loc.clone())
                        .or_insert_with(HashSet::new)
                        .insert(used_value);
                }
            }
        }

        Self {
            definitions,
            uses,
            defs_at,
            uses_at,
        }
    }

    fn get_used_values(inst: &Instruction) -> Vec<Value> {
        let mut values = Vec::new();

        match inst {
            Instruction::Add { left, right, .. }
            | Instruction::Sub { left, right, .. }
            | Instruction::Mul { left, right, .. }
            | Instruction::Div { left, right, .. }
            | Instruction::Mod { left, right, .. } => {
                values.push(left.clone());
                values.push(right.clone());
            }
            Instruction::StorageStore { value, .. } => {
                values.push(value.clone());
            }
            Instruction::Call { args, value, .. } => {
                values.extend(args.clone());
                if let Some(v) = value {
                    values.push(v.clone());
                }
            }

            _ => {}
        }

        values
    }

    pub fn get_uses(&self, value: &Value) -> Option<&HashSet<Location>> {
        self.uses.get(value)
    }

    pub fn get_definition(&self, value: &Value) -> Option<&Location> {
        self.definitions.get(value)
    }
}

#[derive(Debug, Clone)]
pub struct LiveVariables {
    pub live_in: HashMap<BlockId, HashSet<Value>>,
    pub live_out: HashMap<BlockId, HashSet<Value>>,
}

impl LiveVariables {
    pub fn compute(body: &FunctionBody, def_use: &DefUseChains) -> Self {
        let mut live_in = HashMap::new();
        let mut live_out = HashMap::new();

        for block_id in body.blocks.keys() {
            live_in.insert(*block_id, HashSet::new());
            live_out.insert(*block_id, HashSet::new());
        }

        let mut changed = true;
        while changed {
            changed = false;

            for (block_id, block) in body.blocks.iter().rev() {
                let mut new_live_out = HashSet::new();
                for succ in block.terminator.successors() {
                    if let Some(succ_live_in) = live_in.get(&succ) {
                        new_live_out.extend(succ_live_in.clone());
                    }
                }

                let mut new_live_in = new_live_out.clone();

                for (inst_idx, _inst) in block.instructions.iter().enumerate().rev() {
                    let loc = Location {
                        block: *block_id,
                        instruction: inst_idx,
                    };

                    if let Some(defs) = def_use.defs_at.get(&loc) {
                        for def in defs {
                            new_live_in.remove(def);
                        }
                    }

                    if let Some(uses) = def_use.uses_at.get(&loc) {
                        new_live_in.extend(uses.clone());
                    }
                }

                if new_live_in != live_in[block_id] || new_live_out != live_out[block_id] {
                    live_in.insert(*block_id, new_live_in);
                    live_out.insert(*block_id, new_live_out);
                    changed = true;
                }
            }
        }

        Self { live_in, live_out }
    }

    pub fn is_live_in(&self, block: BlockId, value: &Value) -> bool {
        self.live_in
            .get(&block)
            .map(|set| set.contains(value))
            .unwrap_or(false)
    }

    pub fn is_live_out(&self, block: BlockId, value: &Value) -> bool {
        self.live_out
            .get(&block)
            .map(|set| set.contains(value))
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone)]
pub struct ReachingDefinitions {
    pub reaching_in: HashMap<BlockId, HashSet<(Value, Location)>>,
    pub reaching_out: HashMap<BlockId, HashSet<(Value, Location)>>,
}

impl ReachingDefinitions {
    pub fn compute(body: &FunctionBody) -> Self {
        let mut reaching_in = HashMap::new();
        let mut reaching_out = HashMap::new();

        for block_id in body.blocks.keys() {
            reaching_in.insert(*block_id, HashSet::new());
            reaching_out.insert(*block_id, HashSet::new());
        }

        let mut changed = true;
        while changed {
            changed = false;

            for (block_id, block) in &body.blocks {
                let new_reaching_in = HashSet::new();

                let mut new_reaching_out = new_reaching_in.clone();

                for (inst_idx, inst) in block.instructions.iter().enumerate() {
                    let loc = Location {
                        block: *block_id,
                        instruction: inst_idx,
                    };

                    if let Some(result) = inst.result() {
                        new_reaching_out.retain(|(val, _)| val != result);

                        new_reaching_out.insert((result.clone(), loc));
                    }
                }

                if new_reaching_out != reaching_out[block_id] {
                    reaching_out.insert(*block_id, new_reaching_out);
                    changed = true;
                }
            }
        }

        Self {
            reaching_in,
            reaching_out,
        }
    }

    pub fn get_reaching(&self, loc: &Location) -> HashSet<(Value, Location)> {
        self.reaching_in
            .get(&loc.block)
            .cloned()
            .unwrap_or_default()
    }
}
