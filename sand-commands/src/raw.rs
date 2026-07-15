//! Explicit raw Minecraft command escape hatch.

use std::fmt;

/// A command line intentionally excluded from typed grammar validation.
///
/// Raw commands still must be a single `.mcfunction`-safe line and must not
/// start with `/`. Prefer typed command builders whenever Sand models the
/// syntax you need.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[must_use = "raw commands do nothing until collected into a function"]
pub struct RawCommand(String);

impl RawCommand {
    pub fn new(command: impl Into<String>) -> Self {
        Self(command.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// Consume this wrapper and return the unchecked command text.
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
