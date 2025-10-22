use super::ObfuscationConfig;

pub struct StringSanitizer {
    config: ObfuscationConfig,
    error_counter: usize,
}

impl StringSanitizer {
    pub fn new(config: ObfuscationConfig) -> Self {
        Self {
            config,
            error_counter: 0,
        }
    }

    pub fn sanitize_string(&mut self, s: &str) -> String {
        if self.is_security_relevant(s) {
            return s.to_string();
        }

        if self.config.strip_string_constants || self.config.strip_error_messages {
            let result = format!("error_{}", self.error_counter);
            self.error_counter += 1;
            result
        } else {
            s.to_string()
        }
    }

    fn is_security_relevant(&self, s: &str) -> bool {
        if s.is_empty() {
            return true;
        }

        if s.starts_with("0x") && s.len() > 2 {
            let hex_part = &s[2..];

            if hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
                return matches!(hex_part.len(), 8 | 40 | 64);
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_error_messages() {
        let config = ObfuscationConfig {
            strip_error_messages: true,
            ..Default::default()
        };

        let mut sanitizer = StringSanitizer::new(config);

        assert_eq!(
            sanitizer.sanitize_string("Invalid bonding curve parameters"),
            "error_0"
        );
        assert_eq!(sanitizer.sanitize_string("Insufficient balance"), "error_1");
        assert_eq!(sanitizer.sanitize_string("Transfer failed"), "error_2");
    }

    #[test]
    fn test_preserve_addresses() {
        let config = ObfuscationConfig {
            strip_string_constants: true,
            ..Default::default()
        };

        let mut sanitizer = StringSanitizer::new(config);

        let addr = "0x1234567890abcdef1234567890abcdef12345678";
        assert_eq!(sanitizer.sanitize_string(addr), addr);
    }

    #[test]
    fn test_preserve_transaction_hashes() {
        let config = ObfuscationConfig {
            strip_string_constants: true,
            ..Default::default()
        };

        let mut sanitizer = StringSanitizer::new(config);

        let hash = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        assert_eq!(sanitizer.sanitize_string(hash), hash);
    }

    #[test]
    fn test_preserve_function_selectors() {
        let config = ObfuscationConfig {
            strip_string_constants: true,
            ..Default::default()
        };

        let mut sanitizer = StringSanitizer::new(config);

        let selector = "0xa9059cbb";
        assert_eq!(sanitizer.sanitize_string(selector), selector);
    }

    #[test]
    fn test_preserve_empty_strings() {
        let config = ObfuscationConfig {
            strip_string_constants: true,
            ..Default::default()
        };

        let mut sanitizer = StringSanitizer::new(config);

        assert_eq!(sanitizer.sanitize_string(""), "");
    }

    #[test]
    fn test_no_sanitization_when_disabled() {
        let config = ObfuscationConfig {
            strip_string_constants: false,
            strip_error_messages: false,
            ..Default::default()
        };

        let mut sanitizer = StringSanitizer::new(config);

        let msg = "Some error message";
        assert_eq!(sanitizer.sanitize_string(msg), msg);
    }

    #[test]
    fn test_non_hex_strings_sanitized() {
        let config = ObfuscationConfig {
            strip_string_constants: true,
            ..Default::default()
        };

        let mut sanitizer = StringSanitizer::new(config);

        assert_eq!(sanitizer.sanitize_string("0xGGGG"), "error_0");

        assert_eq!(sanitizer.sanitize_string("0xabcd"), "error_1");
    }
}
