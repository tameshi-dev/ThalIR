use super::transform_solidity_to_ir;
use super::*;

#[test]
fn test_empty_contract_transformation() {
    let source = "contract Empty {}";
    let result = transform_solidity_to_ir(source);
    assert!(result.is_ok());
    let contracts = result.unwrap();
    assert_eq!(contracts.len(), 1);
    assert_eq!(contracts[0].name, "Empty");
}

#[test]
fn test_contract_with_state_variables() {
    let source = r#"
        contract Storage {
            uint256 public value;
            address owner;
            mapping(address => uint256) balances;
        }
    "#;
    let result = transform_solidity_to_ir(source);
    assert!(result.is_ok());
    let contracts = result.unwrap();
    assert_eq!(contracts.len(), 1);
    assert_eq!(contracts[0].name, "Storage");

    let state_vars = &contracts[0].storage_layout.slots;
    assert_eq!(state_vars.len(), 3);
    assert!(state_vars.iter().any(|v| v.name == "value"));
    assert!(state_vars.iter().any(|v| v.name == "owner"));
    assert!(state_vars.iter().any(|v| v.name == "balances"));
}

#[test]
fn test_simple_function() {
    let source = r#"
        contract Simple {
            function test() public pure returns (uint256) {
                return 42;
            }
        }
    "#;
    let result = transform_solidity_to_ir(source);
    assert!(result.is_ok());
    let contracts = result.unwrap();
    assert_eq!(contracts.len(), 1);

    let contract = &contracts[0];
    assert_eq!(contract.name, "Simple");
    assert_eq!(contract.functions.len(), 1);

    let func = contract.functions.values().next().unwrap();
    assert_eq!(func.signature.name, "test");
    assert_eq!(func.visibility, thalir_core::function::Visibility::Public);
    assert_eq!(func.mutability, thalir_core::function::Mutability::Pure);
}

#[test]
fn test_function_with_parameters() {
    let source = r#"
        contract Calculator {
            function add(uint256 a, uint256 b) public pure returns (uint256) {
                return a + b;
            }
        }
    "#;
    let result = transform_solidity_to_ir(source);
    assert!(result.is_ok());
    let contracts = result.unwrap();
    assert_eq!(contracts.len(), 1);

    let func = contracts[0].functions.values().next().unwrap();

    assert_eq!(func.signature.name, "add_uint256_uint256");
    assert_eq!(func.signature.params.len(), 2);
    assert_eq!(func.signature.params[0].name, "a");
    assert_eq!(func.signature.params[1].name, "b");
}

#[test]
fn test_multiple_functions() {
    let source = r#"
        contract MultiFunction {
            function foo() public pure {}
            function bar() external view {}
            function baz() internal {}
        }
    "#;
    let result = transform_solidity_to_ir(source);
    assert!(result.is_ok());
    let contracts = result.unwrap();
    assert_eq!(contracts.len(), 1);
    assert_eq!(contracts[0].functions.len(), 3);

    let funcs = &contracts[0].functions;
    assert!(funcs
        .values()
        .any(|f| f.signature.name == "foo"
            && f.visibility == thalir_core::function::Visibility::Public));
    assert!(funcs.values().any(|f| f.signature.name == "bar"
        && f.visibility == thalir_core::function::Visibility::External));
    assert!(funcs.values().any(|f| f.signature.name == "baz"
        && f.visibility == thalir_core::function::Visibility::Internal));
}

#[test]
fn test_constructor() {
    let source = r#"
        contract WithConstructor {
            uint256 public value;

            constructor(uint256 _value) {
                value = _value;
            }
        }
    "#;
    let result = transform_solidity_to_ir(source);
    assert!(result.is_ok());
    let contracts = result.unwrap();
    assert_eq!(contracts.len(), 1);

    let funcs = &contracts[0].functions;
    assert!(funcs.values().any(|f| f.signature.params.len() == 1));
}

#[test]
fn test_different_types() {
    let source = r#"
        contract Types {
            bool public flag;
            uint8 public small;
            uint256 public large;
            address public owner;
            bytes32 public hash;
            string public name;
        }
    "#;
    let result = transform_solidity_to_ir(source);
    assert!(result.is_ok());
    let contracts = result.unwrap();
    assert_eq!(contracts.len(), 1);

    let state_vars = &contracts[0].storage_layout.slots;
    assert_eq!(state_vars.len(), 6);

    assert!(state_vars
        .iter()
        .any(|v| v.name == "flag" && matches!(v.var_type, thalir_core::types::Type::Bool)));
    assert!(state_vars
        .iter()
        .any(|v| v.name == "small" && matches!(v.var_type, thalir_core::types::Type::Uint(8))));
    assert!(state_vars
        .iter()
        .any(|v| v.name == "large" && matches!(v.var_type, thalir_core::types::Type::Uint(256))));
    assert!(state_vars
        .iter()
        .any(|v| v.name == "owner" && matches!(v.var_type, thalir_core::types::Type::Address)));
    assert!(state_vars
        .iter()
        .any(|v| v.name == "hash" && matches!(v.var_type, thalir_core::types::Type::Bytes32)));
    assert!(state_vars
        .iter()
        .any(|v| v.name == "name" && matches!(v.var_type, thalir_core::types::Type::String)));
}

