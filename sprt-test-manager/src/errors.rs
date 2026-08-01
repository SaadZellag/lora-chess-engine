use thiserror::Error;

#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("Compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Engine not found: {0}")]
    EngineNotFound(String),

    #[error("Fastchess execution failed: {0}")]
    FastchessExecutionFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Git error: {0}")]
    GitError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
