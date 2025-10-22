#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_must_use)]

use num_bigint::BigUint;
use thalir_core::{
    builder::{IRBuilder, InstBuilder, InstBuilderExt},
    types::Type,
    values::Value,
};

#[test]
fn test_contract_creation() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("TestContract");
    contract.build().unwrap();

    let registry = builder.registry();
    let test_contract = registry.get_contract("TestContract").unwrap();

    assert_eq!(test_contract.name, "TestContract");
    assert_eq!(test_contract.functions.len(), 0);
    assert_eq!(test_contract.storage_layout.slots.len(), 0);
}

#[test]
fn test_storage_variables() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("StorageTest");

    contract.state_variable("value1", Type::Uint(256), 0);
    contract.state_variable("value2", Type::Address, 1);

    contract.build().unwrap();

    let registry = builder.registry();
    let storage_contract = registry.get_contract("StorageTest").unwrap();
    assert_eq!(storage_contract.storage_layout.slots.len(), 2);
}

#[test]
fn test_function_builder() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("FunctionTest");

    let mut func = contract.function("testFunc");
    func.returns(Type::Bool);

    let mut entry = func.entry_block();
    let true_val = entry.constant_bool(true);
    entry.return_value(true_val);

    func.build().unwrap();
    contract.build().unwrap();

    let registry = builder.registry();
    let test_contract = registry.get_contract("FunctionTest").unwrap();
    assert_eq!(test_contract.functions.len(), 1);
    assert!(test_contract.functions.contains_key("testFunc"));
}

#[test]
fn test_arithmetic_operations() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("ArithmeticTest");

    let mut func = contract.function("add");
    func.param("a", Type::Uint(256))
        .param("b", Type::Uint(256))
        .returns(Type::Uint(256));

    let a = func.get_param(0);
    let b = func.get_param(1);

    let mut entry = func.entry_block();
    let result = entry.add(a, b, Type::Uint(256));
    entry.return_value(result);

    func.build().unwrap();
    contract.build().unwrap();

    let registry = builder.registry();
    let arithmetic_contract = registry.get_contract("ArithmeticTest").unwrap();
    let function = arithmetic_contract.functions.get("add").unwrap();
    assert_eq!(function.signature.params.len(), 2);
    assert_eq!(function.signature.returns.len(), 1);
}

#[test]
fn test_control_flow() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("ControlFlowTest");

    let mut func = contract.function("checkValue");
    func.param("x", Type::Uint(256)).returns(Type::Bool);

    let x = func.get_param(0);

    let then_block_id = func.create_block_id();
    let else_block_id = func.create_block_id();
    let merge_block_id = func.create_block_id();

    {
        let mut entry = func.entry_block();
        let ten = entry.constant_uint(10, 256);
        let condition = entry.eq(x, ten);
        entry.branch(condition, then_block_id, else_block_id);
    }

    let true_result = {
        let mut then_block = func.block_with_id(then_block_id);
        let true_val = then_block.constant_bool(true);
        then_block.jump(merge_block_id);
        true_val
    };

    let false_result = {
        let mut else_block = func.block_with_id(else_block_id);
        let false_val = else_block.constant_bool(false);
        else_block.jump(merge_block_id);
        false_val
    };

    {
        let mut merge_block = func.block_with_id(merge_block_id);
        let result = merge_block.phi(vec![
            (then_block_id, true_result),
            (else_block_id, false_result),
        ]);
        merge_block.return_value(result);
    }

    func.build().unwrap();
    contract.build().unwrap();

    let registry = builder.registry();
    let control_contract = registry.get_contract("ControlFlowTest").unwrap();
    let function = control_contract.functions.get("checkValue").unwrap();
    assert_eq!(function.body.blocks.len(), 4);
}

#[test]
fn test_context_access() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("ContextTest");

    let mut func = contract.function("getSender");
    func.returns(Type::Address);

    let mut entry = func.entry_block();
    let sender = entry.msg_sender();
    entry.return_value(sender);

    func.build().unwrap();
    contract.build().unwrap();

    let registry = builder.registry();
    let context_contract = registry.get_contract("ContextTest").unwrap();
    assert!(context_contract.functions.contains_key("getSender"));
}
