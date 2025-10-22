/*! EVM-specific operations beyond standard IR.
 *
 * Cranelift doesn't know about storage, delegatecall, or keccak256. These extensions define the
 * EVM-specific operations that make ThalIR suitable for smart contract analysis, keeping them
 * separate from core IR so the abstraction boundary stays clear.
 */

pub mod evm;
pub mod crypto;
pub mod storage;

pub use evm::*;
pub use crypto::*;
pub use storage::*;
