use thiserror::Error;

/// Errors that can occur when using sand-core.
#[derive(Debug, Error)]
pub enum SandError {
    /// Namespace failed validation (must match `[a-z0-9_.-]+` and be non-empty).
    #[error("Invalid namespace '{0}': must only contain [a-z0-9_.-] and be non-empty")]
    InvalidNamespace(String),

    /// Resource location path failed validation (must match `[a-z0-9_./-]+` and be non-empty).
    #[error(
        "Invalid resource location path '{0}': must only contain [a-z0-9_./-] and be non-empty"
    )]
    InvalidPath(String),

    /// JSON serialization or deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// File I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Minecraft version string failed to parse (expected format: `major.minor` or `major.minor.patch`).
    #[error("Invalid Minecraft version '{0}': expected format major.minor or major.minor.patch")]
    InvalidVersion(String),
}

/// Convenience type alias for `Result<T, SandError>`.
pub type Result<T> = std::result::Result<T, SandError>;
