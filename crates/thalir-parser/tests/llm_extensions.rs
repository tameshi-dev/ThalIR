use thalir_parser::{annotations::*, parse, Rule, ThalirParser};

fn check(input: &str) -> bool {
    match parse(input) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            false
        }
    }
}

fn parse_instruction(input: &str) -> Result<InstructionAnnotations, String> {
    use pest::Parser;

    let pairs =
        ThalirParser::parse(Rule::instruction, input).map_err(|e| format!("Parse error: {}", e))?;

    let pair = pairs
        .into_iter()
        .next()
        .ok_or_else(|| "No instruction found".to_string())?;

    Ok(extract_instruction_annotations(&pair))
}

#[test]
fn test_position_marker_basic() {
    let input = r#"
function %test(i32) -> i32 {
block0(v0: i32):
    [0] v1 = iadd v0, v0
    return v1
}
"#;
    assert!(check(input), "Failed to parse position marker");
}

#[test]
fn test_position_marker_with_block() {
    let input = r#"
function %test(i32) -> i32 {
block0(v0: i32):
    [0] v1 = iadd v0, v0
    [1] v2 = iadd v1, v1
    [2] return v2
}
"#;
    assert!(
        check(input),
        "Failed to parse function with position markers"
    );
}

#[test]
fn test_visual_cue_emoji_external_call() {
    let input = r#"
function %test(i32) -> i32 {
block0(v0: i32):
    [5]  v3 = call fn0(v0)
    return v3
}
"#;
    assert!(check(input), "Failed to parse emoji visual cue");
}

#[test]
fn test_visual_cue_emoji_state_write() {
    let input = r#"
function %test(i32) {
block0(v0: i32):
    [8] 🟡 storage_store slot0, v0
    return
}
"#;
    assert!(check(input), "Failed to parse state write emoji");
}

#[test]
fn test_visual_cue_ascii() {
    let input = r#"
function %test(i32) -> i32 {
block0(v0: i32):
    [5] [EXTERNAL_CALL] v3 = call fn0(v0)
    return v3
}
"#;
    assert!(check(input), "Failed to parse ASCII visual cue");
}

#[test]
fn test_visual_cue_warning() {
    let input = r#"
function %test(i32, i32) -> i32 {
block0(v1: i32, v2: i32):
    [10]  v4 = iadd v1, v2
    return v4
}
"#;
    assert!(check(input), "Failed to parse warning emoji");
}

#[test]
fn test_position_and_visual_cue() {
    let input = r#"
function %test(i32) -> i32 {
block0(v0: i32):
    [5]  v3 = call_ext fn0(v0)
    return v3
}
"#;
    assert!(check(input), "Failed to parse position + visual cue");
}

#[test]
fn test_full_annotated_function() {
    let input = r#"
function %withdraw(i256) -> i256 public {
block0(v0: i256):
    [0] v1 = mapping_load balances[msg.sender]
    [1] v2 = icmp slt v1, v0
    [2] v3 = require v2, "Insufficient balance"
    [3]  v4 = call_ext fn0(v0)
    [4] v5 = require v4, "Transfer failed"
    [5] v6 = isub v1, v0
    [6] 🟡 mapping_store balances[msg.sender] <- v6
    [7] return v6
}
"#;
    assert!(check(input), "Failed to parse fully annotated function");
}

#[test]
fn test_analysis_comment_ordering_header() {
    let input = r#"
function %test(i32) -> i32 {
;  ORDERING ANALYSIS:
block0(v0: i32):
    return v0
}
"#;
    assert!(check(input), "Failed to parse ordering analysis header");
}

#[test]
fn test_analysis_comment_external_call() {
    let input = r#"
function %test(i32) -> i32 {
; - External call at position [4]
block0(v0: i32):
    return v0
}
"#;
    assert!(check(input), "Failed to parse external call comment");
}

#[test]
fn test_analysis_comment_ordering_comparison() {
    let input = r#"
function %test(i32) -> i32 {
; - [4] < [8] → REENTRANCY RISK
block0(v0: i32):
    return v0
}
"#;
    assert!(check(input), "Failed to parse ordering comparison comment");
}

#[test]
fn test_analysis_comment_function_header() {
    let input = r#"
; ### Function: withdraw (Public)
function %withdraw(i256) -> i256 public {
block0(v0: i256):
    return v0
}
"#;
    assert!(check(input), "Failed to parse function header comment");
}

