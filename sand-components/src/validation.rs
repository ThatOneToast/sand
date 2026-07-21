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

pub(crate) fn validate_resource_location_str(
    location: &ResourceLocation,
    kind: &str,
    field: &str,
    value: &str,
) -> Result<()> {
    let target = value.strip_prefix('#').unwrap_or(value);
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
