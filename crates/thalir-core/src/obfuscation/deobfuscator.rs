use super::ObfuscationMapping;
use std::collections::HashMap;

pub struct VulnerabilityMapper {
    mapping: HashMap<String, String>,
}

impl VulnerabilityMapper {
    pub fn from_mapping(mapping: ObfuscationMapping) -> Self {
        Self {
            mapping: mapping.mapping,
        }
    }

    pub fn deobfuscate_identifier(&self, obfuscated: &str) -> Option<String> {
        self.mapping.get(obfuscated).cloned()
    }

    pub fn deobfuscate_report(&self, report: &str) -> String {
        let mut result = report.to_string();

        let mut mappings: Vec<_> = self.mapping.iter().collect();
        mappings.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        for (obfuscated, original) in mappings {
            result = result.replace(obfuscated, original);
        }

        result
    }

    pub fn deobfuscate_reports(&self, reports: &[String]) -> Vec<String> {
        reports.iter().map(|r| self.deobfuscate_report(r)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obfuscation::MappingMetadata;

    fn create_test_mapping() -> ObfuscationMapping {
        let mut mapping = HashMap::new();
        mapping.insert("contract_0".to_string(), "NovelBondingCurve".to_string());
        mapping.insert("fn_0".to_string(), "calculateBondingCurve".to_string());
        mapping.insert("var_0".to_string(), "liquidityPoolReserves".to_string());
        mapping.insert("fn_1".to_string(), "transfer".to_string());

        ObfuscationMapping {
            mapping,
            metadata: MappingMetadata {
                created_at: "2024-01-01T00:00:00Z".to_string(),
                obfuscation_level: "minimal".to_string(),
                hash_salt: None,
            },
        }
    }

    #[test]
    fn test_deobfuscate_identifier() {
        let mapping = create_test_mapping();
        let mapper = VulnerabilityMapper::from_mapping(mapping);

        assert_eq!(
            mapper.deobfuscate_identifier("contract_0"),
            Some("NovelBondingCurve".to_string())
        );
        assert_eq!(
            mapper.deobfuscate_identifier("fn_0"),
            Some("calculateBondingCurve".to_string())
        );
        assert_eq!(
            mapper.deobfuscate_identifier("var_0"),
            Some("liquidityPoolReserves".to_string())
        );
        assert_eq!(mapper.deobfuscate_identifier("unknown"), None);
    }

    #[test]
    fn test_deobfuscate_simple_report() {
        let mapping = create_test_mapping();
        let mapper = VulnerabilityMapper::from_mapping(mapping);

        let obfuscated = "Reentrancy in contract_0::fn_0 at position [7]";
        let expected = "Reentrancy in NovelBondingCurve::calculateBondingCurve at position [7]";

        assert_eq!(mapper.deobfuscate_report(obfuscated), expected);
    }

    #[test]
    fn test_deobfuscate_complex_report() {
        let mapping = create_test_mapping();
        let mapper = VulnerabilityMapper::from_mapping(mapping);

        let obfuscated = "Function fn_1 in contract_0 accesses var_0 unsafely";
        let expected =
            "Function transfer in NovelBondingCurve accesses liquidityPoolReserves unsafely";

        assert_eq!(mapper.deobfuscate_report(obfuscated), expected);
    }

    #[test]
    fn test_deobfuscate_multiple_reports() {
        let mapping = create_test_mapping();
        let mapper = VulnerabilityMapper::from_mapping(mapping);

        let reports = vec![
            "Issue in fn_0".to_string(),
            "Problem with var_0".to_string(),
            "contract_0 has vulnerability".to_string(),
        ];

        let deobfuscated = mapper.deobfuscate_reports(&reports);

        assert_eq!(deobfuscated[0], "Issue in calculateBondingCurve");
        assert_eq!(deobfuscated[1], "Problem with liquidityPoolReserves");
        assert_eq!(deobfuscated[2], "NovelBondingCurve has vulnerability");
    }

    #[test]
    fn test_deobfuscate_preserves_unknown_identifiers() {
        let mapping = create_test_mapping();
        let mapper = VulnerabilityMapper::from_mapping(mapping);

        let obfuscated = "Unknown identifier unknown_fn in contract_0";
        let result = mapper.deobfuscate_report(obfuscated);

        assert!(result.contains("NovelBondingCurve"));
        assert!(result.contains("unknown_fn"));
    }

    #[test]
    fn test_deobfuscate_handles_position_markers() {
        let mapping = create_test_mapping();
        let mapper = VulnerabilityMapper::from_mapping(mapping);

        let obfuscated = "At [5] in fn_0, call to contract_0::fn_1 detected";
        let result = mapper.deobfuscate_report(obfuscated);

        assert_eq!(
            result,
            "At [5] in calculateBondingCurve, call to NovelBondingCurve::transfer detected"
        );
    }
}