#[test]
fn test_complete_analysis_block() {
    let input = r#"
; ### Function: withdraw (Public)
; - External Calls: 2
; - State Modifications: 1
;  ORDERING ANALYSIS:
; - External call at position [4]
; - External call at position [5]
; - State modification at position [8]
; - [4] < [8] → REENTRANCY RISK
; - [5] < [8] → REENTRANCY RISK

function %withdraw(i256) -> i256 public {
block0(v0: i256):
    [0] v1 = mapping_load balances[msg.sender]
    [1] v2 = icmp slt v1, v0
    [2] v3 = require v2, "Insufficient balance"
    [3]  v4 = call_ext fn0(v0)
    [4]  v5 = call_ext fn1(v0)
    [5] v6 = require v4, "Transfer failed"
    [6] v7 = isub v1, v0
    [7] 🟡 mapping_store balances[msg.sender] <- v7
    [8] return v7
}
"#;
    assert!(check(input), "Failed to parse complete analysis block");
}

#[test]
fn test_mixed_annotated_and_standard() {
    let input = r#"
function %test(i32) -> i32 {
block0(v0: i32):
    [0] v1 = iadd v0, v0
    v2 = iadd v1, v1
    [2]  call fn0(v2)
    return v2
}
"#;
    assert!(
        check(input),
        "Failed to parse mixed annotated/standard instructions"
    );
}

#[test]
fn test_extract_position_annotation() {
    let result = parse_instruction("[42] v1 = iadd v0, v0");
    match result {
        Ok(annot) => {
            assert_eq!(annot.position, Some(42), "Position should be 42");
        }
        Err(e) => panic!("Parse failed: {}", e),
    }
}

#[test]
fn test_extract_visual_cue_annotation() {
    let result = parse_instruction("[5] 🔴 v3 = call fn0(v0)");
    match result {
        Ok(annot) => {
            assert_eq!(
                annot.visual_cue,
                Some(VisualCue::ExternalCall),
                "Visual cue should be ExternalCall"
            );
        }
        Err(e) => panic!("Parse failed: {}", e),
    }
}

#[test]
fn test_extract_both_annotations() {
    let result = parse_instruction("[8] 🟡 storage_store slot0, v1");
    match result {
        Ok(annot) => {
            assert_eq!(annot.position, Some(8), "Position should be 8");
            assert_eq!(
                annot.visual_cue,
                Some(VisualCue::StateWrite),
                "Visual cue should be StateWrite"
            );
        }
        Err(e) => panic!("Parse failed: {}", e),
    }
}

#[test]
fn test_backwards_compatibility_no_annotations() {
    let input = r#"
function %test(i32, i32) -> i32 {
block0(v0: i32, v1: i32):
    v2 = iadd v0, v1
    return v2
}
"#;
    assert!(check(input), "Standard ThalIR should still parse");
}

#[test]
fn test_backwards_compatibility_complex_function() {
    let input = r#"
function %f(i64, i32) -> i32 {
    gv0 = vmctx
    gv1 = load.i64 notrap readonly aligned gv0+8
    fn0 = %g(i64)

block0(v0: i64, v1: i32):
    v2 = global_value.i64 gv1
    v3 = load.i32 v2+8
    return v3
}
"#;
    assert!(check(input), "Complex standard ThalIR should still parse");
}

#[test]
fn test_contract_with_annotations() {
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
    assert!(check(input), "Contract with annotations should parse");
}

#[test]
fn test_all_visual_cues() {
    let inputs = vec![
        ("[0] 🔴 v1 = call fn0(v0)", VisualCue::ExternalCall),
        ("[1] 🟡 storage_store slot0, v1", VisualCue::StateWrite),
        ("[2] ⚠️ v2 = iadd v0, v1", VisualCue::Warning),
        ("[3] ✓ v3 = checked_add v0, v1", VisualCue::Checked),
        ("[4] 🟢 v4 = iadd v0, v1", VisualCue::Safe),
        ("[5] ❌ v5 = iadd v0, v1", VisualCue::Unsafe),
    ];

    for (input, expected_cue) in inputs {
        let result = parse_instruction(input);
        match result {
            Ok(annot) => {
                assert_eq!(
                    annot.visual_cue,
                    Some(expected_cue),
                    "Failed for input: {}",
                    input
                );
            }
            Err(e) => panic!("Parse failed for {}: {}", input, e),
        }
    }
}

#[test]
fn test_position_markers_sequential() {
    let input = r#"
function %test(i32) -> i32 {
block0(v0: i32):
    [0] v1 = iadd v0, v0
    [1] v2 = iadd v1, v1
    [2] v3 = iadd v2, v2
    [3] v4 = iadd v3, v3
    [4] return v4
}
"#;
    assert!(check(input), "Sequential position markers should parse");
}

#[test]
fn test_non_sequential_position_markers() {
    let input = r#"
function %test(i32) -> i32 {
block0(v0: i32):
    [0] v1 = iadd v0, v0
    [5] v2 = iadd v1, v1
    [10] return v2
}
"#;
    assert!(
        check(input),
        "Non-sequential position markers should parse (sparse numbering)"
    );
}
