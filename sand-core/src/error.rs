use thiserror::Error;

#[derive(Debug, Error)]
pub enum SandError {
    #[error("Invalid namespace '{0}': must only contain [a-z0-9_.-] and be non-empty")]
    InvalidNamespace(String),

    #[error(
        "Invalid resource location path '{0}': must only contain [a-z0-9_./-] and be non-empty"
    )]
    InvalidPath(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid Minecraft version '{0}': expected format major.minor or major.minor.patch")]
    InvalidVersion(String),
}

pub type Result<T> = std::result::Result<T, SandError>;
