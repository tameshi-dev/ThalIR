/*! Transform Solidity source into auditable IR.
 *
 * Solidity syntax varies wildly and obscures security-critical patterns. This transformer converts
 * source code into canonical IR where storage operations, external calls, and overflow semantics are
 * explicit and uniform—making vulnerability detection systematic rather than ad-hoc.
 */

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_assignments)]
#![allow(unreachable_patterns)]

pub mod solidity_to_ir;

pub use solidity_to_ir::{transform_solidity_to_ir, transform_solidity_to_ir_with_filename};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        use tree_sitter::{Language, Parser};

        let mut parser = Parser::new();
        let language: Language = tree_sitter_solidity::LANGUAGE.into();

        parser
            .set_language(&language)
            .expect("Failed to set language");

        let code = "contract Test {}";
        let tree = parser.parse(code, None).expect("Failed to parse");

        assert_eq!(tree.root_node().kind(), "source_file");
    }

    #[test]
    fn test_basic_transformation() {
        let solidity_code = r#"
pragma solidity ^0.8.0;

contract SimpleStorage {
    uint256 private value;

    function setValue(uint256 _value) public {
        value = _value;
    }

    function getValue() public view returns (uint256) {
        return value;
    }
}
"#;

        let result = transform_solidity_to_ir(solidity_code);

        match result {
            Ok(contracts) => {
                assert!(!contracts.is_empty(), "Should have at least one contract");
                assert_eq!(contracts[0].name, "SimpleStorage");
            }
            Err(e) => {
                panic!("Transformation failed: {}", e);
            }
        }
    }
}
