use crate::builder::{IRBuilder, InstBuilder, InstBuilderExt};
use crate::types::Type;

#[test]
fn test_type_casting() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("TypeContract");

    let mut func = contract.function("cast");
    func.param("value", Type::Uint(256))
        .returns(Type::Uint(128));

    let value = func.get_param(0);
    let mut entry = func.entry_block();

    let casted = entry.cast(value, Type::Uint(128));

    entry.return_value(casted);
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_zero_extend() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("ZextContract");

    let mut func = contract.function("zeroExtend");
    func.param("small", Type::Uint(8)).returns(Type::Uint(256));

    let small = func.get_param(0);
    let mut entry = func.entry_block();

    let extended = entry.zext(small, Type::Uint(256));

    entry.return_value(extended);
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_sign_extend() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("SextContract");

    let mut func = contract.function("signExtend");
    func.param("value", Type::Int(8)).returns(Type::Int(256));

    let value = func.get_param(0);
    let mut entry = func.entry_block();

    let extended = entry.sext(value, Type::Int(256));

    entry.return_value(extended);
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_truncate() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("TruncContract");

    let mut func = contract.function("truncate");
    func.param("large", Type::Uint(256)).returns(Type::Uint(8));

    let large = func.get_param(0);
    let mut entry = func.entry_block();

    let truncated = entry.trunc(large, Type::Uint(8));

    entry.return_value(truncated);
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_address_conversions() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("AddressConvContract");

    let mut func = contract.function("addressToUint");
    func.param("addr", Type::Address).returns(Type::Uint(160));

    let addr = func.get_param(0);
    let mut entry = func.entry_block();

    let as_uint = entry.cast(addr, Type::Uint(160));

    entry.return_value(as_uint);
    func.build().unwrap();

    let mut func2 = contract.function("uintToAddress");
    func2.param("value", Type::Uint(160)).returns(Type::Address);

    let value = func2.get_param(0);
    let mut entry2 = func2.entry_block();

    let as_addr = entry2.cast(value, Type::Address);

    entry2.return_value(as_addr);
    func2.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_bytes_conversions() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("BytesConvContract");

    let mut func = contract.function("bytes32ToUint");
    func.param("data", Type::Bytes32).returns(Type::Uint(256));

    let data = func.get_param(0);
    let mut entry = func.entry_block();

    let as_uint = entry.cast(data, Type::Uint(256));

    entry.return_value(as_uint);
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_bool_conversions() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("BoolConvContract");

    let mut func = contract.function("boolToUint");
    func.param("flag", Type::Bool).returns(Type::Uint(256));

    let flag = func.get_param(0);
    let mut entry = func.entry_block();

    let as_uint = entry.cast(flag, Type::Uint(256));

    entry.return_value(as_uint);
    func.build().unwrap();

    let mut func2 = contract.function("uintToBool");
    func2.param("value", Type::Uint(256)).returns(Type::Bool);

    let value = func2.get_param(0);
    let mut entry2 = func2.entry_block();

    let zero = entry2.constant_uint(0, 256);
    let as_bool = entry2.ne(value, zero);

    entry2.return_value(as_bool);
    func2.build().unwrap();
    contract.build().unwrap();
}
