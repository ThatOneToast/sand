//! Validation of inline dialog payloads carried by raw text click events.

use super::{
    json_event_string, text_error, validate_json_identifier, validate_json_interactions,
    validate_json_text_inner,
};
use crate::{CommandProfile, CommandResult};

/// Match Minecraft's UTF-16 Character.isLetterOrDigit variable-name check.
/// Empty names are valid in the 26.2 codec; supplementary letters are not.
fn validate_variable_name(value: &str, path: &str) -> CommandResult<()> {
    use unicode_general_category::{GeneralCategory, get_general_category};
    if value.chars().all(|ch| {
        ch == '_'
            || (ch <= '\u{ffff}'
                && matches!(
                    get_general_category(ch),
                    GeneralCategory::UppercaseLetter
                        | GeneralCategory::LowercaseLetter
                        | GeneralCategory::TitlecaseLetter
                        | GeneralCategory::ModifierLetter
                        | GeneralCategory::OtherLetter
                        | GeneralCategory::DecimalNumber
                ))
    }) {
        Ok(())
    } else {
        Err(text_error(
            "SAND-TEXT-SHAPE",
            path,
            "expected a template variable containing BMP letters, decimal digits or underscores",
        ))
    }
}

/// Require at least one complete macro variable, using the input-key grammar.
fn validate_command_template(mut value: &str, path: &str) -> CommandResult<()> {
    let mut has_variable = false;
    while let Some(start) = value.find("$(") {
        value = &value[start + 2..];
        let end = value
            .find(')')
            .ok_or_else(|| text_error("SAND-TEXT-SHAPE", path, "unterminated template variable"))?;
        validate_variable_name(&value[..end], path)?;
        has_variable = true;
        value = &value[end + 1..];
    }
    if has_variable {
        Ok(())
    } else {
        Err(text_error(
            "SAND-TEXT-SHAPE",
            path,
            "dynamic command templates require at least one variable",
        ))
    }
}

/// Require a JSON field while retaining the complete resource diagnostic path.
fn json_required<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
    path: &str,
) -> CommandResult<&'a serde_json::Value> {
    object.get(key).ok_or_else(|| {
        text_error(
            "SAND-TEXT-SHAPE",
            format!("{path}.{key}"),
            "required field is missing",
        )
    })
}

/// Validate optional integer codec fields without coercing strings or fractions.
fn json_integer(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    min: i64,
    max: i64,
    path: &str,
) -> CommandResult<()> {
    if let Some(value) = object.get(key)
        && !value
            .as_i64()
            .is_some_and(|value| (min..=max).contains(&value))
    {
        return Err(text_error(
            "SAND-TEXT-SHAPE",
            format!("{path}.{key}"),
            format!("expected an integer in {min}..={max}"),
        ));
    }
    Ok(())
}

/// Check optional boolean members shared by dialog and button codecs.
fn json_booleans(
    object: &serde_json::Map<String, serde_json::Value>,
    keys: &[&str],
    path: &str,
) -> CommandResult<()> {
    for key in keys {
        if let Some(value) = object.get(*key)
            && !value.is_boolean()
        {
            return Err(text_error(
                "SAND-TEXT-SHAPE",
                format!("{path}.{key}"),
                "expected a boolean",
            ));
        }
    }
    Ok(())
}

