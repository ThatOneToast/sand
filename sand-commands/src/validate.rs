//! Shared validation primitives for `sand-commands`' fallible `try_*` helpers.
//!
//! Small, focused checks reused across [`crate::builtins`]'s `try_*` command
//! functions: finite numbers, non-empty/well-formed identifiers, and
//! resource-location shape. This is the single validation foundation for this
//! crate (see [#170](https://github.com/ThatOneToast/sand/issues/170)) — do
//! not add a second, competing validation helper set elsewhere in this crate.
//!
//! `sand-commands` cannot depend on `sand-components` (that would create a
//! dependency cycle — `sand-components` depends on `sand-commands`), so the
//! resource-location charset check here intentionally mirrors
//! `sand_components::resource_location`'s rule rather than importing it.

use crate::error::{CommandError, CommandResult};

/// Reject non-finite (`NaN`/`±inf`) values before they reach command text.
pub fn finite(value: f64, helper: &'static str, field: &'static str) -> CommandResult<f64> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(CommandError::new(
            helper,
            field,
            format!("must be a finite number, got `{value}`"),
        ))
    }
}

/// Reject an empty string after trimming.
pub fn non_empty<'a>(
    value: &'a str,
    helper: &'static str,
    field: &'static str,
) -> CommandResult<&'a str> {
    if value.trim().is_empty() {
        Err(CommandError::new(helper, field, "must not be empty"))
    } else {
        Ok(value)
    }
}

/// Reject a string containing ASCII whitespace or control characters —
/// Minecraft's raw command-line argument grammar splits on whitespace, so an
/// unescaped space or control character in a token like a tag, team, or
/// scoreboard-value string silently truncates or corrupts the command.
pub fn no_whitespace_or_control<'a>(
    value: &'a str,
    helper: &'static str,
    field: &'static str,
) -> CommandResult<&'a str> {
    non_empty(value, helper, field)?;
    if value.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(CommandError::new(
            helper,
            field,
            format!("must not contain whitespace or control characters, got `{value}`"),
        ));
    }
    Ok(value)
}

/// Validate `namespace:path` resource-location shape, matching
/// `sand_components::resource_location`'s charset rule:
/// namespace `[a-z0-9_.-]+`, path `[a-z0-9_./-]+`.
pub fn resource_location_shape<'a>(
    value: &'a str,
    helper: &'static str,
    field: &'static str,
) -> CommandResult<&'a str> {
    non_empty(value, helper, field)?;
    let Some((namespace, path)) = value.split_once(':') else {
        return Err(CommandError::new(
            helper,
            field,
            format!("must be a `namespace:path` resource location, got `{value}`"),
        ));
    };
    let valid_namespace = !namespace.is_empty()
        && namespace
            .chars()
            .all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '-'));
    let valid_path = !path.is_empty()
        && path
            .chars()
            .all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '/' | '-'));
    if valid_namespace && valid_path {
        Ok(value)
    } else {
        Err(CommandError::new(
            helper,
            field,
            format!(
                "must be a valid `namespace:path` resource location \
                 (namespace: [a-z0-9_.-]+, path: [a-z0-9_./-]+), got `{value}`"
            ),
        ))
    }
}

/// Reject a count/amount of `0` where Minecraft requires at least one.
pub fn positive_u32(value: u32, helper: &'static str, field: &'static str) -> CommandResult<u32> {
    if value == 0 {
        Err(CommandError::new(
            helper,
            field,
            "must be at least 1, got 0",
        ))
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_accepts_normal_values() {
        assert_eq!(finite(1.5, "h", "f").unwrap(), 1.5);
        assert_eq!(finite(0.0, "h", "f").unwrap(), 0.0);
        assert_eq!(finite(-100.0, "h", "f").unwrap(), -100.0);
    }

    #[test]
    fn finite_rejects_nan_and_infinite() {
        assert!(finite(f64::NAN, "h", "f").is_err());
        assert!(finite(f64::INFINITY, "h", "f").is_err());
        assert!(finite(f64::NEG_INFINITY, "h", "f").is_err());
    }

    #[test]
    fn non_empty_rejects_blank_strings() {
        assert!(non_empty("", "h", "f").is_err());
        assert!(non_empty("   ", "h", "f").is_err());
        assert!(non_empty("x", "h", "f").is_ok());
    }

    #[test]
    fn no_whitespace_or_control_rejects_spaces_and_tabs() {
        assert!(no_whitespace_or_control("has space", "h", "f").is_err());
        assert!(no_whitespace_or_control("has\ttab", "h", "f").is_err());
        assert!(no_whitespace_or_control("valid_tag", "h", "f").is_ok());
    }

    #[test]
    fn resource_location_shape_accepts_valid() {
        assert!(resource_location_shape("minecraft:diamond", "h", "f").is_ok());
        assert!(resource_location_shape("my_pack:sub/path", "h", "f").is_ok());
    }

    #[test]
    fn resource_location_shape_rejects_missing_colon() {
        assert!(resource_location_shape("diamond", "h", "f").is_err());
    }

    #[test]
    fn resource_location_shape_rejects_uppercase_and_spaces() {
        assert!(resource_location_shape("Minecraft:Diamond", "h", "f").is_err());
        assert!(resource_location_shape("minecraft: diamond", "h", "f").is_err());
    }

    #[test]
    fn positive_u32_rejects_zero() {
        assert!(positive_u32(0, "h", "f").is_err());
        assert!(positive_u32(1, "h", "f").is_ok());
    }
}
