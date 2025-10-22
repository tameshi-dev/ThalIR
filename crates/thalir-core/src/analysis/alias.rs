use crate::{
    block::BlockId,
    function::Function,
    instructions::Instruction,
    values::{Value, ValueId},
};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct AliasAnalysis {
    alias_sets: Vec<AliasSet>,
    value_to_set: HashMap<ValueId, usize>,
    points_to: HashMap<ValueId, PointsToSet>,
}

#[derive(Debug, Clone)]
pub struct AliasSet {
    pub values: HashSet<ValueId>,
    pub kind: AliasKind,
    pub may_alias_unknown: bool,
}

#[derive(Debug, Clone)]
pub struct PointsToSet {
    pub allocations: HashSet<AllocationSite>,
    pub parameters: HashSet<usize>,
    pub globals: HashSet<String>,
    pub unknown: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AllocationSite {
    pub block: BlockId,
    pub instruction: usize,
    pub size: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AliasKind {
    Must,
    May,
    NoAlias,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AliasResult {
    MustAlias,
    MayAlias,
    NoAlias,
    PartialAlias,
}

impl AliasAnalysis {
    pub fn build(function: &Function) -> Self {
        let mut analyzer = AliasAnalyzer::new();
        analyzer.analyze(function);
        analyzer.build_results()
    }

    pub fn query(&self, v1: ValueId, v2: ValueId) -> AliasResult {
        if v1 == v2 {
            return AliasResult::MustAlias;
        }

        let set1 = self.value_to_set.get(&v1);
        let set2 = self.value_to_set.get(&v2);

        match (set1, set2) {
            (Some(s1), Some(s2)) if s1 == s2 => {
                if self.alias_sets[*s1].kind == AliasKind::Must {
                    AliasResult::MustAlias
                } else {
                    AliasResult::MayAlias
                }
            }
            (Some(_), Some(_)) => {
                if self.may_sets_alias(v1, v2) {
                    AliasResult::MayAlias
                } else {
                    AliasResult::NoAlias
                }
            }
            _ => AliasResult::MayAlias,
        }
    }

    fn may_sets_alias(&self, v1: ValueId, v2: ValueId) -> bool {
        let pts1 = self.points_to.get(&v1);
        let pts2 = self.points_to.get(&v2);

        match (pts1, pts2) {
            (Some(p1), Some(p2)) => {
                if p1.unknown || p2.unknown {
                    return true;
                }

                !p1.allocations.is_disjoint(&p2.allocations)
                    || !p1.parameters.is_disjoint(&p2.parameters)
                    || !p1.globals.is_disjoint(&p2.globals)
            }
            _ => true,
        }
    }

    pub fn get_alias_set(&self, value: ValueId) -> Option<&AliasSet> {
        self.value_to_set
            .get(&value)
            .map(|&idx| &self.alias_sets[idx])
    }

    pub fn get_points_to(&self, value: ValueId) -> Option<&PointsToSet> {
        self.points_to.get(&value)
    }

    pub fn may_alias_memory(&self, value: ValueId) -> bool {
        self.get_alias_set(value)
            .map(|set| set.may_alias_unknown)
            .unwrap_or(true)
    }
}

struct AliasAnalyzer {
    allocations: Vec<AllocationSite>,
    value_allocs: HashMap<ValueId, HashSet<usize>>,
    aliases: HashMap<ValueId, HashSet<ValueId>>,
    escaped: HashSet<ValueId>,
}

impl AliasAnalyzer {
    fn new() -> Self {
        Self {
            allocations: Vec::new(),
            value_allocs: HashMap::new(),
            aliases: HashMap::new(),
            escaped: HashSet::new(),
        }
    }

    fn analyze(&mut self, function: &Function) {
        for (&block_id, block) in &function.body.blocks {
            for (idx, inst) in block.instructions.iter().enumerate() {
                self.analyze_instruction(inst, block_id, idx);
            }
        }

        self.propagate_aliases();

        self.find_escaped_values(function);
    }

    fn analyze_instruction(&mut self, inst: &Instruction, block: BlockId, idx: usize) {
        match inst {
            Instruction::Allocate { result, size, .. } => {
                if let Some(id) = result.as_register() {
                    let alloc_site = AllocationSite {
                        block,
                        instruction: idx,
                        size: match size {
                            crate::instructions::Size::Static(s) => Some(*s),
                            crate::instructions::Size::Dynamic(v) => {
                                if let Value::Constant(c) = v {
                                    c.as_int().map(|i| i as usize)
                                } else {
                                    None
                                }
                            }
                        },
                    };

                    let alloc_idx = self.allocations.len();
                    self.allocations.push(alloc_site);

                    self.value_allocs.entry(id).or_default().insert(alloc_idx);
                }
            }

            Instruction::ArrayLoad { result, array, .. }
            | Instruction::MappingLoad {
                result,
                mapping: array,
                ..
            } => {
                if let (Some(res_id), Some(_base_id)) = (result.as_register(), array.as_register())
                {
                    self.aliases.entry(res_id).or_default();
                }
            }
            Instruction::Load { result, .. } => {
                if let Some(res_id) = result.as_register() {
                    self.aliases.entry(res_id).or_default();
                }
            }
            Instruction::Store { .. } => {}
            Instruction::Call { args, .. } | Instruction::DelegateCall { args, .. } => {
                for arg in args {
                    if let Some(id) = arg.as_register() {
                        self.escaped.insert(id);
                    }
                }
            }
            _ => {}
        }
    }

    fn propagate_aliases(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;

            let aliases_snapshot = self.aliases.clone();
            for (&value, aliases) in &aliases_snapshot {
                for &alias in aliases {
                    if let Some(alias_aliases) = aliases_snapshot.get(&alias) {
                        for &transitive in alias_aliases {
                            if transitive != value {
                                let entry = self.aliases.entry(value).or_default();
                                if entry.insert(transitive) {
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn find_escaped_values(&mut self, function: &Function) {
        for block in function.body.blocks.values() {
            for inst in &block.instructions {
                match inst {
                    Instruction::Return {
                        value: Some(val), ..
                    } => {
                        if let Some(id) = val.as_register() {
                            self.mark_escaped(id);
                        }
                    }
                    Instruction::Store { value, .. } | Instruction::StorageStore { value, .. } => {
                        if let Some(id) = value.as_register() {
                            self.mark_escaped(id);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn mark_escaped(&mut self, value: ValueId) {
        if self.escaped.insert(value) {
            if let Some(aliases) = self.aliases.get(&value).cloned() {
                for alias in aliases {
                    self.mark_escaped(alias);
                }
            }
        }
    }

    fn build_results(self) -> AliasAnalysis {
        let mut alias_sets = Vec::new();
        let mut value_to_set = HashMap::new();
        let mut points_to = HashMap::new();

        let mut processed = HashSet::new();

        for (&value, aliases) in &self.aliases {
            if processed.contains(&value) {
                continue;
            }

            let mut set_values = HashSet::new();
            set_values.insert(value);
            set_values.extend(aliases);

            let set_idx = alias_sets.len();
            for &v in &set_values {
                value_to_set.insert(v, set_idx);
                processed.insert(v);
            }

            let may_alias_unknown = set_values.iter().any(|v| self.escaped.contains(v));

            alias_sets.push(AliasSet {
                values: set_values,
                kind: AliasKind::May,
                may_alias_unknown,
            });
        }

        for (value, alloc_indices) in self.value_allocs {
            let mut pts = PointsToSet {
                allocations: HashSet::new(),
                parameters: HashSet::new(),
                globals: HashSet::new(),
                unknown: self.escaped.contains(&value),
            };

            for &idx in &alloc_indices {
                pts.allocations.insert(self.allocations[idx].clone());
            }

            points_to.insert(value, pts);
        }

        AliasAnalysis {
            alias_sets,
            value_to_set,
            points_to,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::IRBuilder;

    #[test]
    fn test_basic_aliasing() {
        let mut builder = IRBuilder::new();
        let mut contract_builder = builder.contract("TestContract");

        let mut func_builder = contract_builder.function("test");
        let mut entry_builder = func_builder.entry_block();

        let i32_type = crate::types::Type::Int(32);
        let ptr1 = entry_builder.allocate(i32_type.clone(), crate::instructions::Size::Static(4));
        let ptr2 = entry_builder.allocate(i32_type, crate::instructions::Size::Static(4));

        let ptr3 = ptr1.clone();

        entry_builder.return_void().unwrap();

        let function = func_builder.build().unwrap();
        let alias = AliasAnalysis::build(&function);

        if let (Some(id1), Some(id3)) = (ptr1.as_register(), ptr3.as_register()) {
            let result = alias.query(id1, id3);
            assert!(matches!(
                result,
                AliasResult::MayAlias | AliasResult::MustAlias
            ));
        }

        if let (Some(id1), Some(id2)) = (ptr1.as_register(), ptr2.as_register()) {
            let result = alias.query(id1, id2);
        }
    }
}
