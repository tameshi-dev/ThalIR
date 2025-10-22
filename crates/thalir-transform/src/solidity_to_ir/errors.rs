use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransformError {
    #[error("Parse error at line {line}, column {column}: {message}")]
    ParseError {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Invalid node kind: expected {expected}, got {actual}")]
    InvalidNodeKind { expected: String, actual: String },

    #[error("Missing required field: {field} in {node_type}")]
    MissingField { field: String, node_type: String },

    #[error("IR builder error: {0}")]
    BuilderError(String),

    #[error("Control flow error: {0}")]
    ControlFlowError(String),

    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("Multiple errors occurred: {0:?}")]
    Multiple(Vec<TransformError>),
}

impl From<thalir_core::IrError> for TransformError {
    fn from(err: thalir_core::IrError) -> Self {
        TransformError::BuilderError(err.to_string())
    }
}
