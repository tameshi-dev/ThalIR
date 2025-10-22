#![allow(unused_imports)]
#![allow(unused_variables)]

use indexmap::IndexMap;
use num_bigint::BigUint;
use thalir_core::{
    analysis::PassManager,
    block::{BasicBlock, Terminator},
    contract::{Contract, ContractMetadata, StorageLayout, StorageSlot},
    function::{Function, FunctionBody, FunctionSignature, Mutability, Visibility},
    instructions::Instruction,
    types::Type,
    values::{Constant, Value},
    ObfuscationConfig, ObfuscationLevel, ObfuscationMapping, ObfuscationPass, VulnerabilityMapper,
};

fn main() -> anyhow::Result<()> {
    println!("=== ThalIR Privacy-Preserving Pseudonymization Example ===\n");

    println!(" STEP 1: Creating contract with proprietary names...\n");

    let contract = create_novel_amm_contract();

    println!("Original Contract:");
    println!("  Name: {}", contract.name);
    println!("  Functions:");
    for func_name in contract.functions.keys() {
        println!("    - {}", func_name);
    }
    println!("  Storage Variables:");
    for slot in &contract.storage_layout.slots {
        println!("    - {} (slot {})", slot.name, slot.slot);
    }
    println!();

    println!(" STEP 2: Applying obfuscation...\n");

    demonstrate_minimal_obfuscation(contract.clone())?;

    println!("\n{}\n", "=".repeat(70));

    demonstrate_standard_obfuscation(contract.clone())?;

    println!("\n{}\n", "=".repeat(70));

    println!(" STEP 3: Complete workflow with vulnerability detection...\n");

    demonstrate_full_workflow(contract)?;

    Ok(())
}

