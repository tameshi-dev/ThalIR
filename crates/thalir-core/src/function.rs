use crate::block::{BasicBlock, BlockId};
use crate::contract::ModifierRef;
use crate::types::Type;
use cranelift::codegen::ir as clif_ir;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub signature: FunctionSignature,
    pub visibility: Visibility,
    pub mutability: Mutability,
    pub modifiers: Vec<ModifierRef>,
    pub body: FunctionBody,
    pub metadata: FunctionMetadata,
}

impl Function {
    pub fn new(signature: FunctionSignature) -> Self {
        Self {
            signature,
            visibility: Visibility::Private,
            mutability: Mutability::NonPayable,
            modifiers: Vec::new(),
            body: FunctionBody::new(),
            metadata: FunctionMetadata::default(),
        }
    }

    pub fn name(&self) -> &str {
        &self.signature.name
    }

    pub fn entry_block(&self) -> BlockId {
        self.body.entry_block()
    }

    pub fn analyze_metadata(&mut self) {
        let (calls_external, modifies_state) = self
            .body
            .blocks
            .values()
            .flat_map(|block| &block.instructions)
            .fold((false, false), |(ext, state), inst| {
                (
                    ext || inst.is_external_call(),
                    state || inst.is_state_changing(),
                )
            });

        self.metadata.calls_external = calls_external;
        self.metadata.modifies_state = modifies_state;
        self.metadata.can_reenter = calls_external && modifies_state;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    pub name: String,
    pub params: Vec<Parameter>,
    pub returns: Vec<Type>,
    pub is_payable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
    pub location: DataLocation,
}

impl Parameter {
    pub fn new(name: impl Into<String>, param_type: Type) -> Self {
        Self {
            name: name.into(),
            param_type,
            location: DataLocation::Memory,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataLocation {
    Storage,
    Memory,
    Calldata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    External,
    Internal,
    Private,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mutability {
    Pure,
    View,
    NonPayable,
    Payable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionBody {
    pub entry_block: BlockId,
    pub blocks: IndexMap<BlockId, BasicBlock>,
    pub locals: Vec<LocalVariable>,
    #[serde(skip)]
    pub cranelift_func: Option<CraneliftFunction>,
    next_block_id: u32,
    next_local_id: u32,
}

impl FunctionBody {
    pub fn new() -> Self {
        let entry_block = BlockId(0);
        let mut blocks = IndexMap::new();
        blocks.insert(entry_block, BasicBlock::new(entry_block));

        Self {
            entry_block,
            blocks,
            locals: Vec::new(),
            cranelift_func: None,
            next_block_id: 1,
            next_local_id: 0,
        }
    }

    pub fn create_block(&mut self) -> BlockId {
        let id = BlockId(self.next_block_id);
        self.next_block_id += 1;
        self.blocks.insert(id, BasicBlock::new(id));
        id
    }

    pub fn get_block(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.get(&id)
    }

    pub fn get_block_mut(&mut self, id: BlockId) -> Option<&mut BasicBlock> {
        self.blocks.get_mut(&id)
    }

    pub fn entry_block(&self) -> BlockId {
        self.entry_block
    }

    pub fn add_local(&mut self, var: LocalVariable) -> LocalId {
        let id = LocalId(self.next_local_id);
        self.next_local_id += 1;
        self.locals.push(var);
        id
    }
}

impl Default for FunctionBody {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalVariable {
    pub id: LocalId,
    pub name: String,
    pub var_type: Type,
    pub location: DataLocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocalId(pub u32);

pub struct CraneliftFunction {
    pub func: clif_ir::Function,
}

impl std::fmt::Debug for CraneliftFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CraneliftFunction")
            .field("name", &self.func.name)
            .finish()
    }
}

impl Clone for CraneliftFunction {
    fn clone(&self) -> Self {
        Self {
            func: self.func.clone(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FunctionMetadata {
    pub is_constructor: bool,
    pub is_fallback: bool,
    pub is_receive: bool,
    pub estimated_gas: Option<u64>,
    pub can_reenter: bool,
    pub has_assembly: bool,
    pub calls_external: bool,
    pub modifies_state: bool,
}
