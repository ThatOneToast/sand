//! Validation error type for `sand-commands`' fallible `try_*` command helpers.
//!
//! `sand-commands` has no path dependency on `sand-components`/`sand-core` (the
//! dependency direction runs the other way), so this crate defines its own
//! small `thiserror`-based error type rather than reusing
//! `sand_components::SandError` — see
//! [#170](https://github.com/ThatOneToast/sand/issues/170).
//!
//! The infallible free functions in [`crate::builtins`] remain available as
//! documented raw/unchecked escape hatches. Their `try_*` counterparts return
//! [`CommandError`] instead of emitting command text that Minecraft would
//! reject at runtime.

use thiserror::Error;

/// A validation failure in a `sand-commands` `try_*` command helper.
#[derive(Debug, Clone, PartialEq, Error)]
#[error("{helper}: invalid `{field}` — {message}")]
pub struct CommandError {
    /// The helper function that rejected its input (e.g. `"tp"`, `"tag_add"`).
    pub helper: &'static str,
    /// The parameter name that failed validation (e.g. `"x"`, `"tag"`).
    pub field: &'static str,
    /// Human-readable explanation of the violated invariant.
    pub message: String,
}

impl CommandError {
    pub fn new(helper: &'static str, field: &'static str, message: impl Into<String>) -> Self {
        Self {
            helper,
            field,
            message: message.into(),
        }
    }
}

/// Convenience alias for `Result<T, CommandError>`.
pub type CommandResult<T> = std::result::Result<T, CommandError>;
