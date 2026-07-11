use thiserror::Error;

use sand_version::ComponentFeature;

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

    /// A component or generated event requires a feature not available in the
    /// target Minecraft version.
    ///
    /// Includes the resource location, the component kind or trigger
    /// identifier, the requested version string, whether the profile is
    /// fallback, and the required feature name.
    #[error(
        "component `{location}` ({kind}) requires feature `{feature_name}` \
         which is not available in target Minecraft {requested_version}\
         {fallback_note} — select a supported target or remove the component"
    )]
    VersionGating {
        /// The resource location of the rejected component or event.
        location: String,
        /// The component kind or trigger identifier.
        kind: String,
        /// The requested Minecraft version string.
        requested_version: String,
        /// Whether the profile is a conservative fallback (not an exact match).
        is_fallback: bool,
        /// The required feature identifier (e.g. `"dialogs"`).
        feature_name: String,
        /// Extra fallback note appended to the diagnostic when `is_fallback` is true.
        fallback_note: String,
    },
}

/// Convenience type alias for `Result<T, SandError>`.
pub type Result<T> = std::result::Result<T, SandError>;

/// Build a [`SandError::VersionGating`] error for a component that requires
/// a feature not available in the target version.
pub fn version_gating_error(
    location: &str,
    kind: &str,
    feature: ComponentFeature,
    requested_version: &str,
    is_fallback: bool,
) -> SandError {
    let fallback_note = if is_fallback {
        " (fallback profile: all features disabled; use an exact known version or \
         `mc_version = \"latest\"` to enable version-gated features)"
    } else {
        ""
    }
    .to_string();
    SandError::VersionGating {
        location: location.to_string(),
        kind: kind.to_string(),
        requested_version: requested_version.to_string(),
        is_fallback,
        feature_name: feature.name().to_string(),
        fallback_note,
    }
}
