use thalir_parser::{annotations::*, parse, Rule, ThalirParser};

#[test]
fn test_parser_roundtrip_simple() {
    let input = r#"
function %test(i32) -> i32 {
block0(v0: i32):
    v1 = iadd v0, v0
    return v1
}
"#;

    let result = parse(input);
    assert!(
        result.is_ok(),
        "Failed to parse standard ThalIR: {:?}",
        result.err()
    );
}

#[test]
fn test_parser_roundtrip_with_position_markers() {
    let input = r#"
function %test(i32) -> i32 {
block0(v0: i32):
    [0] v1 = iadd v0, v0
    [1] v2 = iadd v1, v1
    [2] return v2
}
"#;

    let result = parse(input);
    assert!(
        result.is_ok(),
        "Failed to parse ThalIR with position markers: {:?}",
        result.err()
    );
}

#[test]
fn test_parser_roundtrip_with_visual_cues() {
    let input = r#"
function %test(i32) -> i32 {
block0(v0: i32):
    [0] v1 = iadd v0, v0
    [1] 🔴 v2 = call fn0(v1)
    [2] 🟡 storage_store slot0, v2
    [3] return v2
}
"#;

    let result = parse(input);
    assert!(
        result.is_ok(),
        "Failed to parse ThalIR with visual cues: {:?}",
        result.err()
    );
}

#[test]
fn test_parser_roundtrip_with_analysis_comments() {
    let input = r#"
; ### Function: test (Public)
;  ORDERING ANALYSIS:
; - External call at position [1]
; - State modification at position [2]
; - [1] < [2] → REENTRANCY RISK

function %test(i32) -> i32 {
block0(v0: i32):
    [0] v1 = iadd v0, v0
    [1] 🔴 v2 = call fn0(v1)
    [2] 🟡 storage_store slot0, v2
    [3] return v2
}
"#;

    let result = parse(input);
    assert!(
        result.is_ok(),
        "Failed to parse ThalIR with analysis comments: {:?}",
        result.err()
    );
}

#[test]
fn test_parser_roundtrip_mixed_annotations() {
    let input = r#"
function %test(i32) -> i32 {
block0(v0: i32):
    [0] v1 = iadd v0, v0
    v2 = iadd v1, v1
    [2] 🔴 v3 = call fn0(v2)
    v4 = iadd v3, v3
    [4] return v4
}
"#;

    let result = parse(input);
    assert!(
        result.is_ok(),
        "Failed to parse mixed annotated/standard ThalIR: {:?}",
        result.err()
    );
}

#[test]
fn test_parser_roundtrip_ascii_visual_cues() {
    let input = r#"
function %test(i32) -> i32 {
block0(v0: i32):
    [0] v1 = iadd v0, v0
    [1] [EXTERNAL_CALL] v2 = call fn0(v1)
    [2] [STATE_WRITE] storage_store slot0, v2
    [3] return v2
}
"#;

    let result = parse(input);
    assert!(
        result.is_ok(),
        "Failed to parse ThalIR with ASCII visual cues: {:?}",
        result.err()
    );
}

#[test]
fn test_parser_roundtrip_extract_annotations() {
    use pest::Parser;

    let input = "[5] 🔴 v3 = call fn0(v0)";

    let pairs = ThalirParser::parse(Rule::instruction, input);
    assert!(
        pairs.is_ok(),
        "Failed to parse instruction: {:?}",
        pairs.err()
    );

    let pair = pairs.unwrap().into_iter().next().unwrap();
    let annotations = extract_instruction_annotations(&pair);

    assert_eq!(annotations.position, Some(5), "Position should be 5");
    assert_eq!(
        annotations.visual_cue,
        Some(VisualCue::ExternalCall),
        "Visual cue should be ExternalCall"
    );
}

