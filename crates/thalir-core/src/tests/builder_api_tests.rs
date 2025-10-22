use crate::block::BlockId;
use crate::builder::{FunctionCursor, IRBuilder, IRContext};
use crate::types::Type;

#[test]
fn test_builder_creation() {
    let builder = IRBuilder::new();

    let stats = builder.stats();
    assert_eq!(stats.contracts, 0);
    assert_eq!(stats.functions, 0);
    assert_eq!(stats.blocks, 0);
    assert_eq!(stats.instructions, 0);
    assert_eq!(stats.values, 0);
}

#[test]
fn test_context_tracking() {
    let mut builder = IRBuilder::new();
    let context = builder.context();

    assert!(context.current_contract().is_none());
    assert!(context.current_function().is_none());
    assert!(context.current_block().is_none());

    let mut contract = builder.contract("TestContract");

    let mut func = contract.function("testFunc");

    let entry = func.entry_block();
}

#[test]
fn test_registry_operations() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("RegistryTest");

    let mut func = contract.function("test");
    func.returns(Type::Bool);

    let mut entry = func.entry_block();
    let true_val = entry.constant_bool(true);
    entry.return_value(true_val);

    func.build().unwrap();
    contract.build().unwrap();

    let registry = builder.registry();

    let contract = registry.get_contract("RegistryTest").unwrap();
    assert_eq!(contract.name, "RegistryTest");

    let function = registry
        .get_function_in_contract("RegistryTest", "test")
        .unwrap();
    assert_eq!(function.signature.name, "test");

    let qualified_func = registry.get_function("RegistryTest::test").unwrap();
    assert_eq!(qualified_func.signature.name, "test");
}

#[test]
fn test_cursor_navigation() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("CursorTest");
    let mut func = contract.function("test");

    let block1_id = func.create_block_id();
    let block2_id = func.create_block_id();
    let entry_id = BlockId(0);

    {
        let entry = func.entry_block();
    }

    let mut blocks = std::collections::HashMap::new();
    blocks.insert(entry_id, crate::block::BasicBlock::new(entry_id));
    blocks.insert(block1_id, crate::block::BasicBlock::new(block1_id));
    blocks.insert(block2_id, crate::block::BasicBlock::new(block2_id));

    let mut context = IRContext::new();
    let mut cursor = FunctionCursor::new(&mut context, &mut blocks);

    cursor.goto_block_start(entry_id);
    assert_eq!(cursor.current_block(), Some(entry_id));

    cursor.goto_block_end(block1_id);
    assert_eq!(cursor.current_block(), Some(block1_id));

    let new_block_id = cursor.create_block("new_block".to_string());
    assert_eq!(cursor.current_block(), Some(new_block_id));
}

#[test]
fn test_ssa_tracking() {
    let mut context = IRContext::new();
    let ssa = context.ssa();

    let var1 = ssa.get_or_create_var("x");
    let var2 = ssa.get_or_create_var("y");
    let var1_again = ssa.get_or_create_var("x");

    assert_eq!(var1, var1_again);
    assert_ne!(var1, var2);

    let temp1 = ssa.new_temp();
    let temp2 = ssa.new_temp();

    assert_ne!(temp1, temp2);
}

#[test]
fn test_builder_validation() {
    let mut builder = IRBuilder::new();

    let mut contract = builder.contract("ValidContract");
    let mut func = contract.function("valid");
    let mut entry = func.entry_block();
    entry.return_void().unwrap();
    func.build().unwrap();
    contract.build().unwrap();

    assert!(builder.validate().is_ok());

    let stats = builder.stats();
    assert_eq!(stats.contracts, 1);
    assert_eq!(stats.functions, 1);
    assert!(stats.blocks > 0);
}

#[test]
fn test_builder_clear() {
    let mut builder = IRBuilder::new();

    let mut contract = builder.contract("TempContract");
    let mut func = contract.function("temp");
    let entry = func.entry_block();
    func.build().unwrap();
    contract.build().unwrap();

    let stats = builder.stats();
    assert!(stats.contracts > 0);

    builder.clear();

    let stats = builder.stats();
    assert_eq!(stats.contracts, 0);
    assert_eq!(stats.functions, 0);
    assert_eq!(stats.blocks, 0);
}

#[test]
fn test_multiple_contracts() {
    let mut builder = IRBuilder::new();

    let mut contract1 = builder.contract("Contract1");
    let mut func1 = contract1.function("func1");
    func1.entry_block().return_void().unwrap();
    func1.build().unwrap();
    contract1.build().unwrap();

    let mut contract2 = builder.contract("Contract2");
    let mut func2 = contract2.function("func2");
    func2.entry_block().return_void().unwrap();
    func2.build().unwrap();
    contract2.build().unwrap();

    let registry = builder.registry();
    assert!(registry.get_contract("Contract1").is_some());
    assert!(registry.get_contract("Contract2").is_some());

    assert!(registry.get_function("Contract1::func1").is_some());
    assert!(registry.get_function("Contract2::func2").is_some());
    assert!(registry.get_function("Contract1::func2").is_none());
}
