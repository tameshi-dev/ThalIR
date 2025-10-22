#![allow(unused_imports)]
#![allow(unused_variables)]

use num_bigint::BigUint;
use thalir_core::{
    block::BlockId,
    builder::{IRBuilder, InstBuilder, InstBuilderExt},
    types::Type,
};

fn main() {
    println!("Building Simple Storage contract IR...\n");

    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("SimpleStorage");

    contract.state_variable("storedValue", Type::Uint(256), 0);

    build_setter(&mut contract);

    build_getter(&mut contract);

    build_complex_function(&mut contract);

    contract.build().unwrap();

    let stats = builder.stats();
    println!("\n=== IR Construction Complete ===");
    println!("Contracts: {}", stats.contracts);
    println!("Functions: {}", stats.functions);
    println!("Blocks: {}", stats.blocks);
    println!("Instructions: {}", stats.instructions);

    let registry = builder.registry();
    let simple_storage = registry.get_contract("SimpleStorage").unwrap();

    println!("\n=== Contract Details ===");
    println!("Contract: {}", simple_storage.name);
    println!(
        "Storage slots: {}",
        simple_storage.storage_layout.slots.len()
    );
    println!("Functions:");
    for (name, func) in &simple_storage.functions {
        println!(
            "  - {} ({} params, {} returns)",
            name,
            func.signature.params.len(),
            func.signature.returns.len()
        );
    }
}

fn build_setter(contract: &mut thalir_core::builder::ContractBuilder) {
    println!("Building setter function...");

    let mut func = contract.function("setValue");
    func.param("newValue", Type::Uint(256));

    let new_value = func.get_param(0);
    let mut entry = func.entry_block();

    entry.storage_store(BigUint::from(0u32), new_value);

    entry.return_void().unwrap();

    func.build().unwrap();
    println!("   setValue function built");
}

fn build_getter(contract: &mut thalir_core::builder::ContractBuilder) {
    println!("Building getter function...");

    let mut func = contract.function("getValue");
    func.returns(Type::Uint(256));

    let mut entry = func.entry_block();

    let value = entry.storage_load(BigUint::from(0u32));

    entry.return_value(value);

    func.build().unwrap();
    println!("   getValue function built");
}

fn build_complex_function(contract: &mut thalir_core::builder::ContractBuilder) {
    println!("Building complex function with control flow...");

    let mut func = contract.function("processValue");
    func.param("input", Type::Uint(256))
        .returns(Type::Uint(256));

    let input = func.get_param(0);

    let check_block_id = func.create_block_id();
    let increment_block_id = func.create_block_id();
    let decrement_block_id = func.create_block_id();
    let final_block_id = func.create_block_id();

    let stored_value = {
        let mut entry = func.entry_block();
        let value = entry.storage_load(BigUint::from(0u32));
        entry.jump(check_block_id);
        value
    };

    {
        let mut check_block = func.block("check");
        let condition = check_block.gt(input.clone(), stored_value.clone());
        check_block.branch(condition, increment_block_id, decrement_block_id);
    }

    let inc_result = {
        let mut inc_block = func.block("increment");
        let one = inc_block.constant_uint(1, 256);
        let result = inc_block.add(input.clone(), one, Type::Uint(256));
        inc_block.jump(final_block_id);
        result
    };

    let dec_result = {
        let mut dec_block = func.block("decrement");
        let one = dec_block.constant_uint(1, 256);
        let result = dec_block.sub(input.clone(), one, Type::Uint(256));
        dec_block.jump(final_block_id);
        result
    };

    {
        let mut final_block = func.block("final");
        let result = final_block.phi(vec![
            (increment_block_id, inc_result),
            (decrement_block_id, dec_result),
        ]);

        final_block.storage_store(BigUint::from(0u32), result.clone());

        final_block.return_value(result);
    }

    func.build().unwrap();
    println!("   processValue function built with {} blocks", 5);
}