#[test]
fn test_parser_roundtrip_contract() {
    let input = r#"
contract MyContract {
    function %transfer(address, i256) -> i256 public {
    block0(v0: address, v1: i256):
        [0] v2 = mapping_load balances[msg.sender]
        [1] 🟡 mapping_store balances[v0] <- v1
        [2] return v1
    }
}
"#;

    let result = parse(input);
    assert!(
        result.is_ok(),
        "Failed to parse annotated contract: {:?}",
        result.err()
    );
}

#[test]
fn test_parser_roundtrip_complex_function() {
    let input = r#"
; ### Function: withdraw (Public)
; - External Calls: 1
; - State Modifications: 1
;  ORDERING ANALYSIS:
; - External call at position [3]
; - State modification at position [5]
; - [3] < [5] → REENTRANCY RISK

function %withdraw(i256) -> i256 public {
block0(v0: i256):
    [0] v1 = mapping_load balances[msg.sender]
    [1] v2 = icmp slt v1, v0
    [2] v3 = require v2, "Insufficient balance"
    [3] 🔴 v4 = call_ext fn0(v0)
    [4] v5 = require v4, "Transfer failed"
    [5] 🟡 mapping_store balances[msg.sender] <- v0
    [6] return v0
}
"#;

    let result = parse(input);
    assert!(
        result.is_ok(),
        "Failed to parse complex annotated function: {:?}",
        result.err()
    );
}

#[test]
fn test_parser_roundtrip_example_file() {
    let input = std::fs::read_to_string("examples/annotated_withdraw.thalir");

    if let Ok(content) = input {
        let result = parse(&content);
        assert!(
            result.is_ok(),
            "Failed to parse examples/annotated_withdraw.thalir: {:?}",
            result.err()
        );
    } else {
        println!("Skipping example file test - file not found");
    }
}

#[test]
fn test_parser_roundtrip_all_visual_cues() {
    let input = r#"
function %test(i32) -> i32 {
block0(v0: i32):
    [0] 🔴 v1 = call fn0(v0)
    [1] 🟡 storage_store slot0, v1
    [2] ⚠️ v2 = iadd v0, v1
    [3] ✓ v3 = checked_add v0, v2
    [4] 🟢 v4 = iadd v3, v3
    [5] ❌ v5 = iadd v4, v4
    [6] return v5
}
"#;

    let result = parse(input);
    assert!(
        result.is_ok(),
        "Failed to parse all visual cues: {:?}",
        result.err()
    );
}

#[test]
fn test_parser_roundtrip_entities_with_annotations() {
    let input = r#"
function %f(i64, i32) -> i32 {
    gv0 = vmctx
    gv1 = load.i64 notrap readonly aligned gv0+8
    fn0 = %g(i64)

block0(v0: i64, v1: i32):
    [0] v2 = global_value.i64 gv1
    [1] v3 = load.i32 v2+8
    [2] return v3
}
"#;

    let result = parse(input);
    assert!(
        result.is_ok(),
        "Failed to parse entities with annotations: {:?}",
        result.err()
    );
}

#[test]
fn test_parser_roundtrip_backwards_compatible() {
    let inputs = vec![
        r#"
function %test(i32) -> i32 {
block0(v0: i32):
    v1 = iadd v0, v0
    return v1
}
"#,
        r#"
function %f(i64) -> i64 {
    gv0 = vmctx
    fn0 = %g(i64)

block0(v0: i64):
    v1 = global_value.i64 gv0
    return v1
}
"#,
        r#"
function %f(i64) -> i32 {
block0(v0: i64):
    v1 = load.i32 v0+8
    store.i32 v1, v0+16
    return v1
}
"#,
        r#"
contract Test {
    function %test(i32) -> i32 public {
    block0(v0: i32):
        return v0
    }
}
"#,
    ];

    for (i, input) in inputs.iter().enumerate() {
        let result = parse(input);
        assert!(
            result.is_ok(),
            "Backwards compatibility test {} failed: {:?}",
            i,
            result.err()
        );
    }
}
