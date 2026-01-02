/// Engine abstraction error types

use thiserror::Error;

pub type Result<T> = std::result::Result<T, EngineError>;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Class not found: {0}")]
    ClassNotFound(String),

    #[error("Method not found: {0}")]
    MethodNotFound(String),

    #[error("Field not found: {0}")]
    FieldNotFound(String),

    #[error("Instance not found")]
    InstanceNotFound,

    #[error("Invocation failed: {0}")]
    InvocationFailed(String),

    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: String, got: String },

    #[error("Memory error: {0}")]
    MemoryError(String),

    #[error("Engine not initialized")]
    NotInitialized,

    #[error("Engine initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Platform error: {0}")]
    PlatformError(#[from] anyhow::Error),
}
