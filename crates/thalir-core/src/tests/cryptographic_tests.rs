use crate::builder::{IRBuilder, InstBuilderExt};
use crate::types::Type;

#[test]
fn test_keccak256() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("HashContract");

    let mut func = contract.function("hash");
    func.param("data", Type::Bytes(32)).returns(Type::Bytes32);

    let data = func.get_param(0);
    let mut entry = func.entry_block();

    let len = entry.constant_uint(32, 256);

    let hash = entry.keccak256(data, len);

    entry.return_value(hash);
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_sha256() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("Sha256Contract");

    let mut func = contract.function("sha256Hash");
    func.param("input", Type::Bytes(32)).returns(Type::Bytes32);

    let input = func.get_param(0);
    let mut entry = func.entry_block();

    let len = entry.constant_uint(32, 256);

    let hash = entry.sha256(input, len);

    entry.return_value(hash);
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_ripemd160() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("RipemdContract");

    let mut func = contract.function("ripemdHash");
    func.param("data", Type::Bytes(32)).returns(Type::Bytes20);

    let data = func.get_param(0);
    let mut entry = func.entry_block();

    let len = entry.constant_uint(32, 256);

    let hash = entry.ripemd160(data, len);

    entry.return_value(hash);
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_ecrecover() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("SignatureContract");

    let mut func = contract.function("recover");
    func.param("hash", Type::Bytes32)
        .param("v", Type::Uint(8))
        .param("r", Type::Bytes32)
        .param("s", Type::Bytes32)
        .returns(Type::Address);

    let hash = func.get_param(0);
    let v = func.get_param(1);
    let r = func.get_param(2);
    let s = func.get_param(3);
    let mut entry = func.entry_block();

    let signer = entry.ecrecover(hash, v, r, s);

    entry.return_value(signer);
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_signature_verification() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("VerifyContract");

    let mut func = contract.function("verify");
    func.param("message", Type::Bytes(32))
        .param("signature", Type::Bytes(65))
        .param("expected_signer", Type::Address)
        .returns(Type::Bool);

    let message = func.get_param(0);
    let signature = func.get_param(1);
    let expected = func.get_param(2);
    let mut entry = func.entry_block();

    let msg_len = entry.constant_uint(32, 256);
    let msg_hash = entry.keccak256(message, msg_len);

    let v = entry.constant_uint(27, 8);
    let r = entry.constant_uint(0, 256);
    let s = entry.constant_uint(0, 256);

    let recovered = entry.ecrecover(msg_hash, v, r, s);

    let is_valid = entry.eq(recovered, expected);

    entry.return_value(is_valid);
    func.build().unwrap();
    contract.build().unwrap();
}
