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

    /// A datapack component failed builder-invariant validation or serialization
    /// during export. Includes the resource location, component kind/directory,
    /// the field or validation path, and a diagnostic message.
    #[error("component `{location}` ({kind}): {message} [field: {field}]")]
    ComponentValidation {
        location: sand_components::ResourceLocation,
        kind: String,
        field: String,
        message: String,
    },

    /// A datapack component or generated event requires a feature not available in
    /// the target Minecraft version.
    #[error(
        "component `{location}` ({kind}) requires feature `{feature_name}` \
         which is not available in target Minecraft {requested_version}\
         {fallback_note} — select a supported target or remove the component"
    )]
    VersionGating {
        location: String,
        kind: String,
        requested_version: String,
        is_fallback: bool,
        feature_name: String,
        fallback_note: String,
    },

    /// Minecraft version string failed to parse (expected format: `major.minor` or `major.minor.patch`).
    #[error("Invalid Minecraft version '{0}': expected format major.minor or major.minor.patch")]
    InvalidVersion(String),

    /// A `sand-commands` free-function command helper rejected its input
    /// (see `sand_commands::builtins`'s `try_*` helpers and [`sand_commands::CommandError`]).
    ///
    /// `sand-commands` cannot depend on `sand-core`/`sand-components` (the
    /// dependency direction runs the other way), so it defines its own
    /// crate-local error type; this bridges it into `SandError` so code that
    /// mixes `sand_core::Result`-returning calls with `sand_commands::try_*`
    /// helpers can use `?` across both without a manual `.map_err(...)`.
    #[error(transparent)]
    Command(#[from] sand_commands::CommandError),
}

/// Convenience type alias for `Result<T, SandError>`.
pub type Result<T> = std::result::Result<T, SandError>;

impl From<sand_components::SandError> for SandError {
    fn from(e: sand_components::SandError) -> Self {
        match e {
            sand_components::SandError::InvalidNamespace(s) => SandError::InvalidNamespace(s),
            sand_components::SandError::InvalidPath(s) => SandError::InvalidPath(s),
            sand_components::SandError::Serialization(e) => SandError::Serialization(e),
            sand_components::SandError::Io(e) => SandError::Io(e),
            sand_components::SandError::ComponentValidation {
                location,
                kind,
                field,
                message,
            } => SandError::ComponentValidation {
                location,
                kind,
                field,
                message,
            },
            sand_components::SandError::VersionGating {
                location,
                kind,
                requested_version,
                is_fallback,
                feature_name,
                fallback_note,
            } => SandError::VersionGating {
                location,
                kind,
                requested_version,
                is_fallback,
                feature_name,
                fallback_note,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_error_composes_with_sand_result_via_question_mark() {
        // A function mixing sand_core::Result-returning calls with
        // sand_commands::builtins::try_* must be able to use `?` across both
        // without a manual .map_err(...) — see the #170 review follow-up.
        fn build() -> Result<String> {
            let s =
                sand_commands::builtins::try_tp(sand_commands::Selector::self_(), 0.0, 0.0, 0.0)?;
            Ok(s)
        }
        assert_eq!(build().unwrap(), "tp @s 0 0 0");

        fn build_invalid() -> Result<String> {
            let s = sand_commands::builtins::try_tp(
                sand_commands::Selector::self_(),
                f64::NAN,
                0.0,
                0.0,
            )?;
            Ok(s)
        }
        assert!(matches!(build_invalid(), Err(SandError::Command(_))));
    }
}
