use crate::values::Value;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageOp {
    Load { slot: StorageLocation },
    Store { slot: StorageLocation, value: Value },
    Clear { slot: StorageLocation },
    ComputeMappingSlot { base: BigUint, key: Value },
    ComputeArraySlot { base: BigUint, index: Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageLocation {
    Slot(BigUint),
    Computed(Value),
    Mapping {
        base: BigUint,
        key: Box<StorageLocation>,
    },

    ArrayElement {
        base: BigUint,
        index: Value,
    },

    StructField {
        base: BigUint,
        offset: u8,
    },

    Packed {
        slot: BigUint,
        offset: u8,
        size: u8,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackingInfo {
    pub variables: Vec<PackedVar>,
    pub total_size: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackedVar {
    pub name: String,
    pub offset: u8,
    pub size: u8,
    pub mask: BigUint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessPattern {
    Sequential,
    Random,
    MappingKnownKeys(Vec<Value>),
    MappingComputedKeys,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutOptimization {
    pub pack_structs: bool,
    pub reorder_fields: bool,
    pub alignment: usize,
}

impl StorageLocation {
    pub fn is_static(&self) -> bool {
        matches!(self, StorageLocation::Slot(_))
    }

    pub fn is_dynamic(&self) -> bool {
        matches!(
            self,
            StorageLocation::Computed(_) | StorageLocation::ArrayElement { .. }
        )
    }

    pub fn base_slot(&self) -> Option<&BigUint> {
        match self {
            StorageLocation::Slot(s) => Some(s),
            StorageLocation::Mapping { base, .. } => Some(base),
            StorageLocation::ArrayElement { base, .. } => Some(base),
            StorageLocation::StructField { base, .. } => Some(base),
            StorageLocation::Packed { slot, .. } => Some(slot),
            _ => None,
        }
    }
}

pub fn compute_mapping_slot(base: &BigUint, key: &[u8]) -> BigUint {
    base.clone() + BigUint::from(key.len())
}

pub fn compute_array_slot(base: &BigUint, index: usize) -> BigUint {
    base.clone() + BigUint::from(index)
}
