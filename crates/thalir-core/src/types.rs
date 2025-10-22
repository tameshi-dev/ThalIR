use cranelift::codegen::ir::types as clif_types;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    Bool,
    Uint(u16),
    Int(u16),
    Address,
    Bytes(u8),
    String,
    Array(Box<Type>, Option<usize>),
    Mapping(Box<Type>, Box<Type>),
    Struct(StructId),
    Enum(EnumId),
    Contract(ContractId),
    Function(Box<FunctionType>),
    StoragePointer(Box<Type>),
    MemoryPointer(Box<Type>),
    CalldataPointer(Box<Type>),
    Bytes4,
    Bytes20,
    Bytes32,
    ClifType(ClifTypeWrapper),
}

impl Type {
    pub fn to_cranelift(&self) -> Option<clif_types::Type> {
        match self {
            Type::Bool => Some(clif_types::I8),
            Type::Uint(8) | Type::Int(8) => Some(clif_types::I8),
            Type::Uint(16) | Type::Int(16) => Some(clif_types::I16),
            Type::Uint(32) | Type::Int(32) => Some(clif_types::I32),
            Type::Uint(64) | Type::Int(64) => Some(clif_types::I64),
            Type::Uint(128) | Type::Int(128) => Some(clif_types::I128),
            Type::Address => Some(clif_types::I64),
            Type::ClifType(wrapper) => Some(wrapper.0),
            _ => None,
        }
    }

    pub fn size_bytes(&self) -> usize {
        match self {
            Type::Bool => 1,
            Type::Uint(bits) | Type::Int(bits) => (*bits as usize + 7) / 8,
            Type::Address => 20,
            Type::Bytes(n) => *n as usize,
            Type::String => 32,
            Type::Array(_, _) => 32,
            Type::Mapping(_, _) => 0,
            Type::ClifType(wrapper) => wrapper.0.bytes() as usize,
            _ => 32,
        }
    }

    pub fn is_reference(&self) -> bool {
        matches!(
            self,
            Type::String
                | Type::Array(_, None)
                | Type::StoragePointer(_)
                | Type::MemoryPointer(_)
                | Type::CalldataPointer(_)
        )
    }

    pub fn is_value_type(&self) -> bool {
        matches!(
            self,
            Type::Bool
                | Type::Uint(_)
                | Type::Int(_)
                | Type::Address
                | Type::Bytes(_)
                | Type::Array(_, Some(_))
        )
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Bool => write!(f, "bool"),
            Type::Uint(bits) => write!(f, "uint{}", bits),
            Type::Int(bits) => write!(f, "int{}", bits),
            Type::Address => write!(f, "address"),
            Type::Bytes4 => write!(f, "bytes4"),
            Type::Bytes20 => write!(f, "bytes20"),
            Type::Bytes32 => write!(f, "bytes32"),
            Type::Bytes(n) => write!(f, "bytes{}", n),
            Type::String => write!(f, "string"),
            Type::Array(elem, Some(size)) => write!(f, "{}[{}]", elem, size),
            Type::Array(elem, None) => write!(f, "{}[]", elem),
            Type::Mapping(key, value) => write!(f, "mapping({} => {})", key, value),
            Type::Struct(id) => write!(f, "struct_{}", id.0),
            Type::Enum(id) => write!(f, "enum_{}", id.0),
            Type::Contract(id) => write!(f, "contract_{}", id.0),
            Type::Function(ft) => write!(f, "function({})", ft),
            Type::StoragePointer(inner) => write!(f, "storage_ptr<{}>", inner),
            Type::MemoryPointer(inner) => write!(f, "memory_ptr<{}>", inner),
            Type::CalldataPointer(inner) => write!(f, "calldata_ptr<{}>", inner),
            Type::ClifType(wrapper) => write!(f, "clif_{:?}", wrapper.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClifTypeWrapper(pub clif_types::Type);

impl Serialize for ClifTypeWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{:?}", self.0))
    }
}

impl<'de> Deserialize<'de> for ClifTypeWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let t = match s.as_str() {
            "I8" => clif_types::I8,
            "I16" => clif_types::I16,
            "I32" => clif_types::I32,
            "I64" => clif_types::I64,
            "I128" => clif_types::I128,
            "F32" => clif_types::F32,
            "F64" => clif_types::F64,
            _ => {
                return Err(serde::de::Error::custom(format!(
                    "Unknown cranelift type: {}",
                    s
                )))
            }
        };
        Ok(ClifTypeWrapper(t))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionType {
    pub params: Vec<Type>,
    pub returns: Vec<Type>,
    pub is_payable: bool,
    pub is_pure: bool,
    pub is_view: bool,
}

impl fmt::Display for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let params = self
            .params
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let returns = if self.returns.is_empty() {
            String::new()
        } else {
            format!(
                " returns ({})",
                self.returns
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        write!(f, "({}){}", params, returns)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StructId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EnumId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractId(pub u32);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TypeRegistry {
    pub structs: IndexMap<StructId, StructDefinition>,
    pub enums: IndexMap<EnumId, EnumDefinition>,
    pub contracts: IndexMap<ContractId, ContractInterface>,
    next_struct_id: u32,
    next_enum_id: u32,
    next_contract_id: u32,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_struct(&mut self, def: StructDefinition) -> StructId {
        let id = StructId(self.next_struct_id);
        self.next_struct_id += 1;
        self.structs.insert(id, def);
        id
    }

    pub fn add_enum(&mut self, def: EnumDefinition) -> EnumId {
        let id = EnumId(self.next_enum_id);
        self.next_enum_id += 1;
        self.enums.insert(id, def);
        id
    }

    pub fn add_contract(&mut self, interface: ContractInterface) -> ContractId {
        let id = ContractId(self.next_contract_id);
        self.next_contract_id += 1;
        self.contracts.insert(id, interface);
        id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructDefinition {
    pub name: String,
    pub fields: Vec<StructFieldDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructFieldDef {
    pub name: String,
    pub field_type: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumDefinition {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInterface {
    pub name: String,
    pub functions: Vec<String>,
}
