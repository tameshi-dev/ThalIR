/*! Lower high-level IR to Cranelift for compilation.
 *
 * ThalIR preserves Solidity semantics at a high level, but actual execution requires lower-level IR.
 * This bridges the gap by lowering to Cranelift, mapping storage operations and external calls to
 * runtime functions while keeping native arithmetic fast.
 */

pub mod context;
pub mod lowering;
pub mod module;

pub use context::CodegenContext;
pub use lowering::lower_instruction;
pub use module::ModuleBuilder;
