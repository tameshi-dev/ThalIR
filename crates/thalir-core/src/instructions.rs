use crate::contract::EventId;
use crate::types::Type;
use crate::values::{Location, Value};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    Add {
        result: Value,
        left: Value,
        right: Value,
        ty: Type,
    },
    Sub {
        result: Value,
        left: Value,
        right: Value,
        ty: Type,
    },
    Mul {
        result: Value,
        left: Value,
        right: Value,
        ty: Type,
    },
    Div {
        result: Value,
        left: Value,
        right: Value,
        ty: Type,
    },
    Mod {
        result: Value,
        left: Value,
        right: Value,
        ty: Type,
    },
    Pow {
        result: Value,
        base: Value,
        exp: Value,
    },

    CheckedAdd {
        result: Value,
        left: Value,
        right: Value,
        ty: Type,
    },
    CheckedSub {
        result: Value,
        left: Value,
        right: Value,
        ty: Type,
    },
    CheckedMul {
        result: Value,
        left: Value,
        right: Value,
        ty: Type,
    },
    CheckedDiv {
        result: Value,
        left: Value,
        right: Value,
        ty: Type,
    },

    And {
        result: Value,
        left: Value,
        right: Value,
    },
    Or {
        result: Value,
        left: Value,
        right: Value,
    },
    Xor {
        result: Value,
        left: Value,
        right: Value,
    },
    Not {
        result: Value,
        operand: Value,
    },
    Shl {
        result: Value,
        value: Value,
        shift: Value,
    },
    Shr {
        result: Value,
        value: Value,
        shift: Value,
    },
    Sar {
        result: Value,
        value: Value,
        shift: Value,
    },

    Eq {
        result: Value,
        left: Value,
        right: Value,
    },
    Ne {
        result: Value,
        left: Value,
        right: Value,
    },
    Lt {
        result: Value,
        left: Value,
        right: Value,
    },
    Gt {
        result: Value,
        left: Value,
        right: Value,
    },
    Le {
        result: Value,
        left: Value,
        right: Value,
    },
    Ge {
        result: Value,
        left: Value,
        right: Value,
    },

    Select {
        result: Value,
        condition: Value,
        then_val: Value,
        else_val: Value,
    },

    Load {
        result: Value,
        location: Location,
    },
    Store {
        location: Location,
        value: Value,
    },
    Allocate {
        result: Value,
        ty: Type,
        size: Size,
    },
    Copy {
        dest: Location,
        src: Location,
        size: Value,
    },

    StorageLoad {
        result: Value,
        key: StorageKey,
    },
    StorageStore {
        key: StorageKey,
        value: Value,
    },
    StorageDelete {
        key: StorageKey,
    },

    MappingLoad {
        result: Value,
        mapping: Value,
        key: Value,
    },
    MappingStore {
        mapping: Value,
        key: Value,
        value: Value,
    },

    ArrayLoad {
        result: Value,
        array: Value,
        index: Value,
    },
    ArrayStore {
        array: Value,
        index: Value,
        value: Value,
    },
    ArrayLength {
        result: Value,
        array: Value,
    },
    ArrayPush {
        array: Value,
        value: Value,
    },
    ArrayPop {
        result: Value,
        array: Value,
    },

    Call {
        result: Value,
        target: CallTarget,
        args: Vec<Value>,
        value: Option<Value>,
    },
    DelegateCall {
        result: Value,
        target: Value,
        selector: Value,
        args: Vec<Value>,
    },
    StaticCall {
        result: Value,
        target: Value,
        selector: Value,
        args: Vec<Value>,
    },

    Create {
        result: Value,
        code: Value,
        value: Value,
    },
    Create2 {
        result: Value,
        code: Value,
        salt: Value,
        value: Value,
    },

    Selfdestruct {
        beneficiary: Value,
    },

    GetContext {
        result: Value,
        var: ContextVariable,
    },
    GetBalance {
        result: Value,
        address: Value,
    },
    GetCode {
        result: Value,
        address: Value,
    },
    GetCodeSize {
        result: Value,
        address: Value,
    },
    GetCodeHash {
        result: Value,
        address: Value,
    },

    Keccak256 {
        result: Value,
        data: Value,
        len: Value,
    },
    Sha256 {
        result: Value,
        data: Value,
        len: Value,
    },
    Ripemd160 {
        result: Value,
        data: Value,
        len: Value,
    },
    EcRecover {
        result: Value,
        hash: Value,
        v: Value,
        r: Value,
        s: Value,
    },

    EmitEvent {
        event: EventId,
        topics: Vec<Value>,
        data: Vec<Value>,
    },

    Cast {
        result: Value,
        value: Value,
        to: Type,
    },
    ZeroExtend {
        result: Value,
        value: Value,
        to: Type,
    },
    SignExtend {
        result: Value,
        value: Value,
        to: Type,
    },
    Truncate {
        result: Value,
        value: Value,
        to: Type,
    },

    Assert {
        condition: Value,
        message: String,
    },
    Require {
        condition: Value,
        message: String,
    },
    Revert {
        message: String,
    },

    Assign {
        result: Value,
        value: Value,
    },
    Phi {
        result: Value,
        values: Vec<(BlockId, Value)>,
    },

    Jump {
        target: BlockId,
        args: Vec<Value>,
    },
    Branch {
        condition: Value,
        then_block: BlockId,
        else_block: BlockId,
        then_args: Vec<Value>,
        else_args: Vec<Value>,
    },
    Return {
        value: Option<Value>,
    },

    MemoryAlloc {
        result: Value,
        size: Value,
    },
    MemoryCopy {
        dest: Value,
        src: Value,
        size: Value,
    },
    MemorySize {
        result: Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Size {
    Static(usize),
    Dynamic(Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageKey {
    Slot(BigUint),
    Dynamic(Value),
    Computed(Value),
    MappingKey { base: BigUint, key: Value },
    ArrayElement { base: BigUint, index: Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallTarget {
    Internal(String),
    External(Value),
    Library(String),
    Builtin(BuiltinFunction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuiltinFunction {
    AddMod,
    MulMod,
    BlockHash,
    GasLeft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextVariable {
    MsgSender,
    MsgValue,
    MsgData,
    MsgSig,
    BlockNumber,
    BlockTimestamp,
    BlockDifficulty,
    BlockGasLimit,
    BlockCoinbase,
    ChainId,
    BlockBaseFee,
    TxOrigin,
    TxGasPrice,
    GasLeft,
    ThisAddress,
    ThisBalance,
}

use crate::block::BlockId;

impl Instruction {
    pub fn result(&self) -> Option<&Value> {
        match self {
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
            | Instruction::Allocate { result, .. }
            | Instruction::StorageLoad { result, .. }
            | Instruction::MappingLoad { result, .. }
            | Instruction::ArrayLoad { result, .. }
            | Instruction::ArrayLength { result, .. }
            | Instruction::ArrayPop { result, .. }
            | Instruction::Call { result, .. }
            | Instruction::DelegateCall { result, .. }
            | Instruction::StaticCall { result, .. }
            | Instruction::Create { result, .. }
            | Instruction::Create2 { result, .. }
            | Instruction::GetContext { result, .. }
            | Instruction::GetBalance { result, .. }
            | Instruction::GetCode { result, .. }
            | Instruction::GetCodeSize { result, .. }
            | Instruction::GetCodeHash { result, .. }
            | Instruction::Keccak256 { result, .. }
            | Instruction::Sha256 { result, .. }
            | Instruction::Ripemd160 { result, .. }
            | Instruction::EcRecover { result, .. }
            | Instruction::Cast { result, .. }
            | Instruction::ZeroExtend { result, .. }
            | Instruction::SignExtend { result, .. }
            | Instruction::Truncate { result, .. }
            | Instruction::Assign { result, .. }
            | Instruction::Phi { result, .. }
            | Instruction::MemoryAlloc { result, .. }
            | Instruction::MemorySize { result, .. } => Some(result),
            _ => None,
        }
    }

    pub fn is_state_changing(&self) -> bool {
        matches!(
            self,
            Instruction::Store { .. }
                | Instruction::StorageStore { .. }
                | Instruction::StorageDelete { .. }
                | Instruction::MappingStore { .. }
                | Instruction::ArrayStore { .. }
                | Instruction::ArrayPush { .. }
                | Instruction::ArrayPop { .. }
                | Instruction::Call { .. }
                | Instruction::DelegateCall { .. }
                | Instruction::Create { .. }
                | Instruction::Create2 { .. }
                | Instruction::Selfdestruct { .. }
                | Instruction::EmitEvent { .. }
        )
    }

    pub fn is_external_call(&self) -> bool {
        matches!(
            self,
            Instruction::Call {
                target: CallTarget::External(_),
                ..
            } | Instruction::DelegateCall { .. }
                | Instruction::StaticCall { .. }
        )
    }

    pub fn is_external_call_with_value(&self) -> bool {
        matches!(
            self,
            Instruction::Call {
                target: CallTarget::External(_),
                value: Some(_),
                ..
            }
        )
    }

    pub fn can_revert(&self) -> bool {
        matches!(
            self,
            Instruction::Div { .. }
                | Instruction::Mod { .. }
                | Instruction::CheckedAdd { .. }
                | Instruction::CheckedSub { .. }
                | Instruction::CheckedMul { .. }
                | Instruction::Assert { .. }
                | Instruction::Require { .. }
                | Instruction::Call { .. }
                | Instruction::DelegateCall { .. }
                | Instruction::StaticCall { .. }
                | Instruction::Create { .. }
                | Instruction::Create2 { .. }
        )
    }
}
