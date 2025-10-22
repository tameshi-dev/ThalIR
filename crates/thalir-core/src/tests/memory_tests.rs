use crate::builder::{IRBuilder, InstBuilder, InstBuilderExt};
use crate::types::Type;

#[test]
fn test_memory_allocation() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("MemoryContract");

    let mut func = contract.function("allocate");
    func.param("size", Type::Uint(256)).returns(Type::Uint(256));

    let size = func.get_param(0);
    let mut entry = func.entry_block();

    let ptr = entry.memory_alloc(size);

    entry.return_value(ptr);
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_memory_copy() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("MemCopyContract");

    let mut func = contract.function("copyData");
    func.param("src", Type::Uint(256))
        .param("dst", Type::Uint(256))
        .param("len", Type::Uint(256));

    let src = func.get_param(0);
    let dst = func.get_param(1);
    let len = func.get_param(2);
    let mut entry = func.entry_block();

    entry.memory_copy(dst, src, len);

    entry.return_void().unwrap();
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_memory_size() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("MemSizeContract");

    let mut func = contract.function("getMemorySize");
    func.returns(Type::Uint(256));

    let mut entry = func.entry_block();

    let size = entry.memory_size();

    entry.return_value(size);
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_memory_operations_combined() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("MemOpsContract");

    let mut func = contract.function("processData");
    func.param("input_size", Type::Uint(256))
        .returns(Type::Uint(256));

    let input_size = func.get_param(0);
    let mut entry = func.entry_block();

    let initial_size = entry.memory_size();

    let input_buffer = entry.memory_alloc(input_size.clone());

    let two = entry.constant_uint(2, 256);
    let output_size = entry.mul(input_size.clone(), two, Type::Uint(256));
    let output_buffer = entry.memory_alloc(output_size.clone());

    entry.memory_copy(output_buffer.clone(), input_buffer, input_size);

    let final_size = entry.memory_size();

    let growth = entry.sub(final_size, initial_size, Type::Uint(256));

    entry.return_value(growth);
    func.build().unwrap();
    contract.build().unwrap();
}
