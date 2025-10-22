use crate::{block::BlockId, contract::EventId, types::Type, values::Value};
use num_bigint::BigUint;

pub trait InstBuilderBase<'f>: Sized {
    fn new_temp(&mut self) -> Value;

    fn current_block(&self) -> BlockId;

    fn switch_to_block(&mut self, block: BlockId);
}

pub trait InstBuilderExt<'f>: InstBuilderBase<'f> {
    fn storage_load(&mut self, slot: BigUint) -> Value;

    fn storage_store(&mut self, slot: BigUint, value: Value);

    fn storage_load_dynamic(&mut self, slot: Value) -> Value;

    fn storage_store_dynamic(&mut self, slot: Value, value: Value);

    fn mapping_load(&mut self, mapping: Value, key: Value) -> Value;

    fn mapping_store(&mut self, mapping: Value, key: Value, value: Value);

    fn array_load(&mut self, array: Value, index: Value) -> Value;

    fn array_store(&mut self, array: Value, index: Value, value: Value);

    fn array_length(&mut self, array: Value) -> Value;

    fn array_push(&mut self, array: Value, value: Value);

    fn array_pop(&mut self, array: Value) -> Value;

    fn msg_sender(&mut self) -> Value;

    fn msg_value(&mut self) -> Value;

    fn msg_data(&mut self) -> Value;

    fn block_number(&mut self) -> Value;

    fn block_timestamp(&mut self) -> Value;

    fn block_difficulty(&mut self) -> Value;

    fn block_gaslimit(&mut self) -> Value;

    fn block_coinbase(&mut self) -> Value;

    fn tx_origin(&mut self) -> Value;

    fn tx_gasprice(&mut self) -> Value;

    fn msg_sig(&mut self) -> Value;

    fn block_chainid(&mut self) -> Value;

    fn block_basefee(&mut self) -> Value;

    fn gas_left(&mut self) -> Value;

    fn call_internal(&mut self, name: &str, args: Vec<Value>) -> Value;

    fn call_external(
        &mut self,
        target: Value,
        selector: Value,
        args: Vec<Value>,
        value: Option<Value>,
    ) -> Value;

    fn delegate_call(&mut self, target: Value, selector: Value, args: Vec<Value>) -> Value;

    fn static_call(&mut self, target: Value, selector: Value, args: Vec<Value>) -> Value;

    fn emit_event(&mut self, event: EventId, topics: Vec<Value>, data: Vec<Value>);

    fn keccak256(&mut self, data: Value, len: Value) -> Value;

    fn sha256(&mut self, data: Value, len: Value) -> Value;

    fn ripemd160(&mut self, data: Value, len: Value) -> Value;

    fn ecrecover(&mut self, hash: Value, v: Value, r: Value, s: Value) -> Value;

    fn checked_add(&mut self, left: Value, right: Value, ty: Type) -> Value;

    fn checked_sub(&mut self, left: Value, right: Value, ty: Type) -> Value;

    fn checked_mul(&mut self, left: Value, right: Value, ty: Type) -> Value;

    fn checked_div(&mut self, left: Value, right: Value, ty: Type) -> Value;

    fn require(&mut self, condition: Value, message: &str);

    fn assert(&mut self, condition: Value, message: &str);

    fn revert(&mut self, message: &str);

    fn memory_alloc(&mut self, size: Value) -> Value;

    fn memory_copy(&mut self, dest: Value, src: Value, size: Value);

    fn memory_size(&mut self) -> Value;

    fn cast(&mut self, value: Value, to: Type) -> Value;

    fn zext(&mut self, value: Value, to: Type) -> Value;

    fn sext(&mut self, value: Value, to: Type) -> Value;

    fn trunc(&mut self, value: Value, to: Type) -> Value;
}

pub trait InstBuilder<'f>: InstBuilderBase<'f> {
    fn add(&mut self, left: Value, right: Value, ty: Type) -> Value;

    fn sub(&mut self, left: Value, right: Value, ty: Type) -> Value;

    fn mul(&mut self, left: Value, right: Value, ty: Type) -> Value;

    fn div(&mut self, left: Value, right: Value, ty: Type) -> Value;

    fn mod_(&mut self, left: Value, right: Value, ty: Type) -> Value;

    fn pow(&mut self, base: Value, exp: Value) -> Value;

    fn and(&mut self, left: Value, right: Value) -> Value;

    fn or(&mut self, left: Value, right: Value) -> Value;

    fn xor(&mut self, left: Value, right: Value) -> Value;

    fn not(&mut self, operand: Value) -> Value;

    fn shl(&mut self, value: Value, shift: Value) -> Value;

    fn shr(&mut self, value: Value, shift: Value) -> Value;

    fn sar(&mut self, value: Value, shift: Value) -> Value;

    fn eq(&mut self, left: Value, right: Value) -> Value;

    fn ne(&mut self, left: Value, right: Value) -> Value;

    fn lt(&mut self, left: Value, right: Value) -> Value;

    fn gt(&mut self, left: Value, right: Value) -> Value;

    fn le(&mut self, left: Value, right: Value) -> Value;

    fn ge(&mut self, left: Value, right: Value) -> Value;

    fn jump(&mut self, target: BlockId, args: Vec<Value>);

    fn branch(
        &mut self,
        condition: Value,
        then_block: BlockId,
        else_block: BlockId,
        then_args: Vec<Value>,
        else_args: Vec<Value>,
    );

    fn return_value(&mut self, value: Option<Value>);

    fn assign(&mut self, dest: Value, src: Value);

    fn phi(&mut self, values: Vec<(BlockId, Value)>) -> Value;
}
