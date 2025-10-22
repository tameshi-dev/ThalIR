use super::{ObfuscationConfig, ObfuscationLevel};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub struct NameObfuscator {
    config: ObfuscationConfig,
    mapping: HashMap<String, String>,
    reverse_mapping: HashMap<String, String>,
    contract_counter: usize,
    function_counter: usize,
    storage_counter: usize,
}

impl NameObfuscator {
    pub fn new(config: ObfuscationConfig) -> Self {
        Self {
            config,
            mapping: HashMap::new(),
            reverse_mapping: HashMap::new(),
            contract_counter: 0,
            function_counter: 0,
            storage_counter: 0,
        }
    }

    pub fn obfuscate_contract_name(&mut self, name: &str) -> String {
        if let Some(obfuscated) = self.mapping.get(name) {
            return obfuscated.clone();
        }

        let obfuscated = match self.config.level {
            ObfuscationLevel::None => name.to_string(),
            ObfuscationLevel::Minimal => {
                let result = format!("contract_{}", self.contract_counter);
                self.contract_counter += 1;
                result
            }
            ObfuscationLevel::Standard => self.hash_name(name, "c"),
        };

        if self.config.retain_mapping {
            self.mapping.insert(name.to_string(), obfuscated.clone());
            self.reverse_mapping
                .insert(obfuscated.clone(), name.to_string());
        }

        obfuscated
    }

    pub fn obfuscate_function_name(&mut self, name: &str) -> String {
        if let Some(obfuscated) = self.mapping.get(name) {
            return obfuscated.clone();
        }

        let obfuscated = match self.config.level {
            ObfuscationLevel::None => name.to_string(),
            ObfuscationLevel::Minimal => {
                let result = format!("fn_{}", self.function_counter);
                self.function_counter += 1;
                result
            }
            ObfuscationLevel::Standard => self.hash_name(name, "f"),
        };

        if self.config.retain_mapping {
            self.mapping.insert(name.to_string(), obfuscated.clone());
            self.reverse_mapping
                .insert(obfuscated.clone(), name.to_string());
        }

        obfuscated
    }

    pub fn obfuscate_storage_name(&mut self, name: &str) -> String {
        if let Some(obfuscated) = self.mapping.get(name) {
            return obfuscated.clone();
        }

        let obfuscated = match self.config.level {
            ObfuscationLevel::None => name.to_string(),
            ObfuscationLevel::Minimal => {
                let result = format!("var_{}", self.storage_counter);
                self.storage_counter += 1;
                result
            }
            ObfuscationLevel::Standard => self.hash_name(name, "v"),
        };

        if self.config.retain_mapping {
            self.mapping.insert(name.to_string(), obfuscated.clone());
            self.reverse_mapping
                .insert(obfuscated.clone(), name.to_string());
        }

        obfuscated
    }

    fn hash_name(&self, name: &str, prefix: &str) -> String {
        let mut hasher = Sha256::new();

        if let Some(salt) = &self.config.hash_salt {
            hasher.update(salt.as_bytes());
        }

        hasher.update(name.as_bytes());
        let hash = hasher.finalize();

        format!("{}_{:02x}{:02x}{:02x}", prefix, hash[0], hash[1], hash[2])
    }

    pub fn export_mapping(&self) -> HashMap<String, String> {
        self.reverse_mapping.clone()
    }

    pub fn import_mapping(&mut self, mapping: HashMap<String, String>) {
        for (obfuscated, original) in mapping {
            self.reverse_mapping
                .insert(obfuscated.clone(), original.clone());
            self.mapping.insert(original, obfuscated);
        }
    }

