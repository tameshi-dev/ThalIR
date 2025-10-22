use super::{
    inst_builder::{InstBuilder, InstBuilderBase, InstBuilderExt},
    IRContext, IRRegistry,
};
use crate::{
    block::{BasicBlock, BlockId, Terminator},
    contract::EventId,
    instructions::{CallTarget, ContextVariable, Instruction, StorageKey},
    types::Type,
    values::{Constant, SourceLocation, Value},
    Result,
};
use num_bigint::{BigInt, BigUint};
use std::collections::HashMap;

pub struct BlockBuilder<'a> {
    pub block_id: BlockId,
    function_name: String,
    instructions: Vec<Instruction>,
    context: &'a mut IRContext,
    registry: &'a mut IRRegistry,
    is_sealed: bool,
    current_source_location: Option<SourceLocation>,
    instruction_locations: HashMap<usize, SourceLocation>,
}

impl<'a> BlockBuilder<'a> {
    pub fn new(
        block_id: BlockId,
        function_name: String,
        context: &'a mut IRContext,
        registry: &'a mut IRRegistry,
    ) -> Self {
        Self {
            block_id,
            function_name,
            instructions: Vec::new(),
            context,
            registry,
            is_sealed: false,
            current_source_location: None,
            instruction_locations: HashMap::new(),
        }
    }

    pub fn set_source_location(&mut self, location: SourceLocation) {
        self.current_source_location = Some(location);
    }

    pub fn clear_source_location(&mut self) {
        self.current_source_location = None;
    }

    fn record_instruction_location(&mut self) {
        if let Some(ref location) = self.current_source_location {
            let index = self.instructions.len();
            self.instruction_locations.insert(index, location.clone());
        }
    }

    fn push_instruction(&mut self, inst: Instruction) {
        self.record_instruction_location();
        self.instructions.push(inst);
    }

    pub fn block_id(&self) -> BlockId {
        self.block_id
    }