/// Validate a raw dialog reference or a known inline variant using shared text rules.
pub(super) fn validate_json_dialog(
    value: &serde_json::Value,
    profile: &CommandProfile,
    path: &str,
    depth: usize,
) -> CommandResult<()> {
    if depth > 64 {
        return Err(text_error(
            "SAND-TEXT-DEPTH",
            path,
            "dialog nesting exceeds 64",
        ));
    }
    let Some(dialog) = value.as_object() else {
        return validate_json_identifier(value, path);
    };
    let kind = json_event_string(dialog, "type", path)?;
    validate_json_identifier(
        &serde_json::Value::String(kind.into()),
        &format!("{path}.type"),
    )?;
    if kind.contains(':') && !kind.starts_with("minecraft:") {
        return Ok(());
    }
    let kind = kind.strip_prefix("minecraft:").unwrap_or(kind);
    if !matches!(
        kind,
        "notice" | "confirmation" | "multi_action" | "server_links" | "dialog_list"
    ) {
        return Err(text_error(
            "SAND-TEXT-SHAPE",
            format!("{path}.type"),
            "unknown vanilla dialog type",
        ));
    }
    validate_json_text_inner(
        json_required(dialog, "title", path)?,
        profile,
        &format!("{path}.title"),
        depth + 1,
    )?;
    if let Some(title) = dialog.get("external_title") {
        validate_json_text_inner(title, profile, &format!("{path}.external_title"), depth + 1)?;
    }
    json_booleans(dialog, &["pause", "can_close_with_escape"], path)?;
    if let Some(action) = dialog.get("after_action") {
        if !matches!(
            action.as_str(),
            Some("close" | "none" | "wait_for_response")
        ) {
            return Err(text_error(
                "SAND-TEXT-SHAPE",
                format!("{path}.after_action"),
                "unknown dialog completion action",
            ));
        }
        if action == "none"
            && dialog
                .get("pause")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true)
        {
            return Err(text_error(
                "SAND-TEXT-SHAPE",
                format!("{path}.after_action"),
                "a paused dialog must unpause after an action",
            ));
        }
    }
    match kind {
        "notice" => {
            if let Some(button) = dialog.get("action") {
                validate_json_dialog_button(button, profile, &format!("{path}.action"), depth + 1)?;
            }
        }
        "confirmation" => {
            for key in ["yes", "no"] {
                validate_json_dialog_button(
                    json_required(dialog, key, path)?,
                    profile,
                    &format!("{path}.{key}"),
                    depth + 1,
                )?;
            }
        }
        "multi_action" => {
            let actions = json_required(dialog, "actions", path)?
                .as_array()
                .filter(|a| !a.is_empty())
                .ok_or_else(|| {
                    text_error(
                        "SAND-TEXT-SHAPE",
                        format!("{path}.actions"),
                        "expected a non-empty action list",
                    )
                })?;
            for (i, action) in actions.iter().enumerate() {
                validate_json_dialog_button(
                    action,
                    profile,
                    &format!("{path}.actions[{i}]"),
                    depth + 1,
                )?;
            }
        }
        "dialog_list" => {
            let dialogs = json_required(dialog, "dialogs", path)?;
            if let Some(tag) = dialogs.as_str().and_then(|v| v.strip_prefix('#')) {
                validate_json_identifier(
                    &serde_json::Value::String(tag.into()),
                    &format!("{path}.dialogs"),
                )?;
            } else if let Some(dialogs) = dialogs.as_array() {
                for (i, dialog) in dialogs.iter().enumerate() {
                    validate_json_dialog(
                        dialog,
                        profile,
                        &format!("{path}.dialogs[{i}]"),
                        depth + 1,
                    )?;
                }
            } else {
                validate_json_dialog(dialogs, profile, &format!("{path}.dialogs"), depth + 1)?;
            }
        }
        _ => {}
    }
    if matches!(kind, "multi_action" | "server_links" | "dialog_list") {
        json_integer(dialog, "columns", 1, i32::MAX as i64, path)?;
        if let Some(button) = dialog.get("exit_action") {
            validate_json_dialog_button(
                button,
                profile,
                &format!("{path}.exit_action"),
                depth + 1,
            )?;
        }
    }
    if matches!(kind, "server_links" | "dialog_list") {
        json_integer(dialog, "button_width", 1, 1024, path)?;
    }
    if let Some(body) = dialog.get("body") {
        if let Some(elements) = body.as_array() {
            for (i, element) in elements.iter().enumerate() {
                validate_json_dialog_body(
                    element,
                    profile,
                    &format!("{path}.body[{i}]"),
                    depth + 1,
                )?;
            }
        } else {
            validate_json_dialog_body(body, profile, &format!("{path}.body"), depth + 1)?;
        }
    }
    if let Some(inputs) = dialog.get("inputs") {
        let inputs = inputs.as_array().ok_or_else(|| {
            text_error(
                "SAND-TEXT-SHAPE",
                format!("{path}.inputs"),
                "expected an input list",
            )
        })?;
        for (i, input) in inputs.iter().enumerate() {
            validate_json_dialog_input(input, profile, &format!("{path}.inputs[{i}]"), depth + 1)?;
        }
    }
    Ok(())
}

