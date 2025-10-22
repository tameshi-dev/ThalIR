use crate::contract::Contract;
use crate::source_location::SourceFiles;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn save_contract(contract: &Contract, path: impl AsRef<Path>) -> io::Result<()> {
    let json = serde_json::to_string_pretty(contract)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    fs::write(path, json)?;
    Ok(())
}

pub fn load_contract(path: impl AsRef<Path>) -> io::Result<Contract> {
    let json = fs::read_to_string(path)?;
    let contract =
        serde_json::from_str(&json).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(contract)
}

pub fn generate_ir_index(contract: &Contract) -> IRIndex {
    let mut index = IRIndex::new();

    for (func_name, function) in &contract.functions {
        for (block_id, block) in &function.body.blocks {
            for (inst_idx, instruction) in block.instructions.iter().enumerate() {
                index.add_instruction(
                    func_name.clone(),
                    block_id.to_string(),
                    inst_idx,
                    instruction.clone(),
                );
            }
        }
    }

    index
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFilesData {
    pub files: Vec<SourceFileData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFileData {
    pub id: u32,
    pub path: PathBuf,
    pub content: String,
    pub line_starts: Vec<usize>,
}

pub fn save_source_files(source_files: &SourceFiles, path: impl AsRef<Path>) -> io::Result<()> {
    let files_data = extract_source_files_data(source_files);

    let json = serde_json::to_string_pretty(&files_data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    fs::write(path, json)?;
    Ok(())
}

pub fn load_source_files(path: impl AsRef<Path>) -> io::Result<SourceFilesData> {
    let json = fs::read_to_string(path)?;
    let data =
        serde_json::from_str(&json).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(data)
}

fn extract_source_files_data(source_files: &SourceFiles) -> SourceFilesData {
    let source_files_list = source_files.get_all_files();

    let files: Vec<SourceFileData> = source_files_list
        .iter()
        .enumerate()
        .map(|(id, file)| SourceFileData {
            id: id as u32,
            path: file.path.clone(),
            content: file.text.as_ref().clone(),
            line_starts: file.line_starts.as_ref().clone(),
        })
        .collect();

    SourceFilesData { files }
}

#[derive(Debug, Clone)]
pub struct IRIndex {
    pub instructions:
        std::collections::HashMap<InstructionLocation, crate::instructions::Instruction>,
}

impl IRIndex {
    pub fn new() -> Self {
        Self {
            instructions: std::collections::HashMap::new(),
        }
    }

    pub fn add_instruction(
        &mut self,
        function: String,
        block: String,
        index: usize,
        instruction: crate::instructions::Instruction,
    ) {
        let location = InstructionLocation {
            function,
            block,
            index,
        };
        self.instructions.insert(location, instruction);
    }

    pub fn get_instruction(
        &self,
        function: &str,
        block: &str,
        index: usize,
    ) -> Option<&crate::instructions::Instruction> {
        let location = InstructionLocation {
            function: function.to_string(),
            block: block.to_string(),
            index,
        };
        self.instructions.get(&location)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InstructionLocation {
    pub function: String,
    pub block: String,
    pub index: usize,
}

impl Default for IRIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Contract;
    use tempfile::NamedTempFile;

    #[test]
    fn test_save_load_contract() {
        let contract = Contract::new("TestContract".to_string());
        let temp_file = NamedTempFile::new().unwrap();

        save_contract(&contract, temp_file.path()).unwrap();

        let loaded = load_contract(temp_file.path()).unwrap();
        assert_eq!(loaded.name, "TestContract");
    }
}
