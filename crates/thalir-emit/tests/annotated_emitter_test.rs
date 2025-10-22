use num_bigint::BigUint;
use thalir_core::{
    block::Terminator,
    contract::Contract,
    function::{
        Function, FunctionBody, FunctionMetadata, FunctionSignature, Mutability, Visibility,
    },
    instructions::{Instruction, StorageKey},
    types::Type,
    values::{Constant, TempId, Value},
};
use thalir_emit::{annotated_ir_emitter::AnnotationConfig, AnnotatedIREmitter};

#[test]
fn test_annotated_emitter_basic() {
    let mut function_body = FunctionBody::new();

    let entry_block = function_body
        .get_block_mut(function_body.entry_block())
        .unwrap();

    entry_block.add_instruction(Instruction::Add {
        result: Value::Temp(TempId(1)),
        left: Value::Temp(TempId(0)),
        right: Value::Temp(TempId(0)),
        ty: Type::Uint(256),
    });

    entry_block.add_instruction(Instruction::StorageStore {
        key: StorageKey::Slot(BigUint::from(0u32)),
        value: Value::Temp(TempId(1)),
    });

    entry_block.set_terminator(Terminator::Return(None));

    let signature = FunctionSignature {
        name: "test".to_string(),
        params: vec![],
        returns: vec![],
        is_payable: false,
    };

    let function = Function {
        signature,
        visibility: Visibility::Public,
        mutability: Mutability::NonPayable,
        modifiers: vec![],
        body: function_body,
        metadata: FunctionMetadata::default(),
    };

    let mut contract = Contract::new("TestContract".to_string());
    contract.add_function(function);

    let emitter = AnnotatedIREmitter::new(vec![contract]);
    let output = emitter.emit_to_string(false);

    println!("Annotated output:\n{}", output);

    assert!(output.contains("[0]"), "Should contain position marker [0]");
    assert!(output.contains("[1]"), "Should contain position marker [1]");
    assert!(
        output.contains("🟡") || output.contains("[STATE_WRITE]"),
        "Should contain state write marker"
    );
}

#[test]
fn test_annotation_config_ascii_mode() {
    let mut function_body = FunctionBody::new();

    let entry_block = function_body
        .get_block_mut(function_body.entry_block())
        .unwrap();
    entry_block.add_instruction(Instruction::StorageStore {
        key: StorageKey::Slot(BigUint::from(0u32)),
        value: Value::Constant(Constant::Uint(BigUint::from(42u32), 256)),
    });
    entry_block.set_terminator(Terminator::Return(None));

    let signature = FunctionSignature {
        name: "test".to_string(),
        params: vec![],
        returns: vec![],
        is_payable: false,
    };

    let function = Function {
        signature,
        visibility: Visibility::Public,
        mutability: Mutability::NonPayable,
        modifiers: vec![],
        body: function_body,
        metadata: FunctionMetadata::default(),
    };

    let mut contract = Contract::new("TestContract".to_string());
    contract.add_function(function);

    let config = AnnotationConfig {
        emit_position_markers: true,
        emit_visual_cues: true,
        use_ascii_cues: true,
        emit_ordering_analysis: false,
        emit_function_headers: false,
    };

    let emitter = AnnotatedIREmitter::new(vec![contract]).with_annotation_config(config);
    let output = emitter.emit_to_string(false);

    println!("ASCII annotated output:\n{}", output);

    assert!(
        output.contains("[STATE_WRITE]"),
        "Should contain ASCII state write marker"
    );
    assert!(
        !output.contains("🟡"),
        "Should not contain emoji markers in ASCII mode"
    );
}

#[test]
fn test_annotation_disabled() {
    let mut function_body = FunctionBody::new();

    let entry_block = function_body
        .get_block_mut(function_body.entry_block())
        .unwrap();
    entry_block.set_terminator(Terminator::Return(None));

    let signature = FunctionSignature {
        name: "test".to_string(),
        params: vec![],
        returns: vec![],
        is_payable: false,
    };

    let function = Function {
        signature,
        visibility: Visibility::Public,
        mutability: Mutability::NonPayable,
        modifiers: vec![],
        body: function_body,
        metadata: FunctionMetadata::default(),
    };

    let mut contract = Contract::new("TestContract".to_string());
    contract.add_function(function);

    let config = AnnotationConfig {
        emit_position_markers: false,
        emit_visual_cues: false,
        use_ascii_cues: false,
        emit_ordering_analysis: false,
        emit_function_headers: false,
    };

    let emitter = AnnotatedIREmitter::new(vec![contract]).with_annotation_config(config);
    let output = emitter.emit_to_string(false);

    println!("Plain output (annotations disabled):\n{}", output);

    assert!(
        !output.contains("[0]"),
        "Should not contain position markers"
    );
    assert!(!output.contains("🟡"), "Should not contain emoji markers");
    assert!(
        !output.contains("[STATE_WRITE]"),
        "Should not contain ASCII markers"
    );
}
