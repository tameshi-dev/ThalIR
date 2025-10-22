#![allow(unused_imports)]
#![allow(unused_variables)]

use thalir_core::{
    analysis::{Pass, PassManager},
    block::{BasicBlock, BlockId, Terminator},
    contract::{Contract, ContractMetadata, StorageLayout, StorageSlot},
    function::{Function, FunctionBody, FunctionSignature, Mutability, Visibility},
    instructions::Instruction,
    types::Type,
    values::{Constant, TempId, Value},
    ObfuscationConfig, ObfuscationLevel, ObfuscationPass, VulnerabilityMapper,
};

use indexmap::IndexMap;
use num_bigint::BigUint;

fn create_test_contract_with_identifiable_names() -> Contract {
    let mut contract = Contract {
        name: "NovelBondingCurveAMM".to_string(),
        functions: IndexMap::new(),
        storage_layout: StorageLayout {
            slots: vec![
                StorageSlot {
                    slot: BigUint::from(0u32),
                    offset: 0,
                    var_type: Type::Uint(256),
                    name: "liquidityPoolReserves".to_string(),
                    packed_with: Vec::new(),
                },
                StorageSlot {
                    slot: BigUint::from(1u32),
                    offset: 0,
                    var_type: Type::Address,
                    name: "contractOwner".to_string(),
                    packed_with: Vec::new(),
                },
            ],
            mappings: Vec::new(),
            arrays: Vec::new(),
            structs: Vec::new(),
        },
        events: Vec::new(),
        modifiers: Vec::new(),
        constants: Vec::new(),
        metadata: ContractMetadata {
            source_file: Some("NovelBondingCurveAMM.sol".to_string()),
            source_code: Some("contract NovelBondingCurveAMM { ... }".to_string()),
            ..Default::default()
        },
        source_files: thalir_core::SourceFiles::new(),
    };

    let mut func_body = FunctionBody::new();
    func_body.blocks.insert(
        func_body.entry_block,
        BasicBlock {
            id: func_body.entry_block,
            params: Vec::new(),
            instructions: vec![Instruction::Require {
                condition: Value::Constant(Constant::Bool(true)),
                message: "Invalid bonding curve parameters".to_string(),
            }],
            terminator: Terminator::Return(None),
            metadata: Default::default(),
        },
    );

    let function = Function {
        signature: FunctionSignature {
            name: "calculateBondingCurve".to_string(),
            params: vec![],
            returns: vec![],
            is_payable: false,
        },
        visibility: Visibility::Public,
        mutability: Mutability::NonPayable,
        modifiers: Vec::new(),
        body: func_body,
        metadata: Default::default(),
    };

    contract
        .functions
        .insert("calculateBondingCurve".to_string(), function);
    contract
}

#[test]
fn test_end_to_end_obfuscation_workflow() {
    let mut contract = create_test_contract_with_identifiable_names();

    assert_eq!(contract.name, "NovelBondingCurveAMM");
    assert!(contract.functions.contains_key("calculateBondingCurve"));
    assert_eq!(
        contract.storage_layout.slots[0].name,
        "liquidityPoolReserves"
    );
    assert_eq!(contract.storage_layout.slots[1].name, "contractOwner");

    let mut manager = PassManager::new();
    let obf_config = ObfuscationConfig {
        level: ObfuscationLevel::Minimal,
        retain_mapping: true,
        strip_string_constants: true,
        strip_error_messages: true,
        strip_metadata: true,
        ..Default::default()
    };

    manager.register_pass(ObfuscationPass::new(obf_config));
    manager.run_all(&mut contract).unwrap();

    assert_eq!(contract.name, "contract_0");
    assert!(!contract.functions.contains_key("calculateBondingCurve"));
    assert!(contract.functions.contains_key("fn_0"));
    assert_eq!(contract.storage_layout.slots[0].name, "var_0");
    assert_eq!(contract.storage_layout.slots[1].name, "var_1");

    assert!(contract.metadata.source_file.is_none());
    assert!(contract.metadata.source_code.is_none());

    let func = contract.functions.get("fn_0").unwrap();
    if let Some(block) = func.body.blocks.get(&func.body.entry_block) {
        if let Some(Instruction::Require { message, .. }) = block.instructions.first() {
            assert_eq!(message, "error_0");
        }
    }

    let pass = manager.get_pass::<ObfuscationPass>().unwrap();
    let mapping = pass.export_mapping();

    assert_eq!(
        mapping.deobfuscate("contract_0"),
        Some("NovelBondingCurveAMM")
    );
    assert_eq!(mapping.deobfuscate("fn_0"), Some("calculateBondingCurve"));
    assert_eq!(mapping.deobfuscate("var_0"), Some("liquidityPoolReserves"));
    assert_eq!(mapping.deobfuscate("var_1"), Some("contractOwner"));

    let obfuscated_report = "VULNERABILITY FOUND:\n\
        Reentrancy risk in contract_0::fn_0 at position [1]\n\
        Access to storage var_0 after external call\n\
        Consider using checks-effects-interactions pattern";

    let mapper = VulnerabilityMapper::from_mapping(mapping);
    let original_report = mapper.deobfuscate_report(obfuscated_report);

    assert!(original_report.contains("NovelBondingCurveAMM"));
    assert!(original_report.contains("calculateBondingCurve"));
    assert!(original_report.contains("liquidityPoolReserves"));
    assert!(!original_report.contains("contract_0"));
    assert!(!original_report.contains("fn_0"));
    assert!(!original_report.contains("var_0"));
}

