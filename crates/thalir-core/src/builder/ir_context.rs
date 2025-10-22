use crate::{
    block::BlockId,
    types::TypeRegistry,
    values::{TempId, VarId},
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SourceMapping {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub length: u32,
}

#[derive(Debug, Default)]
pub struct SSATracker {
    variables: HashMap<String, VarId>,
    versions: HashMap<VarId, u32>,
    next_var_id: u32,
    next_temp_id: u32,
}

impl SSATracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_create_var(&mut self, name: &str) -> VarId {
        if let Some(&id) = self.variables.get(name) {
            id
        } else {
            let id = VarId(self.next_var_id);
            self.next_var_id += 1;
            self.variables.insert(name.to_string(), id);
            self.versions.insert(id, 0);
            id
        }
    }

    pub fn new_temp(&mut self) -> TempId {
        let id = TempId(self.next_temp_id);
        self.next_temp_id += 1;
        id
    }

    pub fn get_version(&self, var: VarId) -> u32 {
        self.versions.get(&var).copied().unwrap_or(0)
    }

    pub fn increment_version(&mut self, var: VarId) -> u32 {
        let version = self.get_version(var) + 1;
        self.versions.insert(var, version);
        version
    }
}

pub struct IRContext {
    source_mappings: HashMap<String, SourceMapping>,
    id_counter: u64,
    type_registry: TypeRegistry,
    ssa_tracker: SSATracker,
    current_contract: Option<String>,
    current_function: Option<String>,
    current_block: Option<BlockId>,
    errors: Vec<String>,
}

impl IRContext {
    pub fn new() -> Self {
        Self {
            source_mappings: HashMap::new(),
            id_counter: 1,
            type_registry: TypeRegistry::new(),
            ssa_tracker: SSATracker::new(),
            current_contract: None,
            current_function: None,
            current_block: None,
            errors: Vec::new(),
        }
    }

    pub fn next_id(&mut self) -> u64 {
        let id = self.id_counter;
        self.id_counter += 1;
        id
    }

    pub fn next_string_id(&mut self, prefix: &str) -> String {
        format!("{}_{}", prefix, self.next_id())
    }

    pub fn add_source_mapping(&mut self, id: String, mapping: SourceMapping) {
        self.source_mappings.insert(id, mapping);
    }

    pub fn get_source_mapping(&self, id: &str) -> Option<&SourceMapping> {
        self.source_mappings.get(id)
    }

    pub fn set_current_contract(&mut self, name: String) {
        self.current_contract = Some(name);
    }

    pub fn current_contract(&self) -> Option<&str> {
        self.current_contract.as_deref()
    }

    pub fn set_current_function(&mut self, name: String) {
        self.current_function = Some(name);
    }

    pub fn current_function(&self) -> Option<&str> {
        self.current_function.as_deref()
    }

    pub fn set_current_block(&mut self, block: BlockId) {
        self.current_block = Some(block);
    }

    pub fn current_block(&self) -> Option<BlockId> {
        self.current_block
    }

    pub fn ssa(&mut self) -> &mut SSATracker {
        &mut self.ssa_tracker
    }

    pub fn types(&mut self) -> &mut TypeRegistry {
        &mut self.type_registry
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    pub fn clear(&mut self) {
        self.source_mappings.clear();
        self.id_counter = 1;
        self.ssa_tracker = SSATracker::new();
        self.current_contract = None;
        self.current_function = None;
        self.current_block = None;
        self.errors.clear();
    }
}
