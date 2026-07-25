use serde_json::Value;

use crate::error::{Result, SandError};
use crate::resource_location::ResourceLocation;

pub(crate) fn error(
    location: &ResourceLocation,
    kind: &str,
    field: &str,
    message: &str,
) -> SandError {
    SandError::ComponentValidation {
        location: location.clone(),
        kind: kind.to_string(),
        field: field.to_string(),
        message: message.to_string(),
    }
}

pub(crate) fn require_non_empty(
    location: &ResourceLocation,
    kind: &str,
    field: &str,
    value: &str,
) -> Result<()> {
    if value.is_empty() {
        return Err(error(
            location,
            kind,
            field,
            &format!("{field} must not be empty"),
        ));
    }
    Ok(())
}

pub(crate) fn reject_whitespace_only(
    location: &ResourceLocation,
    kind: &str,
    field: &str,
    value: &str,
) -> Result<()> {
    if !value.is_empty() && value.trim().is_empty() {
        return Err(error(
            location,
            kind,
            field,
            &format!("{field} must not be whitespace-only"),
        ));
    }
    Ok(())
}

pub(crate) fn reject_control_chars(
    location: &ResourceLocation,
    kind: &str,
    field: &str,
    value: &str,
) -> Result<()> {
    if value.chars().any(|c| c.is_control()) {
        return Err(error(
            location,
            kind,
            field,
            &format!("{field} must not contain control characters"),
        ));
    }
    Ok(())
}

pub(crate) fn require_finite_f32(
    location: &ResourceLocation,
    kind: &str,
    field: &str,
    value: f32,
) -> Result<()> {
    if !value.is_finite() {
        return Err(error(
            location,
            kind,
            field,
            &format!("{field} must be finite; received {value}"),
        ));
    }
    Ok(())
}

pub(crate) fn require_non_negative_f32(
    location: &ResourceLocation,
    kind: &str,
    field: &str,
    value: f32,
) -> Result<()> {
    require_finite_f32(location, kind, field, value)?;
    if value < 0.0 {
        return Err(error(
            location,
            kind,
            field,
            &format!("{field} must be non-negative; received {value}"),
        ));
    }
    Ok(())
}

pub(crate) fn require_positive_f32(
    location: &ResourceLocation,
    kind: &str,
    field: &str,
    value: f32,
) -> Result<()> {
    require_finite_f32(location, kind, field, value)?;
    if value <= 0.0 {
        return Err(error(
            location,
            kind,
            field,
            &format!("{field} must be positive; received {value}"),
        ));
    }
    Ok(())
}

pub(crate) fn require_u32_in_range(
    location: &ResourceLocation,
    kind: &str,
    field: &str,
    value: u32,
    min: u32,
    max: u32,
) -> Result<()> {
    if value < min || value > max {
        return Err(error(
            location,
            kind,
            field,
            &format!("{field} must be in {min}..={max}; received {value}"),
        ));
    }
    Ok(())
}

pub(crate) fn require_non_empty_collection(
    location: &ResourceLocation,
    kind: &str,
    field: &str,
    len: usize,
) -> Result<()> {
    if len == 0 {
        return Err(error(
            location,
            kind,
            field,
            &format!("{field} must not be empty"),
        ));
    }
    Ok(())
}

fn is_valid_namespace_char(c: char) -> bool {
    matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '-')
}

fn is_valid_path_char(c: char) -> bool {
    matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '-' | '/')
}

