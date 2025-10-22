#![allow(unused_imports)]
#![allow(unused_variables)]

use thalir_core::{
    builder::{IRBuilder, InstBuilder},
    codegen::module::ModuleBuilder,
    types::Type,
};

#[test]
fn test_simple_add_cranelift() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("Test");

    let mut func = contract.function("add");
    func.param("a", Type::Uint(64))
        .param("b", Type::Uint(64))
        .returns(Type::Uint(64));

    let a = func.get_param(0);
    let b = func.get_param(1);

    let mut entry = func.entry_block();
    let result = entry.add(a, b, Type::Uint(64));
    entry.return_value(result);

    func.build().unwrap();
    contract.build().unwrap();

    let registry = builder.registry();
    let test_contract = registry.get_contract("Test").unwrap();

    let module_builder = ModuleBuilder::new().unwrap();
    let res = module_builder.compile_contract(test_contract);
    assert!(res.is_ok());
}