    pub fn add(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Add {
            result: result.clone(),
            left,
            right,
            ty,
        });
        result
    }

    pub fn sub(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Sub {
            result: result.clone(),
            left,
            right,
            ty,
        });
        result
    }

    pub fn mul(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Mul {
            result: result.clone(),
            left,
            right,
            ty,
        });
        result
    }

    pub fn div(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Div {
            result: result.clone(),
            left,
            right,
            ty,
        });
        result
    }

    pub fn mod_(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Mod {
            result: result.clone(),
            left,
            right,
            ty,
        });
        result
    }

    pub fn pow(&mut self, base: Value, exp: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Pow {
            result: result.clone(),
            base,
            exp,
        });
        result
    }

    pub fn checked_add(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::CheckedAdd {
            result: result.clone(),
            left,
            right,
            ty,
        });
        result
    }

    pub fn checked_sub(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::CheckedSub {
            result: result.clone(),
            left,
            right,
            ty,
        });
        result
    }

    pub fn checked_mul(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::CheckedMul {
            result: result.clone(),
            left,
            right,
            ty,
        });
        result
    }

    pub fn and(&mut self, left: Value, right: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::And {
            result: result.clone(),
            left,
            right,
        });
        result
    }

    pub fn or(&mut self, left: Value, right: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Or {
            result: result.clone(),
            left,
            right,
        });
        result
    }

    pub fn xor(&mut self, left: Value, right: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Xor {
            result: result.clone(),
            left,
            right,
        });
        result
    }

    pub fn not(&mut self, operand: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Not {
            result: result.clone(),
            operand,
        });
        result
    }

    pub fn shl(&mut self, value: Value, shift: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Shl {
            result: result.clone(),
            value,
            shift,
        });
        result
    }

    pub fn shr(&mut self, value: Value, shift: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Shr {
            result: result.clone(),
            value,
            shift,
        });
        result
    }

    pub fn sar(&mut self, value: Value, shift: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Sar {
            result: result.clone(),
            value,
            shift,
        });
        result
    }

    pub fn eq(&mut self, left: Value, right: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Eq {
            result: result.clone(),
            left,
            right,
        });
        result
    }

    pub fn ne(&mut self, left: Value, right: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Ne {
            result: result.clone(),
            left,
            right,
        });
        result
    }

    pub fn lt(&mut self, left: Value, right: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Lt {
            result: result.clone(),
            left,
            right,
        });
        result
    }

    pub fn gt(&mut self, left: Value, right: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Gt {
            result: result.clone(),
            left,
            right,
        });
        result
    }

    pub fn le(&mut self, left: Value, right: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Le {
            result: result.clone(),
            left,
            right,
        });
        result
    }

    pub fn ge(&mut self, left: Value, right: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Ge {
            result: result.clone(),
            left,
            right,
        });
        result
    }

    pub fn select(&mut self, condition: Value, then_val: Value, else_val: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Select {
            result: result.clone(),
            condition,
            then_val,
            else_val,
        });
        result
    }

    pub fn allocate(&mut self, ty: Type, size: crate::instructions::Size) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Allocate {
            result: result.clone(),
            ty,
            size,
        });
        result
    }

    pub fn storage_load(&mut self, slot: BigUint) -> Value {
        let result = self.new_temp();
        let key = StorageKey::Slot(slot);
        self.push_instruction(Instruction::StorageLoad {
            result: result.clone(),
            key,
        });
        result
    }

    pub fn storage_store(&mut self, slot: BigUint, value: Value) {
        let key = StorageKey::Slot(slot);
        self.push_instruction(Instruction::StorageStore { key, value });
    }

    pub fn mapping_load(&mut self, mapping: Value, key: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::MappingLoad {
            result: result.clone(),
            mapping,
            key,
        });
        result
    }

    pub fn mapping_store(&mut self, mapping: Value, key: Value, value: Value) {
        self.push_instruction(Instruction::MappingStore {
            mapping,
            key,
            value,
        });
    }

    pub fn array_load(&mut self, array: Value, index: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::ArrayLoad {
            result: result.clone(),
            array,
            index,
        });
        result
    }

    pub fn array_store(&mut self, array: Value, index: Value, value: Value) {
        self.push_instruction(Instruction::ArrayStore {
            array,
            index,
            value,
        });
    }

    pub fn array_length(&mut self, array: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::ArrayLength {
            result: result.clone(),
            array,
        });
        result
    }

    pub fn array_push(&mut self, array: Value, value: Value) {
        self.push_instruction(Instruction::ArrayPush { array, value });
    }

    pub fn array_pop(&mut self, array: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::ArrayPop {
            result: result.clone(),
            array,
        });
        result
    }

    pub fn msg_sender(&mut self) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::MsgSender,
        });
        result
    }

    pub fn msg_value(&mut self) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::MsgValue,
        });
        result
    }

    pub fn block_number(&mut self) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::BlockNumber,
        });
        result
    }

    pub fn block_timestamp(&mut self) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::BlockTimestamp,
        });
        result
    }

    pub fn call_internal(&mut self, name: &str, args: Vec<Value>) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Call {
            result: result.clone(),
            target: CallTarget::Internal(name.to_string()),
            args,
            value: None,
        });
        result
    }

    pub fn call_external(
        &mut self,
        target: Value,
        selector: Value,
        args: Vec<Value>,
        value: Option<Value>,
    ) -> Value {
        let result = self.new_temp();

        let mut call_args = vec![selector];
        call_args.extend(args);
        self.push_instruction(Instruction::Call {
            result: result.clone(),
            target: CallTarget::External(target),
            args: call_args,
            value,
        });
        result
    }

    pub fn keccak256(&mut self, data: Value, len: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Keccak256 {
            result: result.clone(),
            data,
            len,
        });
        result
    }

    pub fn sha256(&mut self, data: Value, len: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Sha256 {
            result: result.clone(),
            data,
            len,
        });
        result
    }

    pub fn emit_event(&mut self, event_id: EventId, topics: Vec<Value>, data: Vec<Value>) {
        self.push_instruction(Instruction::EmitEvent {
            event: event_id,
            topics,
            data,
        });
    }

    pub fn cast(&mut self, value: Value, to: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Cast {
            result: result.clone(),
            value,
            to,
        });
        result
    }

    pub fn assert(&mut self, condition: Value, message: &str) {
        self.push_instruction(Instruction::Assert {
            condition,
            message: message.to_string(),
        });
    }

    pub fn require(&mut self, condition: Value, message: &str) {
        self.push_instruction(Instruction::Require {
            condition,
            message: message.to_string(),
        });
    }

    pub fn assign(&mut self, result: Value, value: Value) {
        self.push_instruction(Instruction::Assign { result, value });
    }

    pub fn phi(&mut self, values: Vec<(BlockId, Value)>) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Phi {
            result: result.clone(),
            values,
        });
        result
    }

    pub fn jump(&mut self, target: BlockId) -> Result<()> {
        self.seal_with_terminator(Terminator::Jump(target, Vec::new()))
    }

    pub fn branch(
        &mut self,
        condition: Value,
        then_block: BlockId,
        else_block: BlockId,
    ) -> Result<()> {
        self.seal_with_terminator(Terminator::Branch {
            condition,
            then_block,
            else_block,
            then_args: Vec::new(),
            else_args: Vec::new(),
        })
    }

    pub fn return_value(&mut self, value: Value) -> Result<()> {
        self.seal_with_terminator(Terminator::Return(Some(value)))
    }

    pub fn return_void(&mut self) -> Result<()> {
        self.seal_with_terminator(Terminator::Return(None))
    }

    pub fn revert(&mut self, message: &str) -> Result<()> {
        self.seal_with_terminator(Terminator::Revert(message.to_string()))
    }

    pub fn is_sealed(&self) -> bool {
        self.is_sealed
    }

    pub fn seal_with_terminator(&mut self, terminator: Terminator) -> Result<()> {
        if self.is_sealed {
            return Err(crate::IrError::BuilderError(format!(
                "Block {} already sealed",
                self.block_id
            )));
        }

        let mut block = BasicBlock::new(self.block_id);
        block.instructions = self.instructions.clone();
        block.terminator = terminator;

        block.metadata.instruction_locations = self.instruction_locations.clone();

        self.registry.add_block(self.function_name.clone(), block)?;
        self.is_sealed = true;
        Ok(())
    }

    pub fn new_temp(&mut self) -> Value {
        let temp_id = self.context.ssa().new_temp();
        Value::Temp(temp_id)
    }

    pub fn constant_uint(&self, value: u64, bits: u16) -> Value {
        Value::Constant(Constant::Uint(BigUint::from(value), bits))
    }

    pub fn constant_int(&self, value: i64, bits: u16) -> Value {
        Value::Constant(Constant::Int(BigInt::from(value), bits))
    }

    pub fn constant_bool(&self, value: bool) -> Value {
        Value::Constant(Constant::Bool(value))
    }

    pub fn constant_address(&self, bytes: [u8; 20]) -> Value {
        Value::Constant(Constant::Address(bytes))
    }
}