/// Validates the namespace/path syntax of `target` (the part of `value` after
/// any leading `#` has already been stripped or rejected by the caller).
/// `value` is the original, unstripped input, used only for error messages so
/// diagnostics always show what the user actually wrote.
fn validate_resource_location_chars(
    location: &ResourceLocation,
    kind: &str,
    field: &str,
    value: &str,
    target: &str,
) -> Result<()> {
    if target.is_empty() {
        return Err(error(
            location,
            kind,
            field,
            &format!("{field} must be a valid resource location; received empty string"),
        ));
    }
    match target.split_once(':') {
        Some((ns, path)) => {
            if ns.is_empty() {
                return Err(error(
                    location,
                    kind,
                    field,
                    &format!("{field} namespace must not be empty in `{value}`"),
                ));
            }
            if !ns.chars().all(is_valid_namespace_char) {
                return Err(error(
                    location,
                    kind,
                    field,
                    &format!("{field} namespace must only contain [a-z0-9_.-] in `{value}`"),
                ));
            }
            if path.is_empty() {
                return Err(error(
                    location,
                    kind,
                    field,
                    &format!("{field} path must not be empty in `{value}`"),
                ));
            }
            if !path.chars().all(is_valid_path_char) {
                return Err(error(
                    location,
                    kind,
                    field,
                    &format!("{field} path must only contain [a-z0-9_./-] in `{value}`"),
                ));
            }
        }
        None => {
            if !target.chars().all(is_valid_path_char) {
                return Err(error(
                    location,
                    kind,
                    field,
                    &format!(
                        "{field} must be a valid resource location (path only: [a-z0-9_./-]+); received `{value}`"
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// Validates that `value` is a **plain** resource location (`namespace:path`
/// or bare `path`), rejecting a leading `#`.
///
/// Use this for fields that are serialized as a single concrete registry
/// entry, never a tag reference — e.g. `Instrument`/`JukeboxSong`
/// `sound_event`. A `#`-prefixed value there would be written unchanged into
/// the datapack and only fail later, at world load, instead of at export
/// time.
pub(crate) fn validate_resource_location_str(
    location: &ResourceLocation,
    kind: &str,
    field: &str,
    value: &str,
) -> Result<()> {
    if value.starts_with('#') {
        return Err(error(
            location,
            kind,
            field,
            &format!(
                "{field} must be a plain resource location, not a tag reference; received `{value}`"
            ),
        ));
    }
    validate_resource_location_chars(location, kind, field, value, value)
}

/// Validates that `value` is either a plain resource location or a tag
/// reference (`#namespace:path`), stripping the leading `#` (if present)
/// before checking namespace/path syntax.
///
/// Use this for fields Minecraft documents as accepting a tag or a single
/// entry — e.g. `Enchantment` `supported_items`, `primary_items`, and
/// `exclusive_set`. Never use this for a field that is serialized as a
/// single concrete registry entry; see [`validate_resource_location_str`].
pub(crate) fn validate_resource_or_tag_location_str(
    location: &ResourceLocation,
    kind: &str,
    field: &str,
    value: &str,
) -> Result<()> {
    let target = value.strip_prefix('#').unwrap_or(value);
    validate_resource_location_chars(location, kind, field, value, target)
}

pub(crate) fn require_json_object(
    location: &ResourceLocation,
    kind: &str,
    field: &str,
    value: &Value,
) -> Result<()> {
    if !value.is_object() {
        return Err(error(
            location,
            kind,
            field,
            &format!("{field} must be a JSON object"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rl() -> ResourceLocation {
        ResourceLocation::new("test", "loc").unwrap()
    }

    // ── validate_resource_location_str (plain — rejects tags) ─────────────────

    #[test]
    fn plain_validator_accepts_valid_namespaced_id() {
        assert!(
            validate_resource_location_str(&rl(), "kind", "field", "minecraft:music_disc.13")
                .is_ok()
        );
    }

    #[test]
    fn plain_validator_accepts_valid_path_only_id() {
        assert!(validate_resource_location_str(&rl(), "kind", "field", "my_sound").is_ok());
    }

    #[test]
    fn plain_validator_rejects_tag_prefix() {
        let err = validate_resource_location_str(&rl(), "kind", "field", "#minecraft:music_disc")
            .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("field"), "{msg}");
        assert!(msg.contains("tag"), "{msg}");
    }

    #[test]
    fn plain_validator_rejects_empty_value() {
        assert!(validate_resource_location_str(&rl(), "kind", "field", "").is_err());
    }

    #[test]
    fn plain_validator_rejects_malformed_namespace() {
        assert!(validate_resource_location_str(&rl(), "kind", "field", "Bad NS:path").is_err());
    }

    #[test]
    fn plain_validator_rejects_uppercase() {
        assert!(validate_resource_location_str(&rl(), "kind", "field", "minecraft:Music").is_err());
    }

    #[test]
    fn plain_validator_rejects_whitespace() {
        assert!(
            validate_resource_location_str(&rl(), "kind", "field", "minecraft:music disc").is_err()
        );
    }

    #[test]
    fn plain_validator_rejects_empty_namespace() {
        assert!(validate_resource_location_str(&rl(), "kind", "field", ":path").is_err());
    }

    #[test]
    fn plain_validator_rejects_empty_path_after_colon() {
        assert!(validate_resource_location_str(&rl(), "kind", "field", "minecraft:").is_err());
    }

    // ── validate_resource_or_tag_location_str (either — permits tags) ─────────

    #[test]
    fn tag_validator_accepts_valid_namespaced_id() {
        assert!(
            validate_resource_or_tag_location_str(
                &rl(),
                "kind",
                "field",
                "minecraft:diamond_sword"
            )
            .is_ok()
        );
    }

    #[test]
    fn tag_validator_accepts_valid_tag_reference() {
        assert!(
            validate_resource_or_tag_location_str(
                &rl(),
                "kind",
                "field",
                "#minecraft:enchantable/sword"
            )
            .is_ok()
        );
    }

    #[test]
    fn tag_validator_rejects_malformed_namespace_under_tag() {
        assert!(
            validate_resource_or_tag_location_str(&rl(), "kind", "field", "#Bad NS:path").is_err()
        );
    }

    #[test]
    fn tag_validator_rejects_empty_value() {
        assert!(validate_resource_or_tag_location_str(&rl(), "kind", "field", "").is_err());
    }

    #[test]
    fn tag_validator_rejects_bare_hash() {
        assert!(validate_resource_or_tag_location_str(&rl(), "kind", "field", "#").is_err());
    }
}
