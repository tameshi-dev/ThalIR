use std::fs;
use std::path::PathBuf;
use thalir_parser;

#[test]
fn test_cranelift_basic_gvn() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/basic-gvn.clif");

    let content = fs::read_to_string(&path)
        .expect("Failed to read basic-gvn.clif fixture");

    match thalir_parser::parse(&content) {
        Ok(_) => {}
        Err(e) => panic!("Failed to parse {}: {}", path.display(), e),
    }
}

#[test]
fn test_cranelift_simple_alias() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/simple-alias.clif");

    let content = fs::read_to_string(&path)
        .expect("Failed to read simple-alias.clif fixture");

    match thalir_parser::parse(&content) {
        Ok(_) => {}
        Err(e) => panic!("Failed to parse {}: {}", path.display(), e),
    }
}