impl<'a> InstBuilderBase<'a> for BlockBuilder<'a> {
    fn new_temp(&mut self) -> Value {
        let temp_id = self.context.ssa().new_temp();
        Value::Temp(temp_id)
    }

    fn current_block(&self) -> BlockId {
        self.block_id
    }

    fn switch_to_block(&mut self, block: BlockId) {
        self.block_id = block;
        self.context.set_current_block(block);
    }
}

impl<'a> InstBuilderExt<'a> for BlockBuilder<'a> {
    fn storage_load(&mut self, slot: BigUint) -> Value {
        let result = self.new_temp();
        let key = StorageKey::Slot(slot);
        self.push_instruction(Instruction::StorageLoad {
            result: result.clone(),
            key,
        });
        result
    }

    fn storage_store(&mut self, slot: BigUint, value: Value) {
        let key = StorageKey::Slot(slot);
        self.push_instruction(Instruction::StorageStore { key, value });
    }

    fn storage_load_dynamic(&mut self, slot: Value) -> Value {
        let result = self.new_temp();
        let key = StorageKey::Dynamic(slot);
        self.push_instruction(Instruction::StorageLoad {
            result: result.clone(),
            key,
        });
        result
    }

    fn storage_store_dynamic(&mut self, slot: Value, value: Value) {
        let key = StorageKey::Dynamic(slot);
        self.push_instruction(Instruction::StorageStore { key, value });
    }

    fn mapping_load(&mut self, mapping: Value, key: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::MappingLoad {
            result: result.clone(),
            mapping,
            key,
        });
        result
    }

    fn mapping_store(&mut self, mapping: Value, key: Value, value: Value) {
        self.push_instruction(Instruction::MappingStore {
            mapping,
            key,
            value,
        });
    }

    fn array_load(&mut self, array: Value, index: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::ArrayLoad {
            result: result.clone(),
            array,
            index,
        });
        result
    }

    fn array_store(&mut self, array: Value, index: Value, value: Value) {
        self.push_instruction(Instruction::ArrayStore {
            array,
            index,
            value,
        });
    }

    fn array_length(&mut self, array: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::ArrayLength {
            result: result.clone(),
            array,
        });
        result
    }

    fn array_push(&mut self, array: Value, value: Value) {
        self.push_instruction(Instruction::ArrayPush { array, value });
    }

    fn array_pop(&mut self, array: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::ArrayPop {
            result: result.clone(),
            array,
        });
        result
    }

    fn msg_sender(&mut self) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::MsgSender,
        });
        result
    }

    fn msg_value(&mut self) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::MsgValue,
        });
        result
    }

    fn msg_data(&mut self) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::MsgData,
        });
        result
    }

    fn block_number(&mut self) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::BlockNumber,
        });
        result
    }

    fn block_timestamp(&mut self) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::BlockTimestamp,
        });
        result
    }

    fn block_difficulty(&mut self) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::BlockDifficulty,
        });
        result
    }

    fn block_gaslimit(&mut self) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::BlockGasLimit,
        });
        result
    }

    fn block_coinbase(&mut self) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::BlockCoinbase,
        });
        result
    }

    fn tx_origin(&mut self) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::TxOrigin,
        });
        result
    }

    fn tx_gasprice(&mut self) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::TxGasPrice,
        });
        result
    }

    fn gas_left(&mut self) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::GasLeft,
        });
        result
    }

    fn msg_sig(&mut self) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::MsgSig,
        });
        result
    }

    fn block_chainid(&mut self) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::ChainId,
        });
        result
    }

    fn block_basefee(&mut self) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::BlockBaseFee,
        });
        result
    }

    fn call_internal(&mut self, name: &str, args: Vec<Value>) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Call {
            result: result.clone(),
            target: CallTarget::Internal(name.to_string()),
            args,
            value: None,
        });
        result
    }

    fn call_external(
        &mut self,
        target: Value,
        selector: Value,
        args: Vec<Value>,
        value: Option<Value>,
    ) -> Value {
        self.call_external(target, selector, args, value)
    }

    fn delegate_call(&mut self, target: Value, selector: Value, args: Vec<Value>) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::DelegateCall {
            result: result.clone(),
            target,
            selector,
            args,
        });
        result
    }

    fn static_call(&mut self, target: Value, selector: Value, args: Vec<Value>) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::StaticCall {
            result: result.clone(),
            target,
            selector,
            args,
        });
        result
    }

    fn emit_event(&mut self, event: EventId, topics: Vec<Value>, data: Vec<Value>) {
        self.push_instruction(Instruction::EmitEvent {
            event,
            topics,
            data,
        });
    }

    fn keccak256(&mut self, data: Value, len: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Keccak256 {
            result: result.clone(),
            data,
            len,
        });
        result
    }

    fn sha256(&mut self, data: Value, len: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Sha256 {
            result: result.clone(),
            data,
            len,
        });
        result
    }

    fn ripemd160(&mut self, data: Value, len: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Ripemd160 {
            result: result.clone(),
            data,
            len,
        });
        result
    }

    fn ecrecover(&mut self, hash: Value, v: Value, r: Value, s: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::EcRecover {
            result: result.clone(),
            hash,
            v,
            r,
            s,
        });
        result
    }

    fn checked_add(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::CheckedAdd {
            result: result.clone(),
            left,
            right,
            ty,
        });
        result
    }

    fn checked_sub(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::CheckedSub {
            result: result.clone(),
            left,
            right,
            ty,
        });
        result
    }

    fn checked_mul(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::CheckedMul {
            result: result.clone(),
            left,
            right,
            ty,
        });
        result
    }

    fn checked_div(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::CheckedDiv {
            result: result.clone(),
            left,
            right,
            ty,
        });
        result
    }

    fn require(&mut self, condition: Value, message: &str) {
        self.push_instruction(Instruction::Require {
            condition,
            message: message.to_string(),
        });
    }

    fn assert(&mut self, condition: Value, message: &str) {
        self.push_instruction(Instruction::Assert {
            condition,
            message: message.to_string(),
        });
    }

    fn revert(&mut self, message: &str) {
        self.push_instruction(Instruction::Revert {
            message: message.to_string(),
        });
    }

    fn memory_alloc(&mut self, size: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::MemoryAlloc {
            result: result.clone(),
            size,
        });
        result
    }

    fn memory_copy(&mut self, dest: Value, src: Value, size: Value) {
        self.push_instruction(Instruction::MemoryCopy { dest, src, size });
    }

    fn memory_size(&mut self) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::MemorySize {
            result: result.clone(),
        });
        result
    }

    fn cast(&mut self, value: Value, to: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Cast {
            result: result.clone(),
            value,
            to,
        });
        result
    }

    fn zext(&mut self, value: Value, to: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::ZeroExtend {
            result: result.clone(),
            value,
            to,
        });
        result
    }

    fn sext(&mut self, value: Value, to: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::SignExtend {
            result: result.clone(),
            value,
            to,
        });
        result
    }

    fn trunc(&mut self, value: Value, to: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Truncate {
            result: result.clone(),
            value,
            to,
        });
        result
    }
}

