use crate::values::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvmOp {
    GetTxOrigin,
    GetGasPrice,
    GetGasLeft,
    GetBlockHash(Value),
    GetChainId,
    GetBaseFee,
    SelfDestruct(Value),
    Log {
        topics: Vec<Value>,
        data: Value,
    },

    CreateContract {
        value: Value,
        code: Value,
        salt: Option<Value>,
    },

    ExternalCall {
        target: Value,
        value: Value,
        data: Value,
        gas: Option<Value>,
        call_type: ExternalCallType,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalCallType {
    Call,
    DelegateCall,
    StaticCall,
    CallCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferOp {
    Transfer {
        to: Value,
        amount: Value,
    },

    Send {
        to: Value,
        amount: Value,
        gas: Value,
    },

    SafeTransfer {
        to: Value,
        amount: Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceDefinition {
    pub name: String,
    pub functions: Vec<InterfaceFunction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceFunction {
    pub selector: [u8; 4],
    pub name: String,
    pub inputs: Vec<crate::types::Type>,
    pub outputs: Vec<crate::types::Type>,
    pub state_mutability: StateMutability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateMutability {
    Pure,
    View,
    NonPayable,
    Payable,
}

pub mod constants {

    pub const MAX_CODE_SIZE: usize = 24576;

    pub const MAX_CALL_DEPTH: u32 = 1024;

    pub const WORD_SIZE: usize = 32;

    pub const ADDRESS_SIZE: usize = 20;
}