#[test]
fn test_array_and_mapping_types() {
    let source = r#"
        contract Complex {
            uint256[] public numbers;
            mapping(address => uint256) public balances;
            mapping(uint256 => address) public owners;
        }
    "#;
    let result = transform_solidity_to_ir(source);
    assert!(result.is_ok());
    let contracts = result.unwrap();
    assert_eq!(contracts.len(), 1);

    let state_vars = &contracts[0].storage_layout.slots;
    assert_eq!(state_vars.len(), 3);

    assert!(state_vars.iter().any(|v| v.name == "numbers" &&
        matches!(&v.var_type, thalir_core::types::Type::Array(elem, None) if matches!(elem.as_ref(), thalir_core::types::Type::Uint(256)))));

    assert!(state_vars.iter().any(|v| v.name == "balances"
        && matches!(&v.var_type, thalir_core::types::Type::Mapping(k, v) if
            matches!(k.as_ref(), thalir_core::types::Type::Address) &&
            matches!(v.as_ref(), thalir_core::types::Type::Uint(256)))));
}

#[test]
fn test_interface() {
    let source = r#"
        interface IToken {
            function transfer(address to, uint256 amount) external;
            function balanceOf(address owner) external view returns (uint256);
        }
    "#;
    let result = transform_solidity_to_ir(source);
    assert!(result.is_ok());
    let contracts = result.unwrap();
    assert_eq!(contracts.len(), 1);
    assert_eq!(contracts[0].name, "IToken");
    assert_eq!(contracts[0].functions.len(), 2);
}

#[test]
fn test_library() {
    let source = r#"
        library Math {
            function add(uint256 a, uint256 b) internal pure returns (uint256) {
                return a + b;
            }
        }
    "#;
    let result = transform_solidity_to_ir(source);
    assert!(result.is_ok());
    let contracts = result.unwrap();
    assert_eq!(contracts.len(), 1);
    assert_eq!(contracts[0].name, "Math");
    assert_eq!(contracts[0].functions.len(), 1);
}

#[test]
fn test_visibility_modifiers() {
    let source = r#"
        contract Visibility {
            function publicFunc() public {}
            function externalFunc() external {}
            function internalFunc() internal {}
            function privateFunc() private {}
        }
    "#;
    let result = transform_solidity_to_ir(source);
    assert!(result.is_ok());
    let contracts = result.unwrap();

    let funcs = &contracts[0].functions;
    assert_eq!(funcs.len(), 4);

    assert!(funcs.values().any(|f| f.signature.name == "publicFunc"
        && f.visibility == thalir_core::function::Visibility::Public));
    assert!(funcs.values().any(|f| f.signature.name == "externalFunc"
        && f.visibility == thalir_core::function::Visibility::External));
    assert!(funcs.values().any(|f| f.signature.name == "internalFunc"
        && f.visibility == thalir_core::function::Visibility::Internal));
    assert!(funcs.values().any(|f| f.signature.name == "privateFunc"
        && f.visibility == thalir_core::function::Visibility::Private));
}

#[test]
fn test_mutability_modifiers() {
    let source = r#"
        contract Mutability {
            function pureFunc() public pure {}
            function viewFunc() public view {}
            function payableFunc() public payable {}
            function normalFunc() public {}
        }
    "#;
    let result = transform_solidity_to_ir(source);
    assert!(result.is_ok());
    let contracts = result.unwrap();

    let funcs = &contracts[0].functions;
    assert_eq!(funcs.len(), 4);

    assert!(funcs.values().any(|f| f.signature.name == "pureFunc"
        && f.mutability == thalir_core::function::Mutability::Pure));
    assert!(funcs.values().any(|f| f.signature.name == "viewFunc"
        && f.mutability == thalir_core::function::Mutability::View));
    assert!(funcs.values().any(|f| f.signature.name == "payableFunc"
        && f.mutability == thalir_core::function::Mutability::Payable));
    assert!(funcs.values().any(|f| f.signature.name == "normalFunc"
        && f.mutability == thalir_core::function::Mutability::NonPayable));
}