#[test]
fn test_obfuscation_with_standard_hashing() {
    let mut contract = create_test_contract_with_identifiable_names();

    let mut manager = PassManager::new();
    let obf_config = ObfuscationConfig {
        level: ObfuscationLevel::Standard,
        retain_mapping: true,
        hash_salt: Some("test-salt-12345".to_string()),
        ..Default::default()
    };

    manager.register_pass(ObfuscationPass::new(obf_config));
    manager.run_all(&mut contract).unwrap();

    assert!(contract.name.starts_with("c_"));
    assert_eq!(contract.name.len(), 8);

    let func_names: Vec<_> = contract.functions.keys().collect();
    assert!(func_names[0].starts_with("f_"));
    assert_eq!(func_names[0].len(), 8);

    assert!(contract.storage_layout.slots[0].name.starts_with("v_"));
    assert_eq!(contract.storage_layout.slots[0].name.len(), 8);
}

#[test]
fn test_obfuscation_determinism() {
    let mut contract1 = create_test_contract_with_identifiable_names();
    let mut contract2 = create_test_contract_with_identifiable_names();

    let salt = "deterministic-salt";

    let config = ObfuscationConfig {
        level: ObfuscationLevel::Standard,
        hash_salt: Some(salt.to_string()),
        retain_mapping: true,
        ..Default::default()
    };

    let mut manager1 = PassManager::new();
    manager1.register_pass(ObfuscationPass::new(config.clone()));
    manager1.run_all(&mut contract1).unwrap();

    let mut manager2 = PassManager::new();
    manager2.register_pass(ObfuscationPass::new(config));
    manager2.run_all(&mut contract2).unwrap();

    assert_eq!(contract1.name, contract2.name);

    let func1_names: Vec<_> = contract1.functions.keys().collect();
    let func2_names: Vec<_> = contract2.functions.keys().collect();
    assert_eq!(func1_names, func2_names);

    assert_eq!(
        contract1.storage_layout.slots[0].name,
        contract2.storage_layout.slots[0].name
    );
}

#[test]
fn test_obfuscation_none_level_is_noop() {
    let mut contract = create_test_contract_with_identifiable_names();
    let original_name = contract.name.clone();
    let original_func_name = contract.functions.keys().next().unwrap().clone();

    let mut manager = PassManager::new();
    let obf_config = ObfuscationConfig {
        level: ObfuscationLevel::None,
        ..Default::default()
    };

    manager.register_pass(ObfuscationPass::new(obf_config));
    manager.run_all(&mut contract).unwrap();

    assert_eq!(contract.name, original_name);
    assert!(contract.functions.contains_key(&original_func_name));
}

#[test]
fn test_pass_preserves_structural_analyses() {
    let obf_config = ObfuscationConfig {
        level: ObfuscationLevel::Standard,
        ..Default::default()
    };

    let pass = ObfuscationPass::new(obf_config);
    let preserved = pass.preserved_analyses();

    use thalir_core::analysis::AnalysisID;

    assert!(preserved.contains(&AnalysisID::ControlFlow));
    assert!(preserved.contains(&AnalysisID::Dominator));
    assert!(preserved.contains(&AnalysisID::DefUse));
    assert!(preserved.contains(&AnalysisID::AliasAnalysis));
}
