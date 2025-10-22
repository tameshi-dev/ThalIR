use thalir_parser;

#[test]
fn test_parse_cli_generated_simple_storage() {
    let input = r#"
; Contract: SimpleStorage
; Version: 0.8.0

function %set(i256) {
    ; Visibility: Public
    ; Mutability: NonPayable

block0(v0: i256):
    storage_store slot0, v0
    return
}

function %get() -> i256 {
    ; Visibility: Public
    ; Mutability: View

block0:
    v0 = storage_load slot0
    return v0
}
"#;

    match thalir_parser::parse(input) {
        Ok(_) => {}
        Err(e) => panic!("Failed to parse CLI-generated IR: {}", e),
    }
}

#[test]
fn test_parse_cli_generated_erc20_style() {
    let input = r#"
; Contract: TokenLike
; Version: 0.8.0

function %transfer(address, i256) -> i1 {
    ; Visibility: Public
    ; Mutability: NonPayable

block0(v0: address, v1: i256):
    v2 = get_context msg.sender
    v3 = mapping_load map0, v2
    v4 = icmp uge v3, v1
    require v4, "insufficient balance"
    v5 = isub.i256 v3, v1
    mapping_store map0, v2, v5
    v6 = mapping_load map0, v0
    v7 = iadd.i256 v6, v1
    mapping_store map0, v0, v7
    v8 = iconst.i1 1
    return v8
}
"#;

    match thalir_parser::parse(input) {
        Ok(_) => {}
        Err(e) => panic!("Failed to parse CLI-generated ERC20-style IR: {}", e),
    }
}

#[test]
fn test_parse_with_all_visibility_mutability() {
    let input = r#"
function %publicFunc() {
    ; Visibility: Public
    ; Mutability: NonPayable

block0:
    return
}

function %viewFunc() -> i256 {
    ; Visibility: Public
    ; Mutability: View

block0:
    v0 = iconst.i256 42
    return v0
}

function %payableFunc() {
    ; Visibility: Public
    ; Mutability: Payable

block0:
    v0 = get_context msg.value
    storage_store slot0, v0
    return
}
"#;

    assert!(thalir_parser::parse(input).is_ok());
}
