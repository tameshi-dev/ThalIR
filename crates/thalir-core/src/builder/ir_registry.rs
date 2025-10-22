use crate::{
    block::{BasicBlock, BlockId},
    contract::Contract,
    function::Function,
    instructions::Instruction,
    values::Value,
    IrError, Result,
};
use indexmap::IndexMap;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct IRRegistry {
    contracts: IndexMap<String, Contract>,
    functions: HashMap<String, Function>,
    blocks: HashMap<BlockId, BasicBlock>,
    instructions: HashMap<String, Instruction>,
    values: HashMap<String, Value>,
    function_to_contract: HashMap<String, String>,
    block_to_function: HashMap<BlockId, String>,
}

impl IRRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_contract(&mut self, contract: Contract) -> Result<()> {
        let name = contract.name.clone();
        if self.contracts.contains_key(&name) {
            return Err(IrError::BuilderError(format!(
                "Contract {} already exists",
                name
            )));
        }
        self.contracts.insert(name, contract);
        Ok(())
    }

    pub fn get_contract(&self, name: &str) -> Option<&Contract> {
        self.contracts.get(name)
    }

    pub fn get_contract_mut(&mut self, name: &str) -> Option<&mut Contract> {
        self.contracts.get_mut(name)
    }

    pub fn contracts(&self) -> impl Iterator<Item = (&String, &Contract)> {
        self.contracts.iter()
    }

    pub fn add_function(&mut self, contract_name: String, mut function: Function) -> Result<()> {
        let qualified_name = format!("{}::{}", contract_name, function.signature.name);

        if self.functions.contains_key(&qualified_name) {
            return Err(IrError::BuilderError(format!(
                "Function {} already exists",
                qualified_name
            )));
        }

        let blocks_to_copy: Vec<(BlockId, BasicBlock)> = self
            .blocks
            .iter()
            .filter_map(|(block_id, block)| {
                if let Some(func_name) = self.block_to_function.get(block_id) {
                    if func_name == &qualified_name {
                        Some((*block_id, block.clone()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        for (block_id, block) in blocks_to_copy {
            function.body.blocks.insert(block_id, block);
        }

        if let Some(contract) = self.contracts.get_mut(&contract_name) {
            contract
                .functions
                .insert(function.signature.name.clone(), function.clone());
        }

        self.functions.insert(qualified_name.clone(), function);
        self.function_to_contract
            .insert(qualified_name, contract_name);
        Ok(())
    }

    pub fn get_function(&self, qualified_name: &str) -> Option<&Function> {
        self.functions.get(qualified_name)
    }

    pub fn get_function_mut(&mut self, qualified_name: &str) -> Option<&mut Function> {
        self.functions.get_mut(qualified_name)
    }

    pub fn get_function_in_contract(&self, contract: &str, name: &str) -> Option<&Function> {
        let qualified_name = format!("{}::{}", contract, name);
        self.get_function(&qualified_name)
    }

    pub fn add_block(&mut self, function_name: String, block: BasicBlock) -> Result<()> {
        let block_id = block.id;

        if block_id == BlockId(0) {
            if let Some(function) = self.functions.get_mut(&function_name) {
                if let Some(existing_block) = function.body.blocks.get_mut(&block_id) {
                    existing_block.instructions = block.instructions.clone();
                    existing_block.terminator = block.terminator.clone();
                }
            }

            if let Some(contract_name) = function_name.split("::").next() {
                if let Some(contract) = self.contracts.get_mut(contract_name) {
                    if let Some(func_name) = function_name.split("::").nth(1) {
                        if let Some(function) = contract.functions.get_mut(func_name) {
                            if let Some(existing_block) = function.body.blocks.get_mut(&block_id) {
                                existing_block.instructions = block.instructions.clone();
                                existing_block.terminator = block.terminator.clone();
                            }
                        }
                    }
                }
            }

            self.blocks.insert(block_id, block);
            self.block_to_function.insert(block_id, function_name);
        } else {
            if self.blocks.contains_key(&block_id) {
                return Err(IrError::BuilderError(format!(
                    "Block {:?} already exists",
                    block_id
                )));
            }

            if let Some(function) = self.functions.get_mut(&function_name) {
                function.body.blocks.insert(block_id, block.clone());
            }

            if let Some(contract_name) = function_name.split("::").next() {
                if let Some(contract) = self.contracts.get_mut(contract_name) {
                    if let Some(func_name) = function_name.split("::").nth(1) {
                        if let Some(function) = contract.functions.get_mut(func_name) {
                            function.body.blocks.insert(block_id, block.clone());
                        }
                    }
                }
            }

            self.blocks.insert(block_id, block);
            self.block_to_function.insert(block_id, function_name);
        }

        Ok(())
    }

    pub fn get_block(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.get(&id)
    }

    pub fn get_block_mut(&mut self, id: BlockId) -> Option<&mut BasicBlock> {
        self.blocks.get_mut(&id)
    }

    pub fn add_instruction(&mut self, id: String, instruction: Instruction) -> Result<()> {
        if self.instructions.contains_key(&id) {
            return Err(IrError::BuilderError(format!(
                "Instruction {} already exists",
                id
            )));
        }
        self.instructions.insert(id, instruction);
        Ok(())
    }

    pub fn get_instruction(&self, id: &str) -> Option<&Instruction> {
        self.instructions.get(id)
    }

    pub fn add_value(&mut self, id: String, value: Value) -> Result<()> {
        if self.values.contains_key(&id) {
            return Err(IrError::BuilderError(format!(
                "Value {} already exists",
                id
            )));
        }
        self.values.insert(id, value);
        Ok(())
    }

    pub fn get_value(&self, id: &str) -> Option<&Value> {
        self.values.get(id)
    }

    pub fn get_function_contract(&self, function_name: &str) -> Option<&String> {
        self.function_to_contract.get(function_name)
    }

    pub fn get_block_function(&self, block_id: BlockId) -> Option<&String> {
        self.block_to_function.get(&block_id)
    }

    pub fn clear(&mut self) {
        self.contracts.clear();
        self.functions.clear();
        self.blocks.clear();
        self.instructions.clear();
        self.values.clear();
        self.function_to_contract.clear();
        self.block_to_function.clear();
    }

    pub fn validate(&self) -> Result<()> {
        for (func_name, contract_name) in &self.function_to_contract {
            if !self.contracts.contains_key(contract_name) {
                return Err(IrError::BuilderError(format!(
                    "Function {} references non-existent contract {}",
                    func_name, contract_name
                )));
            }
        }

        for (block_id, func_name) in &self.block_to_function {
            if !self.functions.contains_key(func_name) {
                return Err(IrError::BuilderError(format!(
                    "Block {:?} references non-existent function {}",
                    block_id, func_name
                )));
            }
        }

        Ok(())
    }

    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            contracts: self.contracts.len(),
            functions: self.functions.len(),
            blocks: self.blocks.len(),
            instructions: self.instructions.len(),
            values: self.values.len(),
        }
    }
}

#[derive(Debug)]
pub struct RegistryStats {
    pub contracts: usize,
    pub functions: usize,
    pub blocks: usize,
    pub instructions: usize,
    pub values: usize,
}
