use super::{NameObfuscator, ObfuscationConfig, ObfuscationMapping, StringSanitizer};
use crate::analysis::{AnalysisID, Pass, PassManager};
use crate::contract::Contract;
use crate::function::Function;
use crate::instructions::Instruction;
use anyhow::Result;
use indexmap::IndexMap;
use std::any::Any;

pub struct ObfuscationPass {
    config: ObfuscationConfig,
    obfuscator: NameObfuscator,
    sanitizer: StringSanitizer,
}

impl ObfuscationPass {
    pub fn new(config: ObfuscationConfig) -> Self {
        Self {
            obfuscator: NameObfuscator::new(config.clone()),
            sanitizer: StringSanitizer::new(config.clone()),
            config,
        }
    }

    pub fn export_mapping(&self) -> ObfuscationMapping {
        ObfuscationMapping::from_obfuscator(&self.obfuscator)
    }

    fn obfuscate_functions(&mut self, contract: &mut Contract) -> Result<()> {
        let mut new_functions = IndexMap::new();

        for (old_name, mut func) in contract.functions.drain(..) {
            let new_name = self.obfuscator.obfuscate_function_name(&old_name);
            func.signature.name = new_name.clone();

            self.obfuscate_function_body(&mut func)?;

            new_functions.insert(new_name, func);
        }

        contract.functions = new_functions;
        Ok(())
    }

    fn obfuscate_function_body(&mut self, func: &mut Function) -> Result<()> {
        for (i, param) in func.signature.params.iter_mut().enumerate() {
            param.name = format!("p{}", i);
        }

        for (_block_id, block) in &mut func.body.blocks {
            for inst in &mut block.instructions {
                self.sanitize_instruction_strings(inst);
            }
        }

        Ok(())
    }

    fn sanitize_instruction_strings(&mut self, inst: &mut Instruction) {
        match inst {
            Instruction::Require { message, .. } => {
                *message = self.sanitizer.sanitize_string(message);
            }
            Instruction::Assert { message, .. } => {
                *message = self.sanitizer.sanitize_string(message);
            }
            Instruction::Revert { message } => {
                *message = self.sanitizer.sanitize_string(message);
            }
            _ => {}
        }
    }

    fn obfuscate_storage(&mut self, contract: &mut Contract) -> Result<()> {
        let layout = &mut contract.storage_layout;

        for slot in &mut layout.slots {
            slot.name = self.obfuscator.obfuscate_storage_name(&slot.name);
        }

        for mapping in &mut layout.mappings {
            mapping.name = self.obfuscator.obfuscate_storage_name(&mapping.name);
        }

        for array in &mut layout.arrays {
            array.name = self.obfuscator.obfuscate_storage_name(&array.name);
        }

        for struct_layout in &mut layout.structs {
            struct_layout.name = self.obfuscator.obfuscate_storage_name(&struct_layout.name);
            for field in &mut struct_layout.fields {
                field.name = self.obfuscator.obfuscate_storage_name(&field.name);
            }
        }

        Ok(())
    }
}