impl<'a> InstBuilder<'a> for BlockBuilder<'a> {
    fn add(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Add {
            result: result.clone(),
            left,
            right,
            ty,
        });
        result
    }

    fn sub(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Sub {
            result: result.clone(),
            left,
            right,
            ty,
        });
        result
    }

    fn mul(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Mul {
            result: result.clone(),
            left,
            right,
            ty,
        });
        result
    }

    fn div(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Div {
            result: result.clone(),
            left,
            right,
            ty,
        });
        result
    }

    fn mod_(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Mod {
            result: result.clone(),
            left,
            right,
            ty,
        });
        result
    }

    fn pow(&mut self, base: Value, exp: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Pow {
            result: result.clone(),
            base,
            exp,
        });
        result
    }

    fn and(&mut self, left: Value, right: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::And {
            result: result.clone(),
            left,
            right,
        });
        result
    }

    fn or(&mut self, left: Value, right: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Or {
            result: result.clone(),
            left,
            right,
        });
        result
    }

    fn xor(&mut self, left: Value, right: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Xor {
            result: result.clone(),
            left,
            right,
        });
        result
    }

    fn not(&mut self, operand: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Not {
            result: result.clone(),
            operand,
        });
        result
    }

    fn shl(&mut self, value: Value, shift: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Shl {
            result: result.clone(),
            value,
            shift,
        });
        result
    }

    fn shr(&mut self, value: Value, shift: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Shr {
            result: result.clone(),
            value,
            shift,
        });
        result
    }

    fn sar(&mut self, value: Value, shift: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Sar {
            result: result.clone(),
            value,
            shift,
        });
        result
    }

    fn eq(&mut self, left: Value, right: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Eq {
            result: result.clone(),
            left,
            right,
        });
        result
    }

    fn ne(&mut self, left: Value, right: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Ne {
            result: result.clone(),
            left,
            right,
        });
        result
    }

    fn lt(&mut self, left: Value, right: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Lt {
            result: result.clone(),
            left,
            right,
        });
        result
    }

    fn gt(&mut self, left: Value, right: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Gt {
            result: result.clone(),
            left,
            right,
        });
        result
    }

    fn le(&mut self, left: Value, right: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Le {
            result: result.clone(),
            left,
            right,
        });
        result
    }

    fn ge(&mut self, left: Value, right: Value) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Ge {
            result: result.clone(),
            left,
            right,
        });
        result
    }

    fn jump(&mut self, target: BlockId, args: Vec<Value>) {
        self.push_instruction(Instruction::Jump { target, args });
    }

    fn branch(
        &mut self,
        condition: Value,
        then_block: BlockId,
        else_block: BlockId,
        then_args: Vec<Value>,
        else_args: Vec<Value>,
    ) {
        self.push_instruction(Instruction::Branch {
            condition,
            then_block,
            else_block,
            then_args,
            else_args,
        });
    }

    fn return_value(&mut self, value: Option<Value>) {
        self.push_instruction(Instruction::Return { value });
    }

    fn assign(&mut self, dest: Value, src: Value) {
        self.push_instruction(Instruction::Assign {
            result: dest,
            value: src,
        });
    }

    fn phi(&mut self, values: Vec<(BlockId, Value)>) -> Value {
        let result = self.new_temp();
        self.push_instruction(Instruction::Phi {
            result: result.clone(),
            values,
        });
        result
    }
}
