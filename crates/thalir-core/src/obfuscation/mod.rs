/*! Strip names and metadata for confidential auditing.
 *
 * Proprietary code needs auditing but can't be shared openly. Hash identifiers and remove metadata
 * while preserving security-relevant behavior, then map findings back to original names when reporting
 * vulnerabilities.
 */

pub mod deobfuscator;
pub mod mapping_store;
pub mod name_obfuscator;
pub mod pass;
pub mod string_sanitizer;

pub use deobfuscator::VulnerabilityMapper;
pub use mapping_store::{MappingMetadata, ObfuscationMapping};
pub use name_obfuscator::NameObfuscator;
pub use pass::ObfuscationPass;
pub use string_sanitizer::StringSanitizer;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObfuscationLevel {
    None,
    Minimal,
    Standard,
}

impl Default for ObfuscationLevel {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObfuscationConfig {
    pub level: ObfuscationLevel,
    pub retain_mapping: bool,
    pub hash_salt: Option<String>,
    pub strip_string_constants: bool,
    pub strip_error_messages: bool,
    pub strip_metadata: bool,
}

impl Default for ObfuscationConfig {
    fn default() -> Self {
        Self {
            level: ObfuscationLevel::None,
            retain_mapping: false,
            hash_salt: None,
            strip_string_constants: false,
            strip_error_messages: false,
            strip_metadata: false,
        }
    }
}

impl ObfuscationConfig {
    pub fn standard() -> Self {
        Self {
            level: ObfuscationLevel::Standard,
            retain_mapping: true,
            hash_salt: None,
            strip_string_constants: true,
            strip_error_messages: true,
            strip_metadata: true,
        }
    }

    pub fn minimal() -> Self {
        Self {
            level: ObfuscationLevel::Minimal,
            retain_mapping: true,
            hash_salt: None,
            strip_string_constants: false,
            strip_error_messages: false,
            strip_metadata: false,
        }
    }
}
