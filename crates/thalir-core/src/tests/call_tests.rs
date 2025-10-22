use crate::builder::{IRBuilder, InstBuilderExt};
use crate::types::Type;

#[test]
fn test_internal_call() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("CallContract");

    let mut helper = contract.function("helper");
    helper.param("x", Type::Uint(256)).returns(Type::Uint(256));

    let x = helper.get_param(0);
    let mut helper_entry = helper.entry_block();
    let two = helper_entry.constant_uint(2, 256);
    let doubled = helper_entry.mul(x, two, Type::Uint(256));
    helper_entry.return_value(doubled);
    helper.build().unwrap();

    let mut main = contract.function("main");
    main.param("value", Type::Uint(256))
        .returns(Type::Uint(256));

    let value = main.get_param(0);
    let mut entry = main.entry_block();

    let result = entry.call_internal("helper", vec![value]);

    entry.return_value(result);
    main.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_external_call() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("ExternalCallContract");

    let mut func = contract.function("callExternal");
    func.param("target", Type::Address)
        .param("selector", Type::Bytes4)
        .param("data", Type::Bytes(32))
        .returns(Type::Bool);

    let target = func.get_param(0);
    let selector = func.get_param(1);
    let data = func.get_param(2);
    let mut entry = func.entry_block();

    let result = entry.call_external(target, selector, vec![data], None);

    let zero = entry.constant_uint(0, 256);
    let success = entry.ne(result, zero);

    entry.return_value(success);
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_payable_call() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("PayableCallContract");

    let mut func = contract.function("sendValue");
    func.param("recipient", Type::Address)
        .param("amount", Type::Uint(256))
        .returns(Type::Bool);

    let recipient = func.get_param(0);
    let amount = func.get_param(1);
    let mut entry = func.entry_block();

    let empty_selector = entry.constant_uint(0, 32);

    let result = entry.call_external(recipient, empty_selector, vec![], Some(amount));

    let zero = entry.constant_uint(0, 256);
    let success = entry.ne(result, zero);

    entry.return_value(success);
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_delegate_call() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("DelegateCallContract");

    let mut func = contract.function("delegate");
    func.param("implementation", Type::Address)
        .param("selector", Type::Bytes4)
        .param("data", Type::Bytes(32))
        .returns(Type::Bytes(32));

    let implementation = func.get_param(0);
    let selector = func.get_param(1);
    let data = func.get_param(2);
    let mut entry = func.entry_block();

    let result = entry.delegate_call(implementation, selector, vec![data]);

    entry.return_value(result);
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_static_call() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("StaticCallContract");

    let mut func = contract.function("readOnly");
    func.param("target", Type::Address)
        .param("selector", Type::Bytes4)
        .returns(Type::Uint(256));

    let target = func.get_param(0);
    let selector = func.get_param(1);
    let mut entry = func.entry_block();

    let result = entry.static_call(target, selector, vec![]);

    entry.return_value(result);
    func.build().unwrap();
    contract.build().unwrap();
}
