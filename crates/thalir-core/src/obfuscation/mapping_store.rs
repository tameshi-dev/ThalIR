use super::NameObfuscator;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObfuscationMapping {
    pub mapping: HashMap<String, String>,
    pub metadata: MappingMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingMetadata {
    pub created_at: String,
    pub obfuscation_level: String,
    pub hash_salt: Option<String>,
}

impl ObfuscationMapping {
    pub fn from_obfuscator(obfuscator: &NameObfuscator) -> Self {
        let mapping = obfuscator.export_mapping();

        Self {
            mapping,
            metadata: MappingMetadata {
                created_at: chrono::Utc::now().to_rfc3339(),
                obfuscation_level: "standard".to_string(),
                hash_salt: None,
            },
        }
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let mapping: ObfuscationMapping = serde_json::from_str(&json)?;
        Ok(mapping)
    }

    pub fn deobfuscate(&self, obfuscated: &str) -> Option<&str> {
        self.mapping.get(obfuscated).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obfuscation::{ObfuscationConfig, ObfuscationLevel};
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_mapping_serialization() {
        let mut mapping = HashMap::new();
        mapping.insert("contract_0".to_string(), "MyContract".to_string());
        mapping.insert("fn_0".to_string(), "transfer".to_string());

        let obf_mapping = ObfuscationMapping {
            mapping,
            metadata: MappingMetadata {
                created_at: "2024-01-01T00:00:00Z".to_string(),
                obfuscation_level: "minimal".to_string(),
                hash_salt: None,
            },
        };

        let json = serde_json::to_string_pretty(&obf_mapping).unwrap();
        assert!(json.contains("MyContract"));
        assert!(json.contains("transfer"));

        let deserialized: ObfuscationMapping = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.deobfuscate("contract_0"), Some("MyContract"));
        assert_eq!(deserialized.deobfuscate("fn_0"), Some("transfer"));
    }

    #[test]
    fn test_file_save_and_load() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();

        let mut mapping = HashMap::new();
        mapping.insert("c_abc123".to_string(), "TestContract".to_string());
        mapping.insert("f_def456".to_string(), "testFunction".to_string());

        let obf_mapping = ObfuscationMapping {
            mapping,
            metadata: MappingMetadata {
                created_at: "2024-01-01T00:00:00Z".to_string(),
                obfuscation_level: "standard".to_string(),
                hash_salt: Some("test-salt".to_string()),
            },
        };

        obf_mapping.save_to_file(temp_path).unwrap();

        let loaded = ObfuscationMapping::load_from_file(temp_path).unwrap();

        assert_eq!(loaded.deobfuscate("c_abc123"), Some("TestContract"));
        assert_eq!(loaded.deobfuscate("f_def456"), Some("testFunction"));
        assert_eq!(loaded.metadata.hash_salt, Some("test-salt".to_string()));
    }

    #[test]
    fn test_from_obfuscator() {
        let config = ObfuscationConfig {
            level: ObfuscationLevel::Minimal,
            retain_mapping: true,
            ..Default::default()
        };

        let mut obfuscator = NameObfuscator::new(config);
        obfuscator.obfuscate_contract_name("Contract1");
        obfuscator.obfuscate_function_name("function1");
        obfuscator.obfuscate_storage_name("storage1");

        let mapping = ObfuscationMapping::from_obfuscator(&obfuscator);

        assert_eq!(mapping.deobfuscate("contract_0"), Some("Contract1"));
        assert_eq!(mapping.deobfuscate("fn_0"), Some("function1"));
        assert_eq!(mapping.deobfuscate("var_0"), Some("storage1"));
    }
}