    pub fn deobfuscate(&self, obfuscated: &str) -> Option<&str> {
        self.reverse_mapping.get(obfuscated).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_obfuscation() {
        let config = ObfuscationConfig {
            level: ObfuscationLevel::Minimal,
            retain_mapping: true,
            ..Default::default()
        };

        let mut obfuscator = NameObfuscator::new(config);

        assert_eq!(obfuscator.obfuscate_contract_name("MyToken"), "contract_0");
        assert_eq!(
            obfuscator.obfuscate_contract_name("AnotherContract"),
            "contract_1"
        );
        assert_eq!(obfuscator.obfuscate_function_name("transfer"), "fn_0");
        assert_eq!(obfuscator.obfuscate_function_name("approve"), "fn_1");
        assert_eq!(obfuscator.obfuscate_storage_name("balances"), "var_0");
        assert_eq!(obfuscator.obfuscate_storage_name("allowances"), "var_1");
    }

    #[test]
    fn test_standard_obfuscation() {
        let config = ObfuscationConfig {
            level: ObfuscationLevel::Standard,
            retain_mapping: true,
            hash_salt: Some("test-salt".to_string()),
            ..Default::default()
        };

        let mut obfuscator = NameObfuscator::new(config);

        let obf_contract = obfuscator.obfuscate_contract_name("MyToken");
        assert!(obf_contract.starts_with("c_"));
        assert_eq!(obf_contract.len(), 8);

        let obf_contract2 = obfuscator.obfuscate_contract_name("MyToken");
        assert_eq!(obf_contract, obf_contract2);
    }

    #[test]
    fn test_deterministic_hashing() {
        let config = ObfuscationConfig {
            level: ObfuscationLevel::Standard,
            retain_mapping: true,
            hash_salt: Some("fixed-salt".to_string()),
            ..Default::default()
        };

        let mut obfuscator1 = NameObfuscator::new(config.clone());
        let mut obfuscator2 = NameObfuscator::new(config);

        let name1 = obfuscator1.obfuscate_contract_name("TestContract");
        let name2 = obfuscator2.obfuscate_contract_name("TestContract");

        assert_eq!(name1, name2, "Same salt should produce same hash");
    }

    #[test]
    fn test_different_salts_produce_different_hashes() {
        let config1 = ObfuscationConfig {
            level: ObfuscationLevel::Standard,
            retain_mapping: true,
            hash_salt: Some("salt1".to_string()),
            ..Default::default()
        };

        let config2 = ObfuscationConfig {
            level: ObfuscationLevel::Standard,
            retain_mapping: true,
            hash_salt: Some("salt2".to_string()),
            ..Default::default()
        };

        let mut obfuscator1 = NameObfuscator::new(config1);
        let mut obfuscator2 = NameObfuscator::new(config2);

        let name1 = obfuscator1.obfuscate_contract_name("TestContract");
        let name2 = obfuscator2.obfuscate_contract_name("TestContract");

        assert_ne!(
            name1, name2,
            "Different salts should produce different hashes"
        );
    }

    #[test]
    fn test_deobfuscation() {
        let config = ObfuscationConfig {
            level: ObfuscationLevel::Minimal,
            retain_mapping: true,
            ..Default::default()
        };

        let mut obfuscator = NameObfuscator::new(config);
        let obf_name = obfuscator.obfuscate_contract_name("NovelBondingCurve");

        assert_eq!(obfuscator.deobfuscate(&obf_name), Some("NovelBondingCurve"));
    }

    #[test]
    fn test_export_import_mapping() {
        let config = ObfuscationConfig {
            level: ObfuscationLevel::Standard,
            retain_mapping: true,
            ..Default::default()
        };

        let mut obfuscator = NameObfuscator::new(config.clone());
        obfuscator.obfuscate_contract_name("Contract1");
        obfuscator.obfuscate_function_name("function1");

        let mapping = obfuscator.export_mapping();

        let mut new_obfuscator = NameObfuscator::new(config);
        new_obfuscator.import_mapping(mapping);

        assert!(
            new_obfuscator.deobfuscate("c_").is_some() || new_obfuscator.reverse_mapping.len() > 0
        );
    }

    #[test]
    fn test_none_level_preserves_names() {
        let config = ObfuscationConfig {
            level: ObfuscationLevel::None,
            retain_mapping: false,
            ..Default::default()
        };

        let mut obfuscator = NameObfuscator::new(config);

        assert_eq!(
            obfuscator.obfuscate_contract_name("MyContract"),
            "MyContract"
        );
        assert_eq!(
            obfuscator.obfuscate_function_name("myFunction"),
            "myFunction"
        );
        assert_eq!(obfuscator.obfuscate_storage_name("myVar"), "myVar");
    }
}
