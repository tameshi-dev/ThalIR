/*! Core IR types and builders for smart contract analysis.
 *
 * Auditing requires a structured representation where security-critical operations are explicit.
 * This crate provides the building blocks to construct and manipulate IR that preserves Solidity's
 * semantics while exposing the patterns auditors care about.
 */

pub mod analysis;
pub mod block;
pub mod builder;
pub mod codegen;
pub mod contract;
pub mod cursor;
pub mod extensions;
pub mod format;
pub mod function;
pub mod inst_builder;
pub mod instructions;
pub mod ir_persist;
pub mod metadata;
pub mod obfuscation;
pub mod source_location;
pub mod types;
pub mod values;

pub use block::{BasicBlock, BlockId, BlockParam, Terminator};
pub use builder::{ContractBuilder, FunctionBuilder};
pub use contract::{Contract, ContractMetadata, StorageLayout};
pub use function::{Function, FunctionBody, FunctionSignature, Mutability, Visibility};
pub use instructions::Instruction;
pub use metadata::{OptimizationHints, SecurityMetadata};
pub use obfuscation::{
    ObfuscationConfig, ObfuscationLevel, ObfuscationMapping, ObfuscationPass, VulnerabilityMapper,
};
pub use source_location::SourceFiles;
pub use types::{Type, TypeRegistry};
pub use values::{Constant, Location, SourceLocation, Value};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum IrError {
    #[error("Type error: {0}")]
    TypeError(String),
    #[error("Invalid instruction: {0}")]
    InvalidInstruction(String),
    #[error("Builder error: {0}")]
    BuilderError(String),
    #[error("Contract not found: {0}")]
    ContractNotFound(String),
    #[error("Transform error: {0}")]
    TransformError(String),
    #[error("Cranelift error: {0}")]
    CraneliftError(String),
}

pub type Result<T> = std::result::Result<T, IrError>;

#[cfg(test)]
mod tests;
