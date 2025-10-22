use crate::builder::{IRBuilder, InstBuilderExt};
use crate::function::{Mutability, Visibility};
use crate::types::Type;
use num_bigint::BigUint;

#[test]
fn test_contract_creation() {
    let mut builder = IRBuilder::new();

    let mut contract = builder.contract("SimpleToken");

    contract.state_variable("totalSupply", Type::Uint(256), 0);
    contract.state_variable("owner", Type::Address, 1);
    contract.state_variable(
        "balances",
        Type::Mapping(Box::new(Type::Address), Box::new(Type::Uint(256))),
        2,
    );

    let transfer_event = contract
        .event("Transfer")
        .indexed("from", Type::Address)
        .indexed("to", Type::Address)
        .data("amount", Type::Uint(256))
        .build();
    contract.add_event(transfer_event);

    let approval_event = contract
        .event("Approval")
        .indexed("owner", Type::Address)
        .indexed("spender", Type::Address)
        .data("amount", Type::Uint(256))
        .build();
    contract.add_event(approval_event);

    contract.build().unwrap();

    let registry = builder.registry();
    let token = registry.get_contract("SimpleToken").unwrap();

    assert_eq!(token.name, "SimpleToken");
    assert_eq!(token.storage_layout.slots.len(), 3);
    assert_eq!(token.events.len(), 2);
}

#[test]
fn test_constructor() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("InitializableContract");

    contract.state_variable("initialized", Type::Bool, 0);
    contract.state_variable("initialValue", Type::Uint(256), 1);

    let mut constructor = contract.function("constructor");
    constructor
        .param("_initialValue", Type::Uint(256))
        .visibility(Visibility::Public);

    let initial_value = constructor.get_param(0);
    let mut entry = constructor.entry_block();

    let true_val = entry.constant_bool(true);
    entry.storage_store(BigUint::from(0u32), true_val);

    entry.storage_store(BigUint::from(1u32), initial_value);

    entry.return_void().unwrap();
    constructor.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_modifiers() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("OwnableContract");

    contract.state_variable("owner", Type::Address, 0);

    let mut only_owner = contract.function("onlyOwner");
    only_owner.visibility(Visibility::Internal);

    let mut entry = only_owner.entry_block();

    let stored_owner = entry.storage_load(BigUint::from(0u32));
    let sender = entry.msg_sender();
    let is_owner = entry.eq(sender, stored_owner);

    entry.require(is_owner, "Not the owner");
    entry.return_void().unwrap();

    only_owner.build().unwrap();

    let mut restricted = contract.function("restrictedFunction");
    restricted
        .modifier("onlyOwner")
        .visibility(Visibility::Public);

    let mut entry = restricted.entry_block();

    let modifier_check = entry.call_internal("onlyOwner", vec![]);

    entry.return_void().unwrap();

    restricted.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_payable_function() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("PayableContract");

    contract.state_variable("balance", Type::Uint(256), 0);

    let mut receive = contract.function("receive");
    receive
        .mutability(Mutability::Payable)
        .visibility(Visibility::External);

    let mut entry = receive.entry_block();

    let value = entry.msg_value();

    let current_balance = entry.storage_load(BigUint::from(0u32));

    let new_balance = entry.add(current_balance, value, Type::Uint(256));

    entry.storage_store(BigUint::from(0u32), new_balance);

    entry.return_void().unwrap();
    receive.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_view_function() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("ViewContract");

    contract.state_variable("data", Type::Uint(256), 0);

    let mut get_data = contract.function("getData");
    get_data
        .mutability(Mutability::View)
        .visibility(Visibility::External)
        .returns(Type::Uint(256));

    let mut entry = get_data.entry_block();

    let data = entry.storage_load(BigUint::from(0u32));

    entry.return_value(data);

    get_data.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_multiple_functions() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("MultiFunction");

    contract.state_variable("counter", Type::Uint(256), 0);

    let mut increment = contract.function("increment");
    increment.visibility(Visibility::Public);

    let mut inc_entry = increment.entry_block();
    let current = inc_entry.storage_load(BigUint::from(0u32));
    let one = inc_entry.constant_uint(1, 256);
    let incremented = inc_entry.add(current, one, Type::Uint(256));
    inc_entry.storage_store(BigUint::from(0u32), incremented);
    inc_entry.return_void().unwrap();
    increment.build().unwrap();

    let mut decrement = contract.function("decrement");
    decrement.visibility(Visibility::Public);

    let mut dec_entry = decrement.entry_block();
    let current = dec_entry.storage_load(BigUint::from(0u32));
    let one = dec_entry.constant_uint(1, 256);
    let decremented = dec_entry.sub(current, one, Type::Uint(256));
    dec_entry.storage_store(BigUint::from(0u32), decremented);
    dec_entry.return_void().unwrap();
    decrement.build().unwrap();

    let mut get = contract.function("get");
    get.visibility(Visibility::Public)
        .mutability(Mutability::View)
        .returns(Type::Uint(256));

    let mut get_entry = get.entry_block();
    let value = get_entry.storage_load(BigUint::from(0u32));
    get_entry.return_value(value);
    get.build().unwrap();

    contract.build().unwrap();

    let registry = builder.registry();
    assert!(registry
        .get_function_in_contract("MultiFunction", "increment")
        .is_some());
    assert!(registry
        .get_function_in_contract("MultiFunction", "decrement")
        .is_some());
    assert!(registry
        .get_function_in_contract("MultiFunction", "get")
        .is_some());
}
