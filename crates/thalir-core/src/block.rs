use crate::instructions::Instruction;
use crate::types::Type;
use crate::values::{SourceLocation, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BlockId(pub u32);

impl std::fmt::Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "block{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicBlock {
    pub id: BlockId,
    pub params: Vec<BlockParam>,
    pub instructions: Vec<Instruction>,
    pub terminator: Terminator,
    pub metadata: BlockMetadata,
}

impl BasicBlock {
    pub fn new(id: BlockId) -> Self {
        Self {
            id,
            params: Vec::new(),
            instructions: Vec::new(),
            terminator: Terminator::Invalid,
            metadata: BlockMetadata::default(),
        }
    }

    pub fn add_param(&mut self, param: BlockParam) {
        self.params.push(param);
    }

    pub fn add_instruction(&mut self, inst: Instruction) {
        self.instructions.push(inst);
    }

    pub fn set_terminator(&mut self, term: Terminator) {
        self.terminator = term;
    }

    pub fn is_terminated(&self) -> bool {
        !matches!(self.terminator, Terminator::Invalid)
    }

    pub fn predecessors(&self) -> Vec<BlockId> {
        Vec::new()
    }

    pub fn successors(&self) -> Vec<BlockId> {
        match &self.terminator {
            Terminator::Jump(target, _) => vec![*target],
            Terminator::Branch {
                then_block,
                else_block,
                ..
            } => {
                vec![*then_block, *else_block]
            }
            Terminator::Switch { default, cases, .. } => {
                let mut succs = vec![*default];
                succs.extend(cases.iter().map(|(_, block)| *block));
                succs
            }
            _ => Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockParam {
    pub name: String,
    pub param_type: Type,
}

impl BlockParam {
    pub fn new(name: impl Into<String>, param_type: Type) -> Self {
        Self {
            name: name.into(),
            param_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Terminator {
    Jump(BlockId, Vec<Value>),
    Branch {
        condition: Value,
        then_block: BlockId,
        then_args: Vec<Value>,
        else_block: BlockId,
        else_args: Vec<Value>,
    },

    Switch {
        value: Value,
        default: BlockId,
        cases: Vec<(Value, BlockId)>,
    },

    Return(Option<Value>),

    Revert(String),

    Panic(String),

    Invalid,
}

impl Terminator {
    pub fn successors(&self) -> Vec<BlockId> {
        match self {
            Terminator::Jump(target, _) => vec![*target],
            Terminator::Branch {
                then_block,
                else_block,
                ..
            } => vec![*then_block, *else_block],
            Terminator::Switch { default, cases, .. } => {
                let mut blocks = vec![*default];
                blocks.extend(cases.iter().map(|(_, block)| *block));
                blocks
            }
            Terminator::Return(_)
            | Terminator::Revert(_)
            | Terminator::Panic(_)
            | Terminator::Invalid => vec![],
        }
    }

    pub fn is_return(&self) -> bool {
        matches!(self, Terminator::Return(_))
    }

    pub fn is_revert(&self) -> bool {
        matches!(self, Terminator::Revert(_) | Terminator::Panic(_))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockMetadata {
    pub is_loop_header: bool,
    pub is_loop_exit: bool,
    pub loop_depth: u32,
    pub predecessors: Vec<BlockId>,
    pub dominators: Vec<BlockId>,
    pub is_reachable: bool,
    pub instruction_locations: HashMap<usize, SourceLocation>,
}

impl BlockMetadata {
    pub fn get_location(&self, index: usize) -> Option<&SourceLocation> {
        self.instruction_locations.get(&index)
    }

    pub fn set_location(&mut self, index: usize, location: SourceLocation) {
        self.instruction_locations.insert(index, location);
    }
}
