/*! Unified interface for smart contract IR transformation.
 *
 * Single import for everything you need: transforming Solidity to IR, parsing/emitting text format,
 * and accessing analysis tools. Batteries-included entry point for auditing workflows.
 */

pub use thalir_core as core;
pub use thalir_emit as emit;
pub use thalir_parser as parser;
pub use thalir_transform as transform;

pub use thalir_core::{
    block::{BasicBlock, BlockId, Terminator},
    contract::Contract,
    function::{Function, Mutability, Visibility},
    instructions::Instruction,
    types::Type,
    values::Value,
};

pub use thalir_emit::{AnnotatedIREmitter, ThalIREmitter};

pub use thalir_parser::parse;

pub use thalir_transform::transform_solidity_to_ir;
