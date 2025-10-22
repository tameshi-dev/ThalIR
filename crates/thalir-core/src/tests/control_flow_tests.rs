use crate::block::BlockId;
use crate::builder::{IRBuilder, InstBuilder, InstBuilderExt};
use crate::types::Type;
use crate::values::Value;

#[test]
fn test_if_else() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("ControlFlowContract");
    let mut func = contract.function("if_else_test");

    func.param("x", Type::Uint(256)).returns(Type::Uint(256));

    let x = func.get_param(0);

    let then_block_id = func.create_block_id();
    let else_block_id = func.create_block_id();
    let merge_block_id = func.create_block_id();

    {
        let mut entry = func.entry_block();
        let threshold = entry.constant_uint(100, 256);
        let condition = entry.gt(x.clone(), threshold);
        entry.branch(condition, then_block_id, else_block_id);
    }

    let then_result = {
        let mut then_block = func.block("then");
        let double = then_block.constant_uint(2, 256);
        let result = then_block.mul(x.clone(), double, Type::Uint(256));
        then_block.jump(merge_block_id);
        result
    };

    let else_result = {
        let mut else_block = func.block("else");
        let half = else_block.constant_uint(2, 256);
        let result = else_block.div(x, half, Type::Uint(256));
        else_block.jump(merge_block_id);
        result
    };

    {
        let mut merge_block = func.block("merge");
        let phi_result = merge_block.phi(vec![
            (then_block_id, then_result),
            (else_block_id, else_result),
        ]);
        merge_block.return_value(phi_result);
    }

    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_loop() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("LoopContract");
    let mut func = contract.function("sum_to_n");

    func.param("n", Type::Uint(256)).returns(Type::Uint(256));

    let n = func.get_param(0);

    let entry_id = BlockId(0);
    let loop_header_id = func.create_block_id();
    let loop_body_id = func.create_block_id();
    let loop_exit_id = func.create_block_id();

    let (zero, one) = {
        let mut entry = func.entry_block();
        let z = entry.constant_uint(0, 256);
        let o = entry.constant_uint(1, 256);
        entry.jump(loop_header_id);
        (z, o)
    };

    let (new_i, new_sum) = {
        let mut loop_body = func.block("loop_body");

        let i_phi = Value::Temp(crate::values::TempId(100));
        let sum_phi = Value::Temp(crate::values::TempId(101));
        let ns = loop_body.add(sum_phi.clone(), i_phi.clone(), Type::Uint(256));
        let ni = loop_body.add(i_phi.clone(), one.clone(), Type::Uint(256));
        loop_body.jump(loop_header_id);
        (ni, ns)
    };

    let sum_phi = {
        let mut loop_header = func.block("loop_header");
        let i_phi = loop_header.phi(vec![(entry_id, zero.clone()), (loop_body_id, new_i)]);
        let s_phi = loop_header.phi(vec![(entry_id, zero.clone()), (loop_body_id, new_sum)]);
        let continue_loop = loop_header.lt(i_phi.clone(), n);
        loop_header.branch(continue_loop, loop_body_id, loop_exit_id);
        s_phi
    };

    {
        let mut loop_exit = func.block("loop_exit");
        let final_sum = loop_exit.phi(vec![(loop_header_id, sum_phi)]);
        loop_exit.return_value(final_sum);
    }

    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_require_assert() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("RequireContract");
    let mut func = contract.function("safe_divide");

    func.param("numerator", Type::Uint(256))
        .param("denominator", Type::Uint(256))
        .returns(Type::Uint(256));

    let numerator = func.get_param(0);
    let denominator = func.get_param(1);

    let mut entry = func.entry_block();

    let zero = entry.constant_uint(0, 256);
    let not_zero = entry.ne(denominator.clone(), zero);

    entry.require(not_zero.clone(), "Division by zero");

    let result = entry.div(numerator.clone(), denominator.clone(), Type::Uint(256));

    let max_value = entry.constant_uint(1000000, 256);
    let valid_result = entry.le(result.clone(), max_value);
    entry.assert(valid_result, "Result too large");

    entry.return_value(result);

    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_revert() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("RevertContract");
    let mut func = contract.function("may_revert");

    func.param("value", Type::Uint(256));

    let value = func.get_param(0);

    let revert_block_id = func.create_block_id();
    let continue_block_id = func.create_block_id();

    {
        let mut entry = func.entry_block();
        let min_value = entry.constant_uint(10, 256);
        let max_value = entry.constant_uint(100, 256);

        let too_small = entry.lt(value.clone(), min_value);
        let too_large = entry.gt(value.clone(), max_value);
        let invalid = entry.or(too_small, too_large);

        entry.branch(invalid, revert_block_id, continue_block_id);
    }

    {
        let mut revert_block = func.block("revert");
        revert_block.revert("Value out of range");
    }

    {
        let mut continue_block = func.block("continue");
        continue_block.return_void().unwrap();
    }

    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_nested_control_flow() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("NestedFlowContract");
    let mut func = contract.function("nested_flow");

    func.param("a", Type::Uint(256))
        .param("b", Type::Uint(256))
        .returns(Type::Uint(256));

    let a = func.get_param(0);
    let b = func.get_param(1);

    let outer_then_id = func.create_block_id();
    let outer_else_id = func.create_block_id();
    let inner_then_id = func.create_block_id();
    let inner_else_id = func.create_block_id();
    let final_block_id = func.create_block_id();

    {
        let mut entry = func.entry_block();
        let ten = entry.constant_uint(10, 256);
        let outer_cond = entry.gt(a.clone(), ten);
        entry.branch(outer_cond, outer_then_id, outer_else_id);
    }

    {
        let mut outer_then = func.block("outer_then");
        let twenty = outer_then.constant_uint(20, 256);
        let inner_cond = outer_then.lt(b.clone(), twenty);
        outer_then.branch(inner_cond, inner_then_id, inner_else_id);
    }

    let result1 = {
        let mut inner_then = func.block("inner_then");
        let r = inner_then.add(a.clone(), b.clone(), Type::Uint(256));
        inner_then.jump(final_block_id);
        r
    };

    let result2 = {
        let mut inner_else = func.block("inner_else");
        let r = inner_else.sub(a.clone(), b.clone(), Type::Uint(256));
        inner_else.jump(final_block_id);
        r
    };

    let result3 = {
        let mut outer_else = func.block("outer_else");
        let r = outer_else.mul(a, b, Type::Uint(256));
        outer_else.jump(final_block_id);
        r
    };

    {
        let mut final_block = func.block("final");
        let final_result = final_block.phi(vec![
            (inner_then_id, result1),
            (inner_else_id, result2),
            (outer_else_id, result3),
        ]);
        final_block.return_value(final_result);
    }

    func.build().unwrap();
    contract.build().unwrap();
}
