use crate::builder::{IRBuilder, InstBuilder, InstBuilderExt};
use crate::types::Type;
use num_bigint::BigUint;

#[test]
fn test_basic_arithmetic() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("TestContract");
    let mut func = contract.function("test_arithmetic");

    func.param("a", Type::Uint(256))
        .param("b", Type::Uint(256))
        .returns(Type::Uint(256));

    let a = func.get_param(0);
    let b = func.get_param(1);

    let mut entry = func.entry_block();

    let sum = entry.add(a.clone(), b.clone(), Type::Uint(256));
    let diff = entry.sub(sum.clone(), b.clone(), Type::Uint(256));
    let product = entry.mul(a.clone(), b.clone(), Type::Uint(256));
    let quotient = entry.div(product.clone(), a.clone(), Type::Uint(256));
    let remainder = entry.mod_(a.clone(), b.clone(), Type::Uint(256));

    let two = entry.constant_uint(2, 256);
    let squared = entry.pow(a.clone(), two);

    entry.return_value(squared).unwrap();
    func.build().unwrap();
    contract.build().unwrap();

    let stats = builder.stats();
    assert_eq!(stats.contracts, 1);
    assert_eq!(stats.functions, 1);
}

#[test]
fn test_checked_arithmetic() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("SafeMathContract");
    let mut func = contract.function("safe_operations");

    func.param("x", Type::Uint(256))
        .param("y", Type::Uint(256))
        .returns(Type::Uint(256));

    let x = func.get_param(0);
    let y = func.get_param(1);
    let mut entry = func.entry_block();

    let safe_sum = entry.checked_add(x.clone(), y.clone(), Type::Uint(256));
    let safe_diff = entry.checked_sub(safe_sum.clone(), y.clone(), Type::Uint(256));
    let safe_product = entry.checked_mul(x.clone(), y.clone(), Type::Uint(256));
    let safe_quotient = entry.checked_div(safe_product, y, Type::Uint(256));

    entry.return_value(safe_quotient).unwrap();
    func.build().unwrap();
    contract.build().unwrap();

    assert!(builder.validate().is_ok());
}

#[test]
fn test_bitwise_operations() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("BitwiseContract");
    let mut func = contract.function("bitwise_ops");

    func.param("a", Type::Uint(256))
        .param("b", Type::Uint(256))
        .returns(Type::Uint(256));

    let a = func.get_param(0);
    let b = func.get_param(1);
    let mut entry = func.entry_block();

    let and_result = entry.and(a.clone(), b.clone());
    let or_result = entry.or(a.clone(), b.clone());
    let xor_result = entry.xor(a.clone(), b.clone());
    let not_result = entry.not(a.clone());

    let shift_amount = entry.constant_uint(4, 256);
    let left_shifted = entry.shl(a.clone(), shift_amount.clone());
    let right_shifted = entry.shr(left_shifted.clone(), shift_amount.clone());
    let arith_shifted = entry.sar(a, shift_amount);

    entry.return_value(xor_result).unwrap();
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_comparison_operations() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("ComparisonContract");
    let mut func = contract.function("compare");

    func.param("x", Type::Uint(256))
        .param("y", Type::Uint(256))
        .returns(Type::Bool);

    let x = func.get_param(0);
    let y = func.get_param(1);
    let mut entry = func.entry_block();

    let eq_result = entry.eq(x.clone(), y.clone());
    let ne_result = entry.ne(x.clone(), y.clone());
    let lt_result = entry.lt(x.clone(), y.clone());
    let gt_result = entry.gt(x.clone(), y.clone());
    let le_result = entry.le(x.clone(), y.clone());
    let ge_result = entry.ge(x.clone(), y.clone());

    let combined = entry.and(eq_result, le_result);

    entry.return_value(combined).unwrap();
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_constants() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("ConstantContract");
    let mut func = contract.function("use_constants");

    func.returns(Type::Uint(256));

    let mut entry = func.entry_block();

    let uint_const = entry.constant_uint(42, 256);
    let int_const = entry.constant_int(-42, 256);
    let bool_const = entry.constant_bool(true);
    let addr_const = entry.constant_address([0xAA; 20]);

    let zero = entry.constant_uint(0, 256);
    let one = entry.constant_uint(1, 256);
    let max = entry.constant_uint(u64::MAX, 256);

    let result = entry.add(uint_const, one, Type::Uint(256));

    entry.return_value(result).unwrap();
    func.build().unwrap();
    contract.build().unwrap();
}