fn create_novel_amm_contract() -> Contract {
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
                    name: "protocolOwner".to_string(),
                    packed_with: Vec::new(),
                },
                StorageSlot {
                    slot: BigUint::from(2u32),
                    offset: 0,
                    var_type: Type::Uint(256),
                    name: "customBondingParameter".to_string(),
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
            source_file: Some("src/amm/NovelBondingCurveAMM.sol".to_string()),
            source_code: Some(
                "// Proprietary bonding curve implementation\ncontract NovelBondingCurveAMM { ... }"
                    .to_string(),
            ),
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
            instructions: vec![
                Instruction::Require {
                    condition: Value::Constant(Constant::Bool(true)),
                    message: "Invalid bonding curve parameters - proprietary check".to_string(),
                },
                Instruction::Assert {
                    condition: Value::Constant(Constant::Bool(true)),
                    message: "Custom slippage protection activated".to_string(),
                },
            ],
            terminator: Terminator::Return(None),
            metadata: Default::default(),
        },
    );

    let function = Function {
        signature: FunctionSignature {
            name: "calculateProprietaryBondingCurve".to_string(),
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
        .insert("calculateProprietaryBondingCurve".to_string(), function);

    contract
}

fn demonstrate_minimal_obfuscation(mut contract: Contract) -> anyhow::Result<()> {
    println!(" Example A: Minimal Obfuscation (Sequential Counters)");
    println!();

    let config = ObfuscationConfig {
        level: ObfuscationLevel::Minimal,
        retain_mapping: true,
        strip_string_constants: true,
        strip_error_messages: true,
        strip_metadata: false,
        ..Default::default()
    };

    let mut manager = PassManager::new();
    manager.register_pass(ObfuscationPass::new(config));
    manager.run_all(&mut contract)?;

    println!("Obfuscated Contract:");
    println!("  Name: {} (was: NovelBondingCurveAMM)", contract.name);
    println!("  Functions:");
    for func_name in contract.functions.keys() {
        println!(
            "    - {} (was: calculateProprietaryBondingCurve)",
            func_name
        );
    }
    println!("  Storage Variables:");
    for (i, slot) in contract.storage_layout.slots.iter().enumerate() {
        let original = match i {
            0 => "liquidityPoolReserves",
            1 => "protocolOwner",
            2 => "customBondingParameter",
            _ => "unknown",
        };
        println!("    - {} (was: {})", slot.name, original);
    }

    if let Some(func) = contract.functions.values().next() {
        if let Some(block) = func.body.blocks.get(&func.body.entry_block) {
            if let Some(Instruction::Require { message, .. }) = block.instructions.first() {
                println!("\n  Error messages sanitized:");
                println!(
                    "    - '{}' (was: 'Invalid bonding curve parameters - proprietary check')",
                    message
                );
            }
        }
    }

    Ok(())
}

fn demonstrate_standard_obfuscation(mut contract: Contract) -> anyhow::Result<()> {
    println!(" Example B: Standard Obfuscation (Hash-Based)");
    println!();

    let config = ObfuscationConfig {
        level: ObfuscationLevel::Standard,
        retain_mapping: true,
        hash_salt: Some("my-secret-salt-12345".to_string()),
        strip_string_constants: true,
        strip_error_messages: true,
        strip_metadata: true,
        ..Default::default()
    };

    let mut manager = PassManager::new();
    manager.register_pass(ObfuscationPass::new(config));
    manager.run_all(&mut contract)?;

    println!("Obfuscated Contract:");
    println!("  Name: {} (deterministic hash with salt)", contract.name);
    println!("  Functions:");
    for func_name in contract.functions.keys() {
        println!("    - {} (hash-based identifier)", func_name);
    }
    println!("  Storage Variables:");
    for slot in &contract.storage_layout.slots {
        println!("    - {} (hash-based identifier)", slot.name);
    }

    println!("\n  Metadata stripped:");
    println!("    - source_file: {:?}", contract.metadata.source_file);
    println!("    - source_code: {:?}", contract.metadata.source_code);

    Ok(())
}

fn demonstrate_full_workflow(mut contract: Contract) -> anyhow::Result<()> {
    println!(" Complete Workflow: Obfuscate → LLM Analysis → De-obfuscate");
    println!();

    println!("Step 1: Applying obfuscation...");
    let config = ObfuscationConfig {
        level: ObfuscationLevel::Minimal,
        retain_mapping: true,
        strip_string_constants: true,
        strip_error_messages: true,
        strip_metadata: true,
        ..Default::default()
    };

    let mut manager = PassManager::new();
    manager.register_pass(ObfuscationPass::new(config));
    manager.run_all(&mut contract)?;

    println!("Step 2: Exporting obfuscation mapping...");
    let pass = manager
        .get_pass::<ObfuscationPass>()
        .expect("ObfuscationPass should be registered");
    let mapping = pass.export_mapping();

    println!("  Mapping contains {} entries", mapping.mapping.len());
    println!("  Sample mappings:");
    for (obf, orig) in mapping.mapping.iter().take(3) {
        println!("    {} → {}", obf, orig);
    }

    println!("\nStep 3: Generating obfuscated IR for LLM submission...");
    println!("  Contract: {}", contract.name);
    println!("  Function: {}", contract.functions.keys().next().unwrap());
    println!("  Storage: {}", contract.storage_layout.slots[0].name);
    println!("   Safe to submit to cloud LLM service!");

    println!("\nStep 4: Simulating LLM vulnerability analysis...");
    let obfuscated_report = simulate_llm_vulnerability_detection(&contract);
    println!("  LLM Response (obfuscated):");
    for line in obfuscated_report.lines() {
        println!("    {}", line);
    }

    println!("\nStep 5: De-obfuscating vulnerability report...");
    let mapper = VulnerabilityMapper::from_mapping(mapping.clone());
    let original_report = mapper.deobfuscate_report(&obfuscated_report);

    println!("  Original Report (with your actual names):");
    for line in original_report.lines() {
        println!("    {}", line);
    }

    println!("\nStep 6: Saving mapping to file...");
    let temp_dir = std::env::temp_dir();
    let mapping_path = temp_dir.join("obfuscation_mapping.json");
    mapping.save_to_file(&mapping_path)?;
    println!("   Mapping saved to: {:?}", mapping_path);
    println!("   Keep this file secure - it's your de-obfuscation key!");

    let loaded_mapping = ObfuscationMapping::load_from_file(&mapping_path)?;
    println!(
        "\n   Mapping loaded successfully ({} entries)",
        loaded_mapping.mapping.len()
    );

    Ok(())
}

fn simulate_llm_vulnerability_detection(contract: &Contract) -> String {
    let contract_name = &contract.name;
    let func_name = contract.functions.keys().next().unwrap();
    let storage_var = &contract.storage_layout.slots[0].name;

    format!(
        "SECURITY ANALYSIS REPORT\n\
         ======================\n\
         \n\
         VULNERABILITY FOUND: Reentrancy Risk\n\
         - Location: {}::{} at position [2]\n\
         - Severity: HIGH\n\
         - Description: External call followed by state modification\n\
         - Affected storage: {}\n\
         - Recommendation: Use checks-effects-interactions pattern\n\
         \n\
         VULNERABILITY FOUND: Unchecked Arithmetic\n\
         - Location: {}::{} at position [5]\n\
         - Severity: MEDIUM  \n\
         - Description: Arithmetic operation without overflow check\n\
         - Recommendation: Use SafeMath or Solidity 0.8+\n\
         \n\
         PASS: Access control properly implemented in {}\n\
         PASS: No timestamp dependencies detected",
        contract_name, func_name, storage_var, contract_name, func_name, contract_name
    )
}
