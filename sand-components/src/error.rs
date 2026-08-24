use thiserror::Error;

use sand_version::ComponentFeature;

use crate::resource_location::ResourceLocation;

/// Errors that can occur in sand-components.
///
/// **API Contract:** Run `sand api show sand::component::SandError` for the canonical contract.
#[derive(Debug, Error)]
pub enum SandError {
    #[doc = "**API Contract:** Run `sand api show sand::component::SandError::InvalidNamespace` for the canonical contract."]
    /// Namespace failed validation (must match `[a-z0-9_.-]+` and be non-empty).
    #[error("Invalid namespace '{0}': must only contain [a-z0-9_.-] and be non-empty")]
    InvalidNamespace(
        #[doc = "The `InvalidNamespace` variant carries the value described by its variant semantics: Namespace failed validation (must match `[a-z0-9_.-]+` and be non-empty)."]
        #[doc = "**API Contract:** Run `sand api show sand::component::SandError::InvalidNamespace::0` for the canonical contract."]
        String,
    ),

    #[doc = "**API Contract:** Run `sand api show sand::component::SandError::InvalidPath` for the canonical contract."]
    /// Resource location path failed validation (must match `[a-z0-9_./-]+` and be non-empty).
    #[error(
        "Invalid resource location path '{0}': must only contain [a-z0-9_./-] and be non-empty"
    )]
    InvalidPath(
        #[doc = "The `InvalidPath` variant carries the value described by its variant semantics: Resource location path failed validation (must match `[a-z0-9_./-]+` and be non-empty)."]
        #[doc = "**API Contract:** Run `sand api show sand::component::SandError::InvalidPath::0` for the canonical contract."]
        String,
    ),

    #[doc = "**API Contract:** Run `sand api show sand::component::SandError::Serialization` for the canonical contract."]
    /// JSON serialization or deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(
        #[doc = "The `Serialization` variant carries the value described by its variant semantics: JSON serialization or deserialization error."]
        #[doc = "**API Contract:** Run `sand api show sand::component::SandError::Serialization::0` for the canonical contract."]
        #[from]
        serde_json::Error,
    ),

    #[doc = "**API Contract:** Run `sand api show sand::component::SandError::Io` for the canonical contract."]
    /// File I/O error.
    #[error("I/O error: {0}")]
    Io(
        #[doc = "The `Io` variant carries the value described by its variant semantics: File I/O error."]
        #[doc = "**API Contract:** Run `sand api show sand::component::SandError::Io::0` for the canonical contract."]
        #[from]
        std::io::Error,
    ),

    #[doc = "**API Contract:** Run `sand api show sand::component::SandError::ComponentValidation` for the canonical contract."]
    /// A datapack component failed builder-invariant validation.
    ///
    /// Includes the resource location, the component kind/directory, the field
    /// or validation path where the failure was detected, and a diagnostic
    /// message explaining the invariant that was violated.
    #[error("component `{location}` ({kind}): {message} [field: {field}]")]
    ComponentValidation {
        #[doc = "**API Contract:** Run `sand api show sand::component::SandError::ComponentValidation::location` for the canonical contract."]
        /// The resource location of the failed component.
        location: ResourceLocation,
        #[doc = "**API Contract:** Run `sand api show sand::component::SandError::ComponentValidation::kind` for the canonical contract."]
        /// The component kind or directory (e.g. `"recipe"`, `"advancement"`).
        kind: String,
        #[doc = "**API Contract:** Run `sand api show sand::component::SandError::ComponentValidation::field` for the canonical contract."]
        /// The field or validation path where the failure was detected.
        field: String,
        #[doc = "**API Contract:** Run `sand api show sand::component::SandError::ComponentValidation::message` for the canonical contract."]
        /// Human-readable explanation of the violated invariant.
        message: String,
    },

    #[doc = "**API Contract:** Run `sand api show sand::component::SandError::VersionGating` for the canonical contract."]
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
        #[doc = "**API Contract:** Run `sand api show sand::component::SandError::VersionGating::location` for the canonical contract."]
        /// The resource location of the rejected component or event.
        location: String,
        #[doc = "**API Contract:** Run `sand api show sand::component::SandError::VersionGating::kind` for the canonical contract."]
        /// The component kind or trigger identifier.
        kind: String,
        #[doc = "**API Contract:** Run `sand api show sand::component::SandError::VersionGating::requested_version` for the canonical contract."]
        /// The requested Minecraft version string.
        requested_version: String,
        #[doc = "**API Contract:** Run `sand api show sand::component::SandError::VersionGating::is_fallback` for the canonical contract."]
        /// Whether the profile is a conservative fallback (not an exact match).
        is_fallback: bool,
        #[doc = "**API Contract:** Run `sand api show sand::component::SandError::VersionGating::feature_name` for the canonical contract."]
        /// The required feature identifier (e.g. `"dialogs"`).
        feature_name: String,
        #[doc = "**API Contract:** Run `sand api show sand::component::SandError::VersionGating::fallback_note` for the canonical contract."]
        /// Extra fallback note appended to the diagnostic when `is_fallback` is true.
        fallback_note: String,
    },
}

/// Convenience type alias for `Result<T, SandError>`.
///
/// **API Contract:** Run `sand api show sand::component::Result` for the canonical contract.
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
