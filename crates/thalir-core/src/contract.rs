use crate::function::Function;
use crate::source_location::SourceFiles;
use crate::types::Type;
use indexmap::IndexMap;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub name: String,
    pub functions: IndexMap<String, Function>,
    pub storage_layout: StorageLayout,
    pub events: Vec<EventDefinition>,
    pub modifiers: Vec<ModifierDefinition>,
    pub constants: Vec<ConstantDefinition>,
    pub metadata: ContractMetadata,
    #[serde(skip)]
    pub source_files: SourceFiles,
}

impl Contract {
    pub fn new(name: String) -> Self {
        Self {
            name,
            functions: IndexMap::new(),
            storage_layout: StorageLayout::default(),
            events: Vec::new(),
            modifiers: Vec::new(),
            constants: Vec::new(),
            metadata: ContractMetadata::default(),
            source_files: SourceFiles::new(),
        }
    }

    pub fn add_function(&mut self, mut function: Function) {
        function.analyze_metadata();
        self.functions
            .insert(function.signature.name.clone(), function);
    }

    pub fn get_function(&self, name: &str) -> Option<&Function> {
        self.functions.get(name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContractMetadata {
    pub version: String,
    pub security_flags: SecurityFlags,
    pub optimization_level: OptLevel,
    pub source_hash: Option<[u8; 32]>,
    pub source_file: Option<String>,
    pub source_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityFlags {
    pub has_external_calls: bool,
    pub has_delegatecalls: bool,
    pub has_selfdestruct: bool,
    pub has_assembly: bool,
    pub is_upgradeable: bool,
    pub uses_randomness: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum OptLevel {
    #[default]
    None,
    Size,
    Speed,
    SpeedAndSize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageLayout {
    pub slots: Vec<StorageSlot>,
    pub mappings: Vec<MappingLayout>,
    pub arrays: Vec<ArrayLayout>,
    pub structs: Vec<StructLayout>,
}

impl StorageLayout {
    pub fn add_variable(&mut self, name: String, ty: Type, slot: u32) {
        self.slots.push(StorageSlot {
            slot: BigUint::from(slot),
            offset: 0,
            var_type: ty,
            name,
            packed_with: Vec::new(),
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSlot {
    pub slot: BigUint,
    pub offset: u8,
    pub var_type: Type,
    pub name: String,
    pub packed_with: Vec<PackedVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackedVariable {
    pub name: String,
    pub offset: u8,
    pub size: u8,
    pub var_type: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingLayout {
    pub base_slot: BigUint,
    pub key_type: Type,
    pub value_type: Type,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayLayout {
    pub base_slot: BigUint,
    pub element_type: Type,
    pub is_dynamic: bool,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructLayout {
    pub base_slot: BigUint,
    pub fields: Vec<StructField>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub field_type: Type,
    pub offset: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDefinition {
    pub id: EventId,
    pub name: String,
    pub parameters: Vec<EventParameter>,
    pub anonymous: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventParameter {
    pub name: String,
    pub param_type: Type,
    pub indexed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifierDefinition {
    pub id: ModifierId,
    pub name: String,
    pub parameters: Vec<ModifierParameter>,
    pub body: ModifierBody,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModifierId(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifierParameter {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifierBody {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifierRef {
    pub id: ModifierId,
    pub arguments: Vec<crate::values::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantDefinition {
    pub name: String,
    pub const_type: Type,
    pub value: crate::values::Constant,
}
