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
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::CommandError",
    aliases = ["sand::cmd::CommandError", "sand::prelude::cmd::CommandError"],
    module = "sand::command",
    summary = "A validation failure in a `sand-commands` `try_*` command helper.",
    context = "A validation failure in a `sand-commands` `try_*` command helper. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::CommandError;",
    fields(code = "Stable diagnostic category suitable for tests and tooling.", context = "Optional owner context added by composed commands or export.", field = "The parameter name that failed validation (e.g. `\"x\"`, `\"tag\"`).", helper = "The helper function that rejected its input (e.g. `\"tp\"`, `\"tag_add\"`).", message = "Human-readable explanation of the violated invariant."),
)]
#[derive(Debug, Clone, PartialEq, Error)]
#[error("error[{code}] {helper}: invalid `{field}` — {message}{context}")]
pub struct CommandError {
    /// Stable diagnostic category suitable for tests and tooling.
    pub code: String,
    /// The helper function that rejected its input (e.g. `"tp"`, `"tag_add"`).
    pub helper: &'static str,
    /// The parameter name that failed validation (e.g. `"x"`, `"tag"`).
    pub field: String,
    /// Human-readable explanation of the violated invariant.
    pub message: String,
    /// Optional owner context added by composed commands or export.
    pub context: String,
}

impl CommandError {
    /// Constructs a validation error for one command helper input.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::CommandError::new",
        aliases = ["sand::cmd::CommandError::new", "sand::prelude::cmd::CommandError::new"],
        module = "sand::command",
        kind = "method",
        summary = "Constructs a validation error for one command helper input.",
        context = "Constructs a validation error for one command helper input. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(helper = "`helper` is used when constructing a validation error for one command helper input.", field = "`field` is used when constructing a validation error for one command helper input.", message = "`message` is used when constructing a validation error for one command helper input."),
        returns = "A `CommandError` representing a validation error for one command helper input.",
        example = "use sand::prelude::*;\n\nfn demonstrate(helper: & 'static str, field: impl Into < String >, message: impl Into < String >)  {\n    let command_error = sand::command::CommandError::new(helper, field, message);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::CommandError::with_code",
        aliases = ["sand::cmd::CommandError::with_code", "sand::prelude::cmd::CommandError::with_code"],
        module = "sand::command",
        kind = "method",
        summary = "Override the stable diagnostic category.",
        context = "Override the stable diagnostic category. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(code = "`code` is used to override the stable diagnostic category."),
        returns = "The `CommandError` value with the documented change applied to override the stable diagnostic category.",
        example = "use sand::prelude::*;\n\nfn demonstrate(command_error_value: sand::command::CommandError, code: impl Into < String >)  {\n    let updated_command_error = command_error_value.with_code(code);\n}",
    )]
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = code.into();
        self
    }

    /// Add command/function context without discarding the original field error.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::CommandError::with_context",
        aliases = ["sand::cmd::CommandError::with_context", "sand::prelude::cmd::CommandError::with_context"],
        module = "sand::command",
        kind = "method",
        summary = "Add command/function context without discarding the original field error.",
        context = "Add command/function context without discarding the original field error. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(context = "`context` provides the context added when building command/function context without discarding the original field error."),
        returns = "The `CommandError` value with the documented change applied to add command/function context without discarding the original field error.",
        example = "use sand::prelude::*;\n\nfn demonstrate(command_error_value: sand::command::CommandError, context: impl AsRef < str >)  {\n    let updated_command_error = command_error_value.with_context(context);\n}",
    )]
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
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::CommandResult",
    aliases = ["sand::cmd::CommandResult", "sand::prelude::cmd::CommandResult"],
    module = "sand::command",
    summary = "Convenience alias for `Result<T, CommandError>`.",
    context = "Convenience alias for `Result<T, CommandError>`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::CommandResult;",
)]
pub type CommandResult<T> = std::result::Result<T, CommandError>;
