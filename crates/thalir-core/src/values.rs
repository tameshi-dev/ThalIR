use crate::types::Type;
use num_bigint::{BigInt, BigUint};
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValueId {
    Var(VarId),
    Temp(TempId),
    Param(ParamId),
    BlockParam(BlockParamId),
    Storage(StorageRefId),
    Memory(MemoryRefId),
    Global(GlobalId),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Value {
    Register(ValueId),
    Variable(VarId),
    Temp(TempId),
    Param(ParamId),
    BlockParam(BlockParamId),
    Constant(Constant),
    StorageRef(StorageRefId),
    MemoryRef(MemoryRefId),
    Global(GlobalId),
    Undefined,
}

impl Value {
    pub fn as_register(&self) -> Option<ValueId> {
        match self {
            Value::Register(id) => Some(*id),
            Value::Variable(v) => Some(ValueId::Var(*v)),
            Value::Temp(t) => Some(ValueId::Temp(*t)),
            Value::Param(p) => Some(ValueId::Param(*p)),
            Value::BlockParam(bp) => Some(ValueId::BlockParam(*bp)),
            Value::StorageRef(s) => Some(ValueId::Storage(*s)),
            Value::MemoryRef(m) => Some(ValueId::Memory(*m)),
            Value::Global(g) => Some(ValueId::Global(*g)),
            _ => None,
        }
    }

    pub fn is_constant(&self) -> bool {
        matches!(self, Value::Constant(_))
    }

    pub fn is_reference(&self) -> bool {
        matches!(self, Value::StorageRef(_) | Value::MemoryRef(_))
    }

    pub fn as_constant(&self) -> Option<&Constant> {
        match self {
            Value::Constant(c) => Some(c),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VarId(pub u32);

impl std::fmt::Display for VarId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TempId(pub u32);

impl std::fmt::Display for TempId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "t{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParamId(pub u32);

impl std::fmt::Display for ParamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "p{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockParamId {
    pub block: crate::block::BlockId,
    pub index: u32,
}

impl std::fmt::Display for BlockParamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:p{}", self.block, self.index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StorageRefId(pub u32);

impl std::fmt::Display for StorageRefId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sref{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoryRefId(pub u32);

impl std::fmt::Display for MemoryRefId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mref{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GlobalId(pub u32);

impl std::fmt::Display for GlobalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "g{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Constant {
    Bool(bool),
    Uint(BigUint, u16),
    Int(BigInt, u16),
    Address([u8; 20]),
    Bytes(Vec<u8>),
    String(String),
    Null,
}

impl Constant {
    pub fn zero(ty: &Type) -> Option<Self> {
        match ty {
            Type::Bool => Some(Constant::Bool(false)),
            Type::Uint(bits) => Some(Constant::Uint(BigUint::from(0u32), *bits)),
            Type::Int(bits) => Some(Constant::Int(BigInt::from(0), *bits)),
            Type::Address => Some(Constant::Address([0; 20])),
            Type::Bytes(n) => Some(Constant::Bytes(vec![0; *n as usize])),
            _ => None,
        }
    }

    pub fn one(ty: &Type) -> Option<Self> {
        match ty {
            Type::Bool => Some(Constant::Bool(true)),
            Type::Uint(bits) => Some(Constant::Uint(BigUint::from(1u32), *bits)),
            Type::Int(bits) => Some(Constant::Int(BigInt::from(1), *bits)),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Constant::Uint(val, _) => val.to_u64_digits().first().copied().and_then(|v| {
                if v <= i64::MAX as u64 {
                    Some(v as i64)
                } else {
                    None
                }
            }),
            Constant::Int(val, _) => val.to_i64(),
            Constant::Bool(b) => Some(if *b { 1 } else { 0 }),
            _ => None,
        }
    }
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::Bool(b) => write!(f, "{}", b),
            Constant::Uint(val, bits) => write!(f, "{}u{}", val, bits),
            Constant::Int(val, bits) => write!(f, "{}i{}", val, bits),
            Constant::Address(addr) => write!(f, "0x{}", hex::encode(addr)),
            Constant::Bytes(bytes) => write!(f, "0x{}", hex::encode(bytes)),
            Constant::String(s) => write!(f, "\"{}\"", s),
            Constant::Null => write!(f, "null"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Location {
    Stack { offset: i32 },
    Memory { base: Value, offset: Value },
    Storage { slot: Value },
    Calldata { offset: Value },
    ReturnData { offset: Value },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValueMetadata {
    pub is_tainted: bool,
    pub is_user_input: bool,
    pub is_constant_folded: bool,
    pub range: Option<ValueRange>,
    pub source_location: Option<SourceLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueRange {
    pub min: Constant,
    pub max: Constant,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
    pub start_byte: usize,
    pub end_byte: usize,
}

impl SourceLocation {
    pub fn new(file: String, line: u32, column: u32, start_byte: usize, end_byte: usize) -> Self {
        Self {
            file,
            line,
            column,
            end_line: None,
            end_column: None,
            start_byte,
            end_byte,
        }
    }

    pub fn from_node(file: String, node: &tree_sitter::Node) -> Self {
        let start = node.start_position();
        let end = node.end_position();

        Self {
            file,
            line: start.row as u32 + 1,
            column: start.column as u32 + 1,
            end_line: Some(end.row as u32 + 1),
            end_column: Some(end.column as u32 + 1),
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
        }
    }

    pub fn extract_snippet(&self, source_code: &str) -> Option<String> {
        if self.start_byte < source_code.len() && self.end_byte <= source_code.len() {
            let snippet = &source_code[self.start_byte..self.end_byte];
            return Some(snippet.to_string());
        }

        let lines: Vec<&str> = source_code.lines().collect();
        if lines.is_empty() {
            return None;
        }

        let start_line = (self.line as usize).saturating_sub(1);
        let end_line = self
            .end_line
            .map(|l| l as usize)
            .unwrap_or(self.line as usize);

        if start_line >= lines.len() {
            return None;
        }

        let end_line = end_line.min(lines.len());

        if start_line == end_line - 1 {
            Some(lines[start_line].to_string())
        } else {
            Some(lines[start_line..end_line].join("\n"))
        }
    }
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
