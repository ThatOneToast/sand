use thiserror::Error;

use crate::resource_location::ResourceLocation;

/// Errors that can occur in sand-components.
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

    /// A datapack component failed builder-invariant validation.
    ///
    /// Includes the resource location, the component kind/directory, the field
    /// or validation path where the failure was detected, and a diagnostic
    /// message explaining the invariant that was violated.
    #[error("component `{location}` ({kind}): {message} [field: {field}]")]
    ComponentValidation {
        /// The resource location of the failed component.
        location: ResourceLocation,
        /// The component kind or directory (e.g. `"recipe"`, `"advancement"`).
        kind: String,
        /// The field or validation path where the failure was detected.
        field: String,
        /// Human-readable explanation of the violated invariant.
        message: String,
    },
}

/// Convenience type alias for `Result<T, SandError>`.
pub type Result<T> = std::result::Result<T, SandError>;
