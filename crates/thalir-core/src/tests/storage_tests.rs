use crate::builder::{IRBuilder, InstBuilderExt};
use crate::types::Type;
use num_bigint::BigUint;

#[test]
fn test_storage_load_store() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("StorageContract");

    contract.state_variable("balance", Type::Uint(256), 0);
    contract.state_variable("owner", Type::Address, 1);
    contract.state_variable("isActive", Type::Bool, 2);

    let mut func = contract.function("update_storage");
    func.param("new_balance", Type::Uint(256))
        .param("new_owner", Type::Address);

    let new_balance = func.get_param(0);
    let new_owner = func.get_param(1);
    let mut entry = func.entry_block();

    let old_balance = entry.storage_load(BigUint::from(0u32));
    let old_owner = entry.storage_load(BigUint::from(1u32));

    entry.storage_store(BigUint::from(0u32), new_balance);
    entry.storage_store(BigUint::from(1u32), new_owner);

    let true_val = entry.constant_bool(true);
    entry.storage_store(BigUint::from(2u32), true_val);

    entry.return_void().unwrap();
    func.build().unwrap();
    contract.build().unwrap();

    let registry = builder.registry();
    assert!(registry.get_contract("StorageContract").is_some());
}

#[test]
fn test_dynamic_storage() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("DynamicStorageContract");

    let mut func = contract.function("dynamic_storage");
    func.param("slot", Type::Uint(256))
        .param("value", Type::Uint(256));

    let slot = func.get_param(0);
    let value = func.get_param(1);
    let mut entry = func.entry_block();

    let old_value = entry.storage_load_dynamic(slot.clone());
    entry.storage_store_dynamic(slot, value);

    entry.return_value(old_value).unwrap();
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_mapping_operations() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("MappingContract");

    contract.state_variable(
        "balances",
        Type::Mapping(Box::new(Type::Address), Box::new(Type::Uint(256))),
        0,
    );

    let mut func = contract.function("update_balance");
    func.param("user", Type::Address)
        .param("amount", Type::Uint(256))
        .returns(Type::Uint(256));

    let user = func.get_param(0);
    let amount = func.get_param(1);
    let mut entry = func.entry_block();

    let mapping_base = entry.constant_uint(0, 256);

    let old_balance = entry.mapping_load(mapping_base.clone(), user.clone());

    entry.mapping_store(mapping_base, user, amount);

    entry.return_value(old_balance).unwrap();
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_array_operations() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("ArrayContract");

    contract.state_variable("numbers", Type::Array(Box::new(Type::Uint(256)), None), 0);

    let mut func = contract.function("array_ops");
    func.param("index", Type::Uint(256))
        .param("value", Type::Uint(256))
        .returns(Type::Uint(256));

    let index = func.get_param(0);
    let value = func.get_param(1);
    let mut entry = func.entry_block();

    let array_base = entry.constant_uint(0, 256);

    let length = entry.array_length(array_base.clone());

    let old_value = entry.array_load(array_base.clone(), index.clone());

    entry.array_store(array_base.clone(), index, value.clone());

    entry.array_push(array_base.clone(), value);

    let popped = entry.array_pop(array_base);

    entry.return_value(popped).unwrap();
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_packed_storage() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("PackedStorageContract");

    contract.state_variable("flag1", Type::Bool, 0);
    contract.state_variable("flag2", Type::Bool, 0);
    contract.state_variable("counter", Type::Uint(16), 0);

    let mut func = contract.function("update_packed");
    func.param("f1", Type::Bool)
        .param("f2", Type::Bool)
        .param("cnt", Type::Uint(16));

    let f1 = func.get_param(0);
    let f2 = func.get_param(1);
    let cnt = func.get_param(2);
    let mut entry = func.entry_block();

    let slot_value = entry.storage_load(BigUint::from(0u32));

    entry.storage_store(BigUint::from(0u32), slot_value);

    entry.return_void().unwrap();
    func.build().unwrap();
    contract.build().unwrap();
}