/// Validate button labels, widths, tooltips and their static or dynamic actions.
fn validate_json_dialog_button(
    value: &serde_json::Value,
    profile: &CommandProfile,
    path: &str,
    depth: usize,
) -> CommandResult<()> {
    let button = value
        .as_object()
        .ok_or_else(|| text_error("SAND-TEXT-SHAPE", path, "expected a button object"))?;
    validate_json_text_inner(
        json_required(button, "label", path)?,
        profile,
        &format!("{path}.label"),
        depth + 1,
    )?;
    if let Some(tooltip) = button.get("tooltip") {
        validate_json_text_inner(tooltip, profile, &format!("{path}.tooltip"), depth + 1)?;
    }
    json_integer(button, "width", 1, 1024, path)?;
    if let Some(action) = button.get("action") {
        let path = format!("{path}.action");
        let action = action
            .as_object()
            .ok_or_else(|| text_error("SAND-TEXT-SHAPE", &path, "expected an action object"))?;
        let kind = json_event_string(action, "type", &path)?;
        validate_json_identifier(
            &serde_json::Value::String(kind.into()),
            &format!("{path}.type"),
        )?;
        if kind.contains(':') && !kind.starts_with("minecraft:") {
            return Ok(());
        }
        let kind = kind.strip_prefix("minecraft:").unwrap_or(kind);
        match kind {
            "dynamic/run_command" => {
                validate_command_template(
                    json_event_string(action, "template", &path)?,
                    &format!("{path}.template"),
                )?;
            }
            "dynamic/custom" => {
                validate_json_identifier(
                    json_required(action, "id", &path)?,
                    &format!("{path}.id"),
                )?;
                if let Some(additions) = action.get("additions")
                    && !additions.is_object()
                {
                    return Err(text_error(
                        "SAND-TEXT-SHAPE",
                        format!("{path}.additions"),
                        "expected a compound object",
                    ));
                }
            }
            "open_url" | "run_command" | "suggest_command" | "show_dialog" | "change_page"
            | "copy_to_clipboard" | "custom" => {
                let mut click = action.clone();
                click.remove("type");
                click.insert("action".into(), serde_json::Value::String(kind.into()));
                let wrapper = serde_json::Map::from_iter([(
                    "click_event".into(),
                    serde_json::Value::Object(click),
                )]);
                validate_json_interactions(&wrapper, profile, &path, depth + 1).map_err(
                    |mut e| {
                        e.field = e.field.replacen(&format!("{path}.click_event"), &path, 1);
                        e
                    },
                )?;
            }
            _ => {
                return Err(text_error(
                    "SAND-TEXT-SHAPE",
                    format!("{path}.type"),
                    "unknown vanilla dialog action",
                ));
            }
        }
    }
    Ok(())
}

/// Validate dialog message and item bodies, retaining explicit extension kinds.
fn validate_json_dialog_body(
    value: &serde_json::Value,
    profile: &CommandProfile,
    path: &str,
    depth: usize,
) -> CommandResult<()> {
    let body = value
        .as_object()
        .ok_or_else(|| text_error("SAND-TEXT-SHAPE", path, "expected a dialog body object"))?;
    let kind = json_event_string(body, "type", path)?;
    validate_json_identifier(
        &serde_json::Value::String(kind.into()),
        &format!("{path}.type"),
    )?;
    if kind.contains(':') && !kind.starts_with("minecraft:") {
        return Ok(());
    }
    match kind.strip_prefix("minecraft:").unwrap_or(kind) {
        "plain_message" => {
            validate_json_text_inner(
                json_required(body, "contents", path)?,
                profile,
                &format!("{path}.contents"),
                depth + 1,
            )?;
            json_integer(body, "width", 1, 1024, path)?;
        }
        "item" => {
            let item = json_required(body, "item", path)?;
            let item_path = format!("{path}.item");
            if let Some(item) = item.as_object() {
                let mut hover = item.clone();
                hover.insert("action".into(), "show_item".into());
                let wrapper = serde_json::Map::from_iter([(
                    "hover_event".into(),
                    serde_json::Value::Object(hover),
                )]);
                validate_json_interactions(&wrapper, profile, &item_path, depth + 1).map_err(
                    |mut e| {
                        e.field =
                            e.field
                                .replacen(&format!("{item_path}.hover_event"), &item_path, 1);
                        e
                    },
                )?;
            } else {
                validate_json_identifier(item, &item_path)?;
            }
            if let Some(description) = body.get("description") {
                let p = format!("{path}.description");
                if let Some(message) = description
                    .as_object()
                    .filter(|o| o.contains_key("contents"))
                {
                    validate_json_text_inner(
                        &message["contents"],
                        profile,
                        &format!("{p}.contents"),
                        depth + 1,
                    )?;
                    json_integer(message, "width", 1, 1024, &p)?;
                } else {
                    validate_json_text_inner(description, profile, &p, depth + 1)?;
                }
            }
            json_booleans(body, &["show_decorations", "show_tooltip"], path)?;
            for key in ["width", "height"] {
                json_integer(body, key, 1, 256, path)?;
            }
        }
        _ => {
            return Err(text_error(
                "SAND-TEXT-SHAPE",
                format!("{path}.type"),
                "unknown vanilla dialog body type",
            ));
        }
    }
    Ok(())
}

