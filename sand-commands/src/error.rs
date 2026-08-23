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
///
/// **API Contract:** Run `sand api show sand::command::CommandError` for the canonical contract.
#[derive(Debug, Clone, PartialEq, Error)]
#[error("error[{code}] {helper}: invalid `{field}` — {message}{context}")]
pub struct CommandError {
    /// Stable diagnostic category suitable for tests and tooling.
    #[doc = "**API Contract:** Run `sand api show sand::command::CommandError::code` for the canonical contract."]
    pub code: String,
    /// The helper function that rejected its input (e.g. `"tp"`, `"tag_add"`).
    #[doc = "**API Contract:** Run `sand api show sand::command::CommandError::helper` for the canonical contract."]
    pub helper: &'static str,
    /// The parameter name that failed validation (e.g. `"x"`, `"tag"`).
    #[doc = "**API Contract:** Run `sand api show sand::command::CommandError::field` for the canonical contract."]
    pub field: String,
    /// Human-readable explanation of the violated invariant.
    #[doc = "**API Contract:** Run `sand api show sand::command::CommandError::message` for the canonical contract."]
    pub message: String,
    /// Optional owner context added by composed commands or export.
    #[doc = "**API Contract:** Run `sand api show sand::command::CommandError::context` for the canonical contract."]
    pub context: String,
}

impl CommandError {
    /// Constructs a validation error for one command helper input.
    #[doc = "**API Contract:** Run `sand api show sand::command::CommandError::new` for the canonical contract."]
    pub fn new(helper: &'static str, field: impl Into<String>, message: impl Into<String>) -> Self {
        let field = field.into();
        Self {
            code: format!(
                "command.{}.invalid_{}",
                diagnostic_fragment(helper),
                diagnostic_fragment(&field)
            ),
            helper,
            field,
            message: message.into(),
            context: String::new(),
        }
    }

    /// Override the stable diagnostic category.
    #[doc = "**API Contract:** Run `sand api show sand::command::CommandError::with_code` for the canonical contract."]
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = code.into();
        self
    }

    /// Add command/function context without discarding the original field error.
    #[doc = "**API Contract:** Run `sand api show sand::command::CommandError::with_context` for the canonical contract."]
    pub fn with_context(mut self, context: impl AsRef<str>) -> Self {
        self.context.push_str(&format!(" [{}]", context.as_ref()));
        self
    }
}

fn diagnostic_fragment(value: &str) -> String {
    let mut result = String::new();
    let mut separator = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            if separator && !result.is_empty() {
                result.push('_');
            }
            result.push(character.to_ascii_lowercase());
            separator = false;
        } else {
            separator = true;
        }
    }
    result
}

/// Convenience alias for `Result<T, CommandError>`.
///
/// **API Contract:** Run `sand api show sand::command::CommandResult` for the canonical contract.
pub type CommandResult<T> = std::result::Result<T, CommandError>;
