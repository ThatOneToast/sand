//! Explicit raw Minecraft command escape hatch.

use std::fmt;

use crate::error::{CommandError, CommandResult};
use crate::render::{CommandProfile, RenderCommand, Validate};

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::RawCommand",
    aliases = ["sand::cmd::RawCommand", "sand::prelude::RawCommand", "sand::prelude::cmd::RawCommand"],
    module = "sand::command",
    summary = "A command line intentionally excluded from typed grammar validation.",
    context = "A command line intentionally excluded from typed grammar validation. Raw commands still must be a single `.mcfunction`-safe line and must not start with `/`. Prefer typed command builders whenever Sand models the syntax you need. `RawCommand::new` stays infallible so it remains ergonomic to construct — [`RawCommand::try_build`] (via [`RenderCommand`]) is the validated boundary, checking (see [#175](https://github.com/ThatOneToast/sand/issues/175)): - not empty or whitespace-only; - no NUL, newline, or carriage return (exactly one logical line); - no other ASCII control characters; - no leading `/` (`.mcfunction` lines never start with a slash).",
    minecraft = "Raw commands still must be a single `.mcfunction`-safe line and must not start with `/`. Prefer typed command builders whenever Sand models the syntax you need.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::RawCommand;",
)]
/// A command line intentionally excluded from typed grammar validation.
///
/// Raw commands still must be a single `.mcfunction`-safe line and must not
/// start with `/`. Prefer typed command builders whenever Sand models the
/// syntax you need.
///
/// `RawCommand::new` stays infallible so it remains ergonomic to construct —
/// [`RawCommand::try_build`] (via [`RenderCommand`]) is the validated
/// boundary, checking (see [#175](https://github.com/ThatOneToast/sand/issues/175)):
///
/// - not empty or whitespace-only;
/// - no NUL, newline, or carriage return (exactly one logical line);
/// - no other ASCII control characters;
/// - no leading `/` (`.mcfunction` lines never start with a slash).
///
/// Unknown/modded command *names* are never rejected — Sand has no way to
/// know what a mod registers, and rejecting them would break legitimate use
/// of this escape hatch.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[must_use = "raw commands do nothing until collected into a function"]
pub struct RawCommand(String);

impl RawCommand {
    /// Wraps an intentionally unchecked Minecraft command string.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::RawCommand::new",
        aliases = ["sand::cmd::RawCommand::new", "sand::prelude::RawCommand::new", "sand::prelude::cmd::RawCommand::new"],
        module = "sand::command",
        kind = "method",
        summary = "Wraps an intentionally unchecked Minecraft command string.",
        context = "Wraps an intentionally unchecked Minecraft command string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(command = "`command` provides the command wrapped when creating an intentionally unchecked Minecraft command string."),
        returns = "A `RawCommand` wrapping an intentionally unchecked Minecraft command string.",
        example = "use sand::prelude::*;\n\nfn demonstrate(command: impl Into < String >)  {\n    let raw_command = sand::command::RawCommand::new(command);\n}",
    )]
    pub fn new(command: impl Into<String>) -> Self {
        Self(command.into())
    }
    /// Returns the exact Minecraft command token represented by this raw command value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::RawCommand::as_str",
        aliases = ["sand::cmd::RawCommand::as_str", "sand::prelude::RawCommand::as_str", "sand::prelude::cmd::RawCommand::as_str"],
        module = "sand::command",
        kind = "method",
        summary = "Returns the exact Minecraft command token represented by this raw command value.",
        context = "Returns the exact Minecraft command token represented by this raw command value. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "Returns the exact Minecraft command token represented by this raw command value.",
        example = "use sand::prelude::*;\n\nfn demonstrate(raw_command_value: &sand::command::RawCommand)  {\n    let as_str = raw_command_value.as_str();\n}",
    )]
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// Consume this wrapper and return the unchecked command text.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::RawCommand::into_inner",
        aliases = ["sand::cmd::RawCommand::into_inner", "sand::prelude::RawCommand::into_inner", "sand::prelude::cmd::RawCommand::into_inner"],
        module = "sand::command",
        kind = "method",
        summary = "Consume this wrapper and return the unchecked command text.",
        context = "Consume this wrapper and return the unchecked command text. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The rendered Minecraft command text produced to consume this wrapper and return the unchecked command text.",
        example = "use sand::prelude::*;\n\nfn demonstrate(raw_command_value: sand::command::RawCommand)  {\n    let command = raw_command_value.into_inner();\n}",
    )]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<RawCommand> for String {
    fn from(command: RawCommand) -> Self {
        command.into_inner()
    }
}

impl fmt::Display for RawCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl PartialEq<&str> for RawCommand {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<RawCommand> for &str {
    fn eq(&self, other: &RawCommand) -> bool {
        *self == other.0
    }
}

impl Validate for RawCommand {
    fn validate(&self, _profile: &CommandProfile) -> CommandResult<()> {
        if self.0.trim().is_empty() {
            return Err(CommandError::new(
                "RawCommand",
                "text",
                "must not be empty or whitespace-only",
            )
            .with_code("SAND-RAW-COMMAND-EMPTY"));
        }
        crate::render::validate_line_integrity(&self.0)
            .map_err(|e| e.with_code("SAND-RAW-COMMAND-LINE"))?;
        if self.0.chars().any(|c| c.is_control()) {
            return Err(CommandError::new(
                "RawCommand",
                "text",
                format!(
                    "must not contain control characters (other than the disallowed NUL/CR/LF already checked), got {:?}",
                    self.0
                ),
            )
            .with_code("SAND-RAW-COMMAND-CONTROL"));
        }
        Ok(())
    }
}

impl RenderCommand for RawCommand {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_vanilla_raw_command_builds() {
        assert_eq!(
            RawCommand::new("say hello").try_build().unwrap(),
            "say hello"
        );
    }

    #[test]
    fn valid_modded_raw_command_builds() {
        assert_eq!(
            RawCommand::new("mymod:pulse 5").try_build().unwrap(),
            "mymod:pulse 5"
        );
    }

    #[test]
    fn rejects_empty_or_whitespace_only() {
        let err = RawCommand::new("").try_build().unwrap_err();
        assert_eq!(err.code, "SAND-RAW-COMMAND-EMPTY");
        assert!(RawCommand::new("   ").try_build().is_err());
    }

    #[test]
    fn rejects_multiple_lines() {
        let err = RawCommand::new("say a\nsay b").try_build().unwrap_err();
        assert_eq!(err.code, "SAND-RAW-COMMAND-LINE");
    }

    #[test]
    fn rejects_leading_slash() {
        assert!(RawCommand::new("/say hi").try_build().is_err());
    }

    #[test]
    fn rejects_nul_and_carriage_return() {
        assert!(RawCommand::new("say hi\0").try_build().is_err());
        assert!(RawCommand::new("say hi\r").try_build().is_err());
    }

    #[test]
    fn rejects_other_control_characters() {
        assert!(RawCommand::new("say hi\u{0007}").try_build().is_err());
    }
}
