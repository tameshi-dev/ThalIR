/*! Turn IR back into readable text format.
 *
 * IR is meant to be read by humans, not just machines. Whether you're debugging a transformation,
 * reviewing generated code, or sharing findings with other auditors, these emitters produce clean
 * text that preserves structure and makes patterns visible.
 */

pub mod annotated_ir_emitter;
pub mod config;
pub mod emitter;
pub mod ir_formatter_base;
pub mod output;
pub mod thalir_emitter;

pub use annotated_ir_emitter::AnnotatedIREmitter;
pub use config::{EmitterConfig, VerbosityLevel};
pub use emitter::{EmitContext, EmitHelper, EmitResult, Emittable, Emitter};
pub use ir_formatter_base::{IRFormatterBase, SSAContext};
pub use output::{OutputFormat, OutputStyle};
pub use thalir_emitter::ThalIREmitter;