/// Validate typed members of the four vanilla input controls without changing their JSON.
fn validate_json_dialog_input(
    value: &serde_json::Value,
    profile: &CommandProfile,
    path: &str,
    depth: usize,
) -> CommandResult<()> {
    let input = value
        .as_object()
        .ok_or_else(|| text_error("SAND-TEXT-SHAPE", path, "expected an input object"))?;
    validate_variable_name(
        json_event_string(input, "key", path)?,
        &format!("{path}.key"),
    )?;
    let kind = json_event_string(input, "type", path)?;
    validate_json_identifier(
        &serde_json::Value::String(kind.into()),
        &format!("{path}.type"),
    )?;
    if kind.contains(':') && !kind.starts_with("minecraft:") {
        return Ok(());
    }
    let kind = kind.strip_prefix("minecraft:").unwrap_or(kind);
    if !matches!(kind, "boolean" | "text" | "single_option" | "number_range") {
        return Err(text_error(
            "SAND-TEXT-SHAPE",
            format!("{path}.type"),
            "unknown vanilla input type",
        ));
    }
    validate_json_text_inner(
        json_required(input, "label", path)?,
        profile,
        &format!("{path}.label"),
        depth + 1,
    )?;
    if kind != "boolean" {
        json_integer(input, "width", 1, 1024, path)?;
    }
    match kind {
        "boolean" => {
            json_booleans(input, &["initial"], path)?;
            for key in ["on_true", "on_false"] {
                if input.contains_key(key) {
                    json_event_string(input, key, path)?;
                }
            }
        }
        "text" => {
            json_booleans(input, &["label_visible"], path)?;
            if input.contains_key("initial") {
                json_event_string(input, "initial", path)?;
            }
            json_integer(input, "max_length", 1, i32::MAX as i64, path)?;
            if let Some(multiline) = input.get("multiline") {
                let p = format!("{path}.multiline");
                let multiline = multiline.as_object().ok_or_else(|| {
                    text_error("SAND-TEXT-SHAPE", &p, "expected multiline options")
                })?;
                json_integer(multiline, "max_lines", 1, i32::MAX as i64, &p)?;
                json_integer(multiline, "height", 1, 512, &p)?;
            }
        }
        "single_option" => {
            json_booleans(input, &["label_visible"], path)?;
            let options = json_required(input, "options", path)?
                .as_array()
                .filter(|v| !v.is_empty())
                .ok_or_else(|| {
                    text_error(
                        "SAND-TEXT-SHAPE",
                        format!("{path}.options"),
                        "expected non-empty options",
                    )
                })?;
            let mut initial = 0;
            for (i, option) in options.iter().enumerate() {
                if option.is_string() {
                    continue;
                }
                let p = format!("{path}.options[{i}]");
                let option = option.as_object().ok_or_else(|| {
                    text_error("SAND-TEXT-SHAPE", &p, "expected a string or option object")
                })?;
                json_event_string(option, "id", &p)?;
                json_booleans(option, &["initial"], &p)?;
                initial += usize::from(
                    option.get("initial").and_then(serde_json::Value::as_bool) == Some(true),
                );
                if let Some(display) = option.get("display") {
                    validate_json_text_inner(display, profile, &format!("{p}.display"), depth + 1)?;
                }
            }
            if initial > 1 {
                return Err(text_error(
                    "SAND-TEXT-SHAPE",
                    format!("{path}.options"),
                    "only one option may be initial",
                ));
            }
        }
        "number_range" => {
            if input.contains_key("label_format") {
                json_event_string(input, "label_format", path)?;
            }
            let number = |key: &str| -> CommandResult<f32> {
                json_required(input, key, path)?
                    .as_f64()
                    .map(|n| n as f32)
                    .filter(|n| n.is_finite())
                    .ok_or_else(|| {
                        text_error(
                            "SAND-TEXT-SHAPE",
                            format!("{path}.{key}"),
                            "expected a finite float",
                        )
                    })
            };
            let start = number("start")?;
            let end = number("end")?;
            if input.contains_key("step") && number("step")? <= 0.0 {
                return Err(text_error(
                    "SAND-TEXT-SHAPE",
                    format!("{path}.step"),
                    "step must be positive",
                ));
            }
            if input.contains_key("initial")
                && !(start.min(end)..=start.max(end)).contains(&number("initial")?)
            {
                return Err(text_error(
                    "SAND-TEXT-SHAPE",
                    format!("{path}.initial"),
                    "initial value is outside the range",
                ));
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}
