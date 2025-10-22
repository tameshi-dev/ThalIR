use anyhow::Result;
use thalir_core::{
    builder::{FunctionBuilderCursor, IRContext, IRRegistry},
    types::Type,
    values::Value,
};

pub fn build_if_else_with_cursor(
    func_builder: &mut FunctionBuilderCursor,
    condition_value: Value,
) -> Result<()> {
    let then_block = func_builder.create_block();
    let else_block = func_builder.create_block();
    let merge_block = func_builder.create_block();

    {
        let mut inst = func_builder.ins()?;
        inst.branch(condition_value, then_block, else_block)?;
    }

    func_builder.switch_to_block(then_block)?;
    {
        let mut inst = func_builder.ins()?;
        let then_value = inst.constant_uint(42, 256);

        inst.jump(merge_block)?;
    }

    func_builder.switch_to_block(else_block)?;
    {
        let mut inst = func_builder.ins()?;
        let else_value = inst.constant_uint(0, 256);

        inst.jump(merge_block)?;
    }

    func_builder.switch_to_block(merge_block)?;

    Ok(())
}

pub fn build_while_loop_with_cursor(func_builder: &mut FunctionBuilderCursor) -> Result<()> {
    let loop_header = func_builder.create_block();
    let loop_body = func_builder.create_block();
    let loop_exit = func_builder.create_block();

    {
        let mut inst = func_builder.ins()?;
        inst.jump(loop_header)?;
    }

    func_builder.switch_to_block(loop_header)?;
    {
        let mut inst = func_builder.ins()?;

        let counter = inst.constant_uint(5, 256);
        let limit = inst.constant_uint(10, 256);
        let should_continue = inst.lt(counter, limit);
        inst.branch(should_continue, loop_body, loop_exit)?;
    }

    func_builder.switch_to_block(loop_body)?;
    {
        let mut inst = func_builder.ins()?;

        inst.jump(loop_header)?;
    }

    func_builder.switch_to_block(loop_exit)?;

    Ok(())
}

pub fn build_complete_function() -> Result<()> {
    let mut context = IRContext::new();
    let mut registry = IRRegistry::new();

    let mut func_builder = FunctionBuilderCursor::new(
        "TestContract".to_string(),
        "testFunction".to_string(),
        &mut context,
        &mut registry,
    );

    func_builder
        .param("amount", Type::Uint(256))
        .returns(Type::Uint(256));

    let entry = func_builder.entry_block();
    func_builder.switch_to_block(entry)?;

    let param_amount = func_builder.get_param(0);
    let threshold = {
        let mut inst = func_builder.ins()?;
        inst.constant_uint(100, 256)
    };

    let condition = {
        let mut inst = func_builder.ins()?;
        inst.lt(param_amount.clone(), threshold)
    };

    let small_block = func_builder.create_block();
    let large_block = func_builder.create_block();
    let merge_block = func_builder.create_block();

    {
        let mut inst = func_builder.ins()?;
        inst.branch(condition, small_block, large_block)?;
    }

    func_builder.switch_to_block(small_block)?;
    let small_result = {
        let mut inst = func_builder.ins()?;
        let fee = inst.constant_uint(1, 256);
        let result = inst.add(param_amount.clone(), fee, Type::Uint(256));
        inst.jump(merge_block)?;
        result
    };

    func_builder.switch_to_block(large_block)?;
    let large_result = {
        let mut inst = func_builder.ins()?;
        let fee = inst.constant_uint(10, 256);
        let result = inst.add(param_amount.clone(), fee, Type::Uint(256));
        inst.jump(merge_block)?;
        result
    };

    func_builder.switch_to_block(merge_block)?;
    {
        let mut inst = func_builder.ins()?;
        inst.return_value(small_result)?;
    }

    let _function = func_builder.build()?;

    Ok(())
}

pub fn build_complex_cfg(func_builder: &mut FunctionBuilderCursor) -> Result<()> {
    let blocks = vec![
        func_builder.entry_block(),
        func_builder.create_block(),
        func_builder.create_block(),
        func_builder.create_block(),
        func_builder.create_block(),
        func_builder.create_block(),
        func_builder.create_block(),
    ];

    func_builder.switch_to_block(blocks[0])?;
    {
        let mut inst = func_builder.ins()?;
        inst.jump(blocks[1])?;
    }

    func_builder.switch_to_block(blocks[1])?;
    {
        let mut inst = func_builder.ins()?;
        let check1 = inst.constant_bool(true);
        inst.branch(check1, blocks[2], blocks[4])?;
    }

    func_builder.switch_to_block(blocks[2])?;
    {
        let mut inst = func_builder.ins()?;
        let check2 = inst.constant_bool(false);
        inst.branch(check2, blocks[3], blocks[4])?;
    }

    func_builder.switch_to_block(blocks[3])?;
    {
        let mut inst = func_builder.ins()?;
        inst.jump(blocks[5])?;
    }

    func_builder.switch_to_block(blocks[4])?;
    {
        let mut inst = func_builder.ins()?;
        inst.jump(blocks[5])?;
    }

    func_builder.switch_to_block(blocks[5])?;
    {
        let mut inst = func_builder.ins()?;
        inst.jump(blocks[6])?;
    }

    func_builder.switch_to_block(blocks[6])?;
    {
        let mut inst = func_builder.ins()?;
        inst.return_void()?;
    }

    Ok(())
}
