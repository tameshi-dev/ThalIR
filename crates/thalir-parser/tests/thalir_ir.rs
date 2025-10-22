use thalir_parser;

#[test]
fn test_thalir_storage_operations() {
    let input = r#"
function %test_storage(i256) -> i256 {
block0(v0: i256):
    v1 = storage_load slot0
    v2 = iadd.i256 v1, v0
    storage_store slot0, v2
    return v2
}
"#;
    assert!(thalir_parser::parse(input).is_ok());
}

#[test]
fn test_thalir_mapping_operations() {
    let input = r#"
function %test_mapping(i256, i256) -> i256 {
block0(v0: i256, v1: i256):
    v2 = mapping_load map0, v0
    v3 = iadd.i256 v2, v1
    mapping_store map0, v0, v3
    return v3
}
"#;
    assert!(thalir_parser::parse(input).is_ok());
}

#[test]
fn test_thalir_context_variables() {
    let input = r#"
function %test_context() -> i256 {
block0:
    v0 = get_context msg.sender
    v1 = get_context msg.value
    v2 = get_context block.timestamp
    v3 = iadd.i256 v0, v1
    return v3
}
"#;
    assert!(thalir_parser::parse(input).is_ok());
}

#[test]
fn test_thalir_crypto_operations() {
    let input = r#"
function %test_crypto(i256, i256) -> i256 {
block0(v0: i256, v1: i256):
    v2 = keccak256 v0, v1
    v3 = sha256 v0, v1
    v4 = iadd.i256 v2, v3
    return v4
}
"#;
    assert!(thalir_parser::parse(input).is_ok());
}

#[test]
fn test_thalir_checked_arithmetic() {
    let input = r#"
function %test_checked(i256, i256) -> i256 {
block0(v0: i256, v1: i256):
    v2 = checked_add v0, v1
    v3 = checked_sub v2, v1
    v4 = checked_mul v3, v0
    return v4
}
"#;
    assert!(thalir_parser::parse(input).is_ok());
}

#[test]
fn test_thalir_assertions_with_messages() {
    let input = r#"
function %test_require(i256) -> i256 {
block0(v0: i256):
    v1 = iconst.i256 0
    v2 = icmp ugt v0, v1
    require v2, "value must be positive"
    assert v2, "assertion failed"
    return v0
}
"#;
    assert!(thalir_parser::parse(input).is_ok());
}

#[test]
fn test_thalir_external_calls() {
    let input = r#"
function %test_calls(address, i256) -> i256 {
block0(v0: address, v1: i256):
    v2 = get_context msg.value
    v3 = call_ext v0, v1, v2
    return v3
}
"#;
    assert!(thalir_parser::parse(input).is_ok());
}

#[test]
fn test_thalir_complete_transfer() {
    let input = r#"
function %transfer(address, i256) -> i1 {
block0(v0: address, v1: i256):
    v2 = get_context msg.sender
    v3 = mapping_load map0, v2
    v4 = icmp uge v3, v1
    require v4, "insufficient balance"

    v5 = checked_sub v3, v1
    mapping_store map0, v2, v5

    v6 = mapping_load map0, v0
    v7 = checked_add v6, v1
    mapping_store map0, v0, v7

    v8 = iconst.i1 1
    return v8
}
"#;
    assert!(thalir_parser::parse(input).is_ok());
}