impl Pass for ObfuscationPass {
    fn name(&self) -> &'static str {
        "obfuscation"
    }

    fn description(&self) -> &'static str {
        "Privacy-preserving identifier pseudonymization for safe LLM submission"
    }

    fn run_on_contract(
        &mut self,
        contract: &mut Contract,
        _manager: &mut PassManager,
    ) -> Result<()> {
        contract.name = self.obfuscator.obfuscate_contract_name(&contract.name);

        self.obfuscate_functions(contract)?;

        self.obfuscate_storage(contract)?;

        if self.config.strip_metadata {
            contract.metadata.source_file = None;
            contract.metadata.source_code = None;
        }

        Ok(())
    }

    fn modifies_ir(&self) -> bool {
        true
    }

    fn preserved_analyses(&self) -> Vec<AnalysisID> {
        vec![
            AnalysisID::ControlFlow,
            AnalysisID::Dominator,
            AnalysisID::DefUse,
            AnalysisID::AliasAnalysis,
        ]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::PassManager;
    use crate::block::{BasicBlock, Terminator};
    use crate::contract::{ContractMetadata, StorageLayout, StorageSlot};
    use crate::function::{Function, FunctionBody, FunctionSignature, Mutability, Visibility};
    use crate::obfuscation::ObfuscationLevel;
    use crate::types::Type;
    use num_bigint::BigUint;

    fn create_test_contract() -> Contract {
        let mut contract = Contract {
            name: "TestContract".to_string(),
            functions: IndexMap::new(),
            storage_layout: StorageLayout {
                slots: vec![StorageSlot {
                    slot: BigUint::from(0u32),
                    offset: 0,
                    var_type: Type::Uint(256),
                    name: "balance".to_string(),
                    packed_with: Vec::new(),
                }],
                mappings: Vec::new(),
                arrays: Vec::new(),
                structs: Vec::new(),
            },
            events: Vec::new(),
            modifiers: Vec::new(),
            constants: Vec::new(),
            metadata: ContractMetadata {
                source_file: Some("test.sol".to_string()),
                source_code: Some("contract TestContract { }".to_string()),
                ..Default::default()
            },
            source_files: crate::SourceFiles::new(),
        };

        let mut func_body = FunctionBody::new();
        func_body.blocks.insert(
            func_body.entry_block,
            BasicBlock {
                id: func_body.entry_block,
                instructions: Vec::new(),
                terminator: Terminator::Return(None),
                params: Vec::new(),
                metadata: Default::default(),
            },
        );

        let function = Function {
            signature: FunctionSignature {
                name: "transfer".to_string(),
                params: Vec::new(),
                returns: Vec::new(),
                is_payable: false,
            },
            visibility: Visibility::Public,
            mutability: Mutability::NonPayable,
            modifiers: Vec::new(),
            body: func_body,
            metadata: Default::default(),
        };

        contract.functions.insert("transfer".to_string(), function);
        contract
    }

    #[test]
    fn test_obfuscation_pass_minimal() {
        let mut contract = create_test_contract();
        let mut manager = PassManager::new();

        let config = ObfuscationConfig {
            level: ObfuscationLevel::Minimal,
            retain_mapping: true,
            ..Default::default()
        };

        let mut pass = ObfuscationPass::new(config);
        pass.run_on_contract(&mut contract, &mut manager).unwrap();

        assert_eq!(contract.name, "contract_0");

        assert!(contract.functions.contains_key("fn_0"));
        assert!(!contract.functions.contains_key("transfer"));

        assert_eq!(contract.storage_layout.slots[0].name, "var_0");
    }

    #[test]
    fn test_obfuscation_pass_standard() {
        let mut contract = create_test_contract();
        let mut manager = PassManager::new();

        let config = ObfuscationConfig {
            level: ObfuscationLevel::Standard,
            retain_mapping: true,
            hash_salt: Some("test".to_string()),
            ..Default::default()
        };

        let mut pass = ObfuscationPass::new(config);
        pass.run_on_contract(&mut contract, &mut manager).unwrap();

        assert!(contract.name.starts_with("c_"));

        let func_names: Vec<_> = contract.functions.keys().collect();
        assert!(func_names[0].starts_with("f_"));

        assert!(contract.storage_layout.slots[0].name.starts_with("v_"));
    }

    #[test]
    fn test_metadata_stripping() {
        let mut contract = create_test_contract();
        let mut manager = PassManager::new();

        let config = ObfuscationConfig {
            level: ObfuscationLevel::Minimal,
            strip_metadata: true,
            ..Default::default()
        };

        let mut pass = ObfuscationPass::new(config);
        pass.run_on_contract(&mut contract, &mut manager).unwrap();

        assert!(contract.metadata.source_file.is_none());
        assert!(contract.metadata.source_code.is_none());
    }

    #[test]
    fn test_export_mapping() {
        let config = ObfuscationConfig {
            level: ObfuscationLevel::Minimal,
            retain_mapping: true,
            ..Default::default()
        };

        let mut pass = ObfuscationPass::new(config);
        let mut contract = create_test_contract();
        let mut manager = PassManager::new();

        pass.run_on_contract(&mut contract, &mut manager).unwrap();

        let mapping = pass.export_mapping();

        assert!(mapping.deobfuscate("contract_0").is_some());
        assert!(mapping.deobfuscate("fn_0").is_some());
        assert!(mapping.deobfuscate("var_0").is_some());
    }

    #[test]
    fn test_pass_preserves_analyses() {
        let config = ObfuscationConfig::default();
        let pass = ObfuscationPass::new(config);

        let preserved = pass.preserved_analyses();

        assert!(preserved.contains(&AnalysisID::ControlFlow));
        assert!(preserved.contains(&AnalysisID::Dominator));
        assert!(preserved.contains(&AnalysisID::DefUse));
    }
}
