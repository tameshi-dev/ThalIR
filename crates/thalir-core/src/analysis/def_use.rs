use crate::{
    block::{BlockId, Terminator},
    function::Function,
    instructions::Instruction,
    values::ValueId,
};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct DefUseChains {
    definitions: HashMap<ValueId, Definition>,
    uses: HashMap<ValueId, Vec<Use>>,
    inst_defs: HashMap<(BlockId, usize), Vec<ValueId>>,
    inst_uses: HashMap<(BlockId, usize), Vec<ValueId>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Definition {
    pub block: BlockId,
    pub instruction: usize,
    pub kind: DefKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Use {
    pub block: BlockId,
    pub instruction: usize,
    pub kind: UseKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DefKind {
    Parameter(usize),
    Instruction,
    Phi,
    Constant,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UseKind {
    Operand,
    Condition,
    Return,
    Address,
    StoreValue,
}

impl DefUseChains {
    pub fn build(function: &Function) -> Self {
        let mut definitions = HashMap::new();
        let mut uses: HashMap<ValueId, Vec<Use>> = HashMap::new();
        let mut inst_defs = HashMap::new();
        let mut inst_uses = HashMap::new();

        for (&block_id, block) in &function.body.blocks {
            for (idx, inst) in block.instructions.iter().enumerate() {
                let inst_key = (block_id, idx);

                let defs = Self::extract_defs(inst, block_id, idx);
                for (value_id, def) in &defs {
                    definitions.insert(*value_id, def.clone());
                }
                inst_defs.insert(inst_key, defs.keys().copied().collect());

                let used_values = Self::extract_uses(inst, block_id, idx);
                for (value_id, use_site) in &used_values {
                    uses.entry(*value_id).or_default().push(use_site.clone());
                }
                inst_uses.insert(inst_key, used_values.keys().copied().collect());
            }

            let term_uses = Self::extract_terminator_uses(&block.terminator, block_id);
            for (value_id, use_site) in &term_uses {
                uses.entry(*value_id).or_default().push(use_site.clone());
            }
        }

        Self {
            definitions,
            uses,
            inst_defs,
            inst_uses,
        }
    }

    fn extract_defs(
        inst: &Instruction,
        block: BlockId,
        idx: usize,
    ) -> HashMap<ValueId, Definition> {
        let mut defs = HashMap::new();

        match inst {
            Instruction::Add { result, .. }
            | Instruction::Sub { result, .. }
            | Instruction::Mul { result, .. }
            | Instruction::Div { result, .. }
            | Instruction::Mod { result, .. }
            | Instruction::Pow { result, .. }
            | Instruction::CheckedAdd { result, .. }
            | Instruction::CheckedSub { result, .. }
            | Instruction::CheckedMul { result, .. }
            | Instruction::CheckedDiv { result, .. }
            | Instruction::And { result, .. }
            | Instruction::Or { result, .. }
            | Instruction::Xor { result, .. }
            | Instruction::Not { result, .. }
            | Instruction::Shl { result, .. }
            | Instruction::Shr { result, .. }
            | Instruction::Sar { result, .. }
            | Instruction::Eq { result, .. }
            | Instruction::Ne { result, .. }
            | Instruction::Lt { result, .. }
            | Instruction::Gt { result, .. }
            | Instruction::Le { result, .. }
            | Instruction::Ge { result, .. }
            | Instruction::Load { result, .. }
            | Instruction::StorageLoad { result, .. }
            | Instruction::MappingLoad { result, .. }
            | Instruction::ArrayLoad { result, .. }
            | Instruction::ArrayLength { result, .. }
            | Instruction::Allocate { result, .. }
            | Instruction::Call { result, .. }
            | Instruction::DelegateCall { result, .. }
            | Instruction::Phi { result, .. } => {
                if let Some(id) = result.as_register() {
                    let def_kind = if matches!(inst, Instruction::Phi { .. }) {
                        DefKind::Phi
                    } else {
                        DefKind::Instruction
                    };

                    defs.insert(
                        id,
                        Definition {
                            block,
                            instruction: idx,
                            kind: def_kind,
                        },
                    );
                }
            }
            _ => {}
        }

        defs
    }

    fn extract_uses(inst: &Instruction, block: BlockId, idx: usize) -> HashMap<ValueId, Use> {
        let mut uses = HashMap::new();

        match inst {
            Instruction::Add { left, right, .. }
            | Instruction::Sub { left, right, .. }
            | Instruction::Mul { left, right, .. }
            | Instruction::Div { left, right, .. }
            | Instruction::Mod { left, right, .. } => {
                if let Some(id) = left.as_register() {
                    uses.insert(
                        id,
                        Use {
                            block,
                            instruction: idx,
                            kind: UseKind::Operand,
                        },
                    );
                }
                if let Some(id) = right.as_register() {
                    uses.insert(
                        id,
                        Use {
                            block,
                            instruction: idx,
                            kind: UseKind::Operand,
                        },
                    );
                }
            }
            Instruction::Pow { base, exp, .. } => {
                if let Some(id) = base.as_register() {
                    uses.insert(
                        id,
                        Use {
                            block,
                            instruction: idx,
                            kind: UseKind::Operand,
                        },
                    );
                }
                if let Some(id) = exp.as_register() {
                    uses.insert(
                        id,
                        Use {
                            block,
                            instruction: idx,
                            kind: UseKind::Operand,
                        },
                    );
                }
            }

            Instruction::CheckedAdd { left, right, .. }
            | Instruction::CheckedSub { left, right, .. }
            | Instruction::CheckedMul { left, right, .. }
            | Instruction::CheckedDiv { left, right, .. } => {
                if let Some(id) = left.as_register() {
                    uses.insert(
                        id,
                        Use {
                            block,
                            instruction: idx,
                            kind: UseKind::Operand,
                        },
                    );
                }
                if let Some(id) = right.as_register() {
                    uses.insert(
                        id,
                        Use {
                            block,
                            instruction: idx,
                            kind: UseKind::Operand,
                        },
                    );
                }
            }

            Instruction::And { left, right, .. }
            | Instruction::Or { left, right, .. }
            | Instruction::Xor { left, right, .. } => {
                if let Some(id) = left.as_register() {
                    uses.insert(
                        id,
                        Use {
                            block,
                            instruction: idx,
                            kind: UseKind::Operand,
                        },
                    );
                }
                if let Some(id) = right.as_register() {
                    uses.insert(
                        id,
                        Use {
                            block,
                            instruction: idx,
                            kind: UseKind::Operand,
                        },
                    );
                }
            }
            Instruction::Not { operand, .. } => {
                if let Some(id) = operand.as_register() {
                    uses.insert(
                        id,
                        Use {
                            block,
                            instruction: idx,
                            kind: UseKind::Operand,
                        },
                    );
                }
            }
            Instruction::Shl { value, shift, .. }
            | Instruction::Shr { value, shift, .. }
            | Instruction::Sar { value, shift, .. } => {
                if let Some(id) = value.as_register() {
                    uses.insert(
                        id,
                        Use {
                            block,
                            instruction: idx,
                            kind: UseKind::Operand,
                        },
                    );
                }
                if let Some(id) = shift.as_register() {
                    uses.insert(
                        id,
                        Use {
                            block,
                            instruction: idx,
                            kind: UseKind::Operand,
                        },
                    );
                }
            }

            Instruction::Eq { left, right, .. }
            | Instruction::Ne { left, right, .. }
            | Instruction::Lt { left, right, .. }
            | Instruction::Gt { left, right, .. }
            | Instruction::Le { left, right, .. }
            | Instruction::Ge { left, right, .. } => {
                if let Some(id) = left.as_register() {
                    uses.insert(
                        id,
                        Use {
                            block,
                            instruction: idx,
                            kind: UseKind::Operand,
                        },
                    );
                }
                if let Some(id) = right.as_register() {
                    uses.insert(
                        id,
                        Use {
                            block,
                            instruction: idx,
                            kind: UseKind::Operand,
                        },
                    );
                }
            }
            Instruction::Store {
                value, location, ..
            } => {
                if let Some(id) = value.as_register() {
                    uses.insert(
                        id,
                        Use {
                            block,
                            instruction: idx,
                            kind: UseKind::StoreValue,
                        },
                    );
                }

                match location {
                    crate::values::Location::Memory { base, offset } => {
                        if let Some(id) = base.as_register() {
                            uses.insert(
                                id,
                                Use {
                                    block,
                                    instruction: idx,
                                    kind: UseKind::Address,
                                },
                            );
                        }
                        if let Some(id) = offset.as_register() {
                            uses.insert(
                                id,
                                Use {
                                    block,
                                    instruction: idx,
                                    kind: UseKind::Address,
                                },
                            );
                        }
                    }
                    crate::values::Location::Storage { slot }
                    | crate::values::Location::Calldata { offset: slot }
                    | crate::values::Location::ReturnData { offset: slot } => {
                        if let Some(id) = slot.as_register() {
                            uses.insert(
                                id,
                                Use {
                                    block,
                                    instruction: idx,
                                    kind: UseKind::Address,
                                },
                            );
                        }
                    }
                    _ => {}
                }
            }
            Instruction::StorageStore { key, value } => {
                match key {
                    crate::instructions::StorageKey::Dynamic(v)
                    | crate::instructions::StorageKey::Computed(v) => {
                        if let Some(id) = v.as_register() {
                            uses.insert(
                                id,
                                Use {
                                    block,
                                    instruction: idx,
                                    kind: UseKind::Address,
                                },
                            );
                        }
                    }
                    crate::instructions::StorageKey::MappingKey { key: k, .. }
                    | crate::instructions::StorageKey::ArrayElement { index: k, .. } => {
                        if let Some(id) = k.as_register() {
                            uses.insert(
                                id,
                                Use {
                                    block,
                                    instruction: idx,
                                    kind: UseKind::Address,
                                },
                            );
                        }
                    }
                    _ => {}
                }

                if let Some(id) = value.as_register() {
                    uses.insert(
                        id,
                        Use {
                            block,
                            instruction: idx,
                            kind: UseKind::StoreValue,
                        },
                    );
                }
            }
            Instruction::Load { location, .. } => match location {
                crate::values::Location::Memory { base, offset } => {
                    if let Some(id) = base.as_register() {
                        uses.insert(
                            id,
                            Use {
                                block,
                                instruction: idx,
                                kind: UseKind::Address,
                            },
                        );
                    }
                    if let Some(id) = offset.as_register() {
                        uses.insert(
                            id,
                            Use {
                                block,
                                instruction: idx,
                                kind: UseKind::Address,
                            },
                        );
                    }
                }
                crate::values::Location::Storage { slot }
                | crate::values::Location::Calldata { offset: slot }
                | crate::values::Location::ReturnData { offset: slot } => {
                    if let Some(id) = slot.as_register() {
                        uses.insert(
                            id,
                            Use {
                                block,
                                instruction: idx,
                                kind: UseKind::Address,
                            },
                        );
                    }
                }
                _ => {}
            },
            Instruction::StorageLoad { key, .. } => match key {
                crate::instructions::StorageKey::Dynamic(v)
                | crate::instructions::StorageKey::Computed(v) => {
                    if let Some(id) = v.as_register() {
                        uses.insert(
                            id,
                            Use {
                                block,
                                instruction: idx,
                                kind: UseKind::Address,
                            },
                        );
                    }
                }
                crate::instructions::StorageKey::MappingKey { key: k, .. }
                | crate::instructions::StorageKey::ArrayElement { index: k, .. } => {
                    if let Some(id) = k.as_register() {
                        uses.insert(
                            id,
                            Use {
                                block,
                                instruction: idx,
                                kind: UseKind::Address,
                            },
                        );
                    }
                }
                _ => {}
            },
            Instruction::Return {
                value: Some(val), ..
            } => {
                if let Some(id) = val.as_register() {
                    uses.insert(
                        id,
                        Use {
                            block,
                            instruction: idx,
                            kind: UseKind::Return,
                        },
                    );
                }
            }

            Instruction::Call { args, .. } | Instruction::DelegateCall { args, .. } => {
                for arg in args {
                    if let Some(id) = arg.as_register() {
                        uses.insert(
                            id,
                            Use {
                                block,
                                instruction: idx,
                                kind: UseKind::Operand,
                            },
                        );
                    }
                }
            }
            Instruction::Phi { values, .. } => {
                for (_, value) in values {
                    if let Some(id) = value.as_register() {
                        uses.insert(
                            id,
                            Use {
                                block,
                                instruction: idx,
                                kind: UseKind::Operand,
                            },
                        );
                    }
                }
            }
            _ => {}
        }

        uses
    }

    fn extract_terminator_uses(terminator: &Terminator, block: BlockId) -> HashMap<ValueId, Use> {
        let mut uses = HashMap::new();

        match terminator {
            Terminator::Return(Some(value)) => {
                if let Some(id) = value.as_register() {
                    uses.insert(
                        id,
                        Use {
                            block,
                            instruction: usize::MAX,
                            kind: UseKind::Operand,
                        },
                    );
                }
            }
            Terminator::Branch {
                condition,
                then_args,
                else_args,
                ..
            } => {
                if let Some(id) = condition.as_register() {
                    uses.insert(
                        id,
                        Use {
                            block,
                            instruction: usize::MAX,
                            kind: UseKind::Operand,
                        },
                    );
                }
                for arg in then_args.iter().chain(else_args.iter()) {
                    if let Some(id) = arg.as_register() {
                        uses.insert(
                            id,
                            Use {
                                block,
                                instruction: usize::MAX,
                                kind: UseKind::Operand,
                            },
                        );
                    }
                }
            }
            Terminator::Jump(_, args) => {
                for arg in args {
                    if let Some(id) = arg.as_register() {
                        uses.insert(
                            id,
                            Use {
                                block,
                                instruction: usize::MAX,
                                kind: UseKind::Operand,
                            },
                        );
                    }
                }
            }
            Terminator::Switch { value, cases, .. } => {
                if let Some(id) = value.as_register() {
                    uses.insert(
                        id,
                        Use {
                            block,
                            instruction: usize::MAX,
                            kind: UseKind::Operand,
                        },
                    );
                }
                for (case_val, _) in cases {
                    if let Some(id) = case_val.as_register() {
                        uses.insert(
                            id,
                            Use {
                                block,
                                instruction: usize::MAX,
                                kind: UseKind::Operand,
                            },
                        );
                    }
                }
            }
            _ => {}
        }

        uses
    }

    pub fn get_def(&self, value: ValueId) -> Option<&Definition> {
        self.definitions.get(&value)
    }

    pub fn get_uses(&self, value: ValueId) -> &[Use] {
        self.uses.get(&value).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn get_inst_defs(&self, block: BlockId, inst: usize) -> &[ValueId] {
        self.inst_defs
            .get(&(block, inst))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn get_inst_uses(&self, block: BlockId, inst: usize) -> &[ValueId] {
        self.inst_uses
            .get(&(block, inst))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn is_used(&self, value: ValueId) -> bool {
        self.uses.contains_key(&value)
    }

    pub fn is_dead(&self, value: ValueId) -> bool {
        self.definitions.contains_key(&value) && !self.is_used(value)
    }

    pub fn reaching_defs(&self, _block: BlockId, _function: &Function) -> HashSet<ValueId> {
        self.definitions.keys().copied().collect()
    }

    pub fn live_values(
        &self,
        _block: BlockId,
        _inst: usize,
        _function: &Function,
    ) -> HashSet<ValueId> {
        self.uses.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::IRBuilder;

    #[test]
    fn test_def_use_chains() {
        let mut builder = IRBuilder::new();
        let mut contract_builder = builder.contract("TestContract");

        let mut func_builder = contract_builder.function("test");
        let mut entry_builder = func_builder.entry_block();

        let v1 = entry_builder.constant_int(10, 32);
        let v2 = entry_builder.constant_int(20, 32);
        let i32_type = crate::types::Type::Int(32);
        let v3 = entry_builder.add(v1.clone(), v2, i32_type.clone());
        let v4 = entry_builder.mul(v3.clone(), v1.clone(), i32_type);

        let v3_id = v3.as_register().expect("v3 should be a register");
        let v4_id = v4.as_register().expect("v4 should be a register");

        entry_builder.return_value(v4).unwrap();

        let function = func_builder.build().unwrap();
        let chains = DefUseChains::build(&function);

        assert!(
            chains.get_def(v3_id).is_some(),
            "v3 should have a definition"
        );
        let v3_uses = chains.get_uses(v3_id);
        assert_eq!(v3_uses.len(), 1, "v3 should be used once (in mul)");
        assert!(
            !chains.is_dead(v3_id),
            "v3 should not be dead (used in mul)"
        );

        assert!(
            chains.get_def(v4_id).is_some(),
            "v4 should have a definition"
        );
        let v4_uses = chains.get_uses(v4_id);
        assert_eq!(v4_uses.len(), 1, "v4 should be used once (in return)");
        assert!(
            !chains.is_dead(v4_id),
            "v4 should not be dead (used in return)"
        );
    }
}
