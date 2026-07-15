//! Shared fallible validation and rendering boundary for typed commands.

use crate::error::{CommandError, CommandResult};

pub use sand_version::CommandProfile;

/// Validate a collected command line and return the exact text to write.
///
/// Collected lines receive conservative foundational checks, including lines
/// introduced through raw compatibility APIs. Raw builders bypass structural
/// typed validation but never bypass `.mcfunction` file-integrity checks.
pub fn validate_collected_line(line: &str, _profile: &CommandProfile) -> CommandResult<String> {
    if line.contains(['\0', '\n', '\r']) {
        return Err(CommandError::new(
            "command_line",
            "text",
            "commands must contain exactly one line",
        ));
    }
    if line.trim_start().starts_with('/') {
        return Err(CommandError::new(
            "command_line",
            "text",
            "commands in `.mcfunction` files must not start with `/`",
        ));
    }
    if line.trim().is_empty() || line.trim_start().starts_with('#') {
        return Ok(line.to_string());
    }

    // This is deliberately a fallback boundary: typed `RenderCommand` values
    // have already validated their own structure. Do not search arbitrary
    // collected text, which may contain JSON, SNBT, macros, or raw/modded
    // syntax. Only validate operands whose command position is known.
    validate_known_command(&tokenize_command(line)?)?;
    Ok(line.to_string())
}

fn tokenize_command(line: &str) -> CommandResult<Vec<&str>> {
    let mut tokens = Vec::new();
    let mut start = None;
    let mut quoted = false;
    let mut escaped = false;
    let mut depth = 0usize;
    for (index, character) in line.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                quoted = false;
            }
            continue;
        }
        match character {
            '"' => quoted = true,
            '[' | '{' | '(' => depth += 1,
            ']' | '}' | ')' => depth = depth.saturating_sub(1),
            character if character.is_ascii_whitespace() && depth == 0 => {
                if let Some(start) = start.take() {
                    tokens.push(&line[start..index]);
                }
            }
            _ if start.is_none() => start = Some(index),
            _ => {}
        }
    }
    if quoted || depth != 0 {
        // Arbitrary raw syntax may have grammar Sand does not model; leave it
        // intact rather than interpreting it as a known command.
        return Ok(vec![]);
    }
    if let Some(start) = start {
        tokens.push(&line[start..]);
    }
    Ok(tokens)
}

fn validate_known_command(tokens: &[&str]) -> CommandResult<()> {
    match tokens {
        ["kill", selector, ..] => validate_rendered_selector(selector),
        ["tp" | "teleport", target, rest @ ..] => {
            validate_rendered_selector(target)?;
            validate_finite_tokens(rest)
        }
        ["scoreboard", ..] => validate_known_scoreboard_syntax(tokens),
        ["execute", rest @ ..] => validate_execute_syntax(rest),
        ["function", id, ..] if !id.starts_with('$') => crate::validate::resource_location_shape(
            id.strip_prefix('#').unwrap_or(id),
            "function",
            "id",
        )
        .map(|_| ()),
        _ => Ok(()),
    }
}

fn validate_execute_syntax(tokens: &[&str]) -> CommandResult<()> {
    for window in tokens.windows(4) {
        if (window[0] == "if" || window[0] == "unless") && window[1] == "score" {
            validate_rendered_selector(window[2])?;
            let holder = window[2];
            if matches!(holder, "@a" | "@e")
                || ((holder.starts_with("@a[") || holder.starts_with("@e["))
                    && !holder.contains("limit=1"))
            {
                return Err(CommandError::new(
                    "execute_score_condition",
                    "holder",
                    format!(
                        "score condition holder `{holder}` may resolve to multiple entities; execute as the targets and use `@s`"
                    ),
                ));
            }
            validate_objective_token(window[3], "objective")?;
        }
    }
    if let Some(run) = tokens.iter().position(|token| *token == "run") {
        validate_known_command(&tokens[run + 1..])?;
    }
    Ok(())
}

fn validate_finite_tokens(tokens: &[&str]) -> CommandResult<()> {
    for token in tokens {
        let bare = token.trim_start_matches(['~', '^']);
        if matches!(
            bare,
            "NaN" | "nan" | "inf" | "+inf" | "-inf" | "Infinity" | "+Infinity" | "-Infinity"
        ) {
            return Err(CommandError::new(
                "command_line",
                "number",
                format!("non-finite numeric token `{bare}`"),
            ));
        }
    }
    Ok(())
}

fn validate_rendered_selector(selector: &str) -> CommandResult<()> {
    if !selector.starts_with('@') {
        return Ok(());
    }
    let Some(base) = selector.as_bytes().get(1) else {
        return Ok(());
    };
    if !matches!(base, b'a' | b'e' | b'p' | b's' | b'r') || selector.len() == 2 {
        return Ok(());
    }
    let Some(args) = selector
        .strip_prefix("@a[")
        .or_else(|| selector.strip_prefix("@e["))
        .or_else(|| selector.strip_prefix("@p["))
        .or_else(|| selector.strip_prefix("@s["))
        .or_else(|| selector.strip_prefix("@r["))
    else {
        return Ok(());
    };
    let Some(args) = args.strip_suffix(']') else {
        return Err(CommandError::new(
            "Selector",
            "arguments",
            "unclosed selector argument list",
        ));
    };
    for arg in split_top_level(args) {
        let Some((key, value)) = arg.split_once('=') else {
            return Err(CommandError::new(
                "Selector",
                "arguments",
                format!("expected `key=value`, got `{arg}`"),
            ));
        };
        if key == "limit" {
            let limit = value.parse::<i32>().map_err(|_| {
                CommandError::new("Selector", "limit", format!("invalid integer `{value}`"))
            })?;
            if limit <= 0 {
                return Err(CommandError::new(
                    "Selector",
                    "limit",
                    format!("selector limits must be greater than zero, got `{limit}`"),
                ));
            }
        }
    }
    Ok(())
}

fn split_top_level(value: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut quoted = false;
    let bytes = value.as_bytes();
    for (i, byte) in bytes.iter().copied().enumerate() {
        match byte {
            b'"' if i == 0 || bytes[i - 1] != b'\\' => quoted = !quoted,
            b'{' | b'[' if !quoted => depth += 1,
            b'}' | b']' if !quoted => depth = depth.saturating_sub(1),
            b',' if !quoted && depth == 0 => {
                result.push(&value[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    result.push(&value[start..]);
    result
}

fn validate_known_scoreboard_syntax(tokens: &[&str]) -> CommandResult<()> {
    for window in tokens.windows(6) {
        if window[0] == "scoreboard" && window[1] == "players" && window[2] == "operation" {
            validate_objective_token(window[4], "target_objective")?;
        }
    }
    if tokens.starts_with(&["scoreboard", "players", "operation"]) && tokens.len() >= 8 {
        let source = tokens[6];
        validate_objective_token(tokens[7], "source_objective")?;
        if matches!(source, "@a" | "@e")
            || ((source.starts_with("@a[") || source.starts_with("@e["))
                && !source.contains("limit=1"))
        {
            return Err(CommandError::new(
                "scoreboard_players_operation",
                "source",
                format!(
                    "source `{source}` may resolve to multiple score holders; execute per entity and use `@s`"
                ),
            ));
        }
    }
    for window in tokens.windows(5) {
        if (window[0] == "if" || window[0] == "unless") && window[1] == "score" {
            let holder = window[2];
            if matches!(holder, "@a" | "@e")
                || ((holder.starts_with("@a[") || holder.starts_with("@e["))
                    && !holder.contains("limit=1"))
            {
                return Err(CommandError::new(
                    "execute_score_condition",
                    "holder",
                    format!(
                        "score condition holder `{holder}` may resolve to multiple entities; execute as the targets and use `@s`"
                    ),
                ));
            }
            validate_objective_token(window[3], "objective")?;
        }
    }
    if tokens.starts_with(&["scoreboard", "objectives", "add"]) && tokens.len() >= 4 {
        validate_objective_token(tokens[3], "objective")?;
    }
    Ok(())
}

fn validate_objective_token(value: &str, field: &'static str) -> CommandResult<()> {
    if value.is_empty()
        || value.len() > 16
        || value.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        Err(CommandError::new(
            "scoreboard",
            field,
            format!("invalid objective name `{value}`"),
        ))
    } else {
        Ok(())
    }
}

/// Validate a typed command value against the active Minecraft profile.
pub trait Validate {
    /// Reject invalid state before it can become command text.
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()>;
}

/// Deterministically render a validated Minecraft command or command fragment.
///
/// [`render`](Self::render) is the normal path. `render_unchecked` exists so
/// compatibility `Display`/`Build` implementations can retain their historical
/// output while exporters and new APIs use the fallible boundary.
pub trait RenderCommand: Validate {
    /// Render without validation. Normal callers should use [`render`](Self::render).
    fn render_unchecked(&self, profile: &CommandProfile) -> String;

    /// Validate, then render with the active Minecraft profile.
    fn render(&self, profile: &CommandProfile) -> CommandResult<String> {
        self.validate(profile)?;
        Ok(self.render_unchecked(profile))
    }

    /// Validate and render using the unprofiled compatibility target.
    fn try_build(&self) -> CommandResult<String> {
        self.render(&CommandProfile::unprofiled())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collected_validation_rejects_foundational_malformed_output() {
        let profile = CommandProfile::new("1.21.11", false);
        assert!(validate_collected_line("kill @e[limit=-1]", &profile).is_err());
        assert!(validate_collected_line("tp @s NaN 0 0", &profile).is_err());
        assert!(
            validate_collected_line("scoreboard players operation @s a = @a b", &profile).is_err()
        );
        assert!(
            validate_collected_line("execute if score @a mana matches 1.. run say no", &profile)
                .is_err()
        );
    }

    #[test]
    fn collected_raw_text_still_obeys_file_integrity() {
        let profile = CommandProfile::unprofiled();
        assert!(validate_collected_line("/say no", &profile).is_err());
        assert_eq!(
            validate_collected_line("modded command syntax", &profile).unwrap(),
            "modded command syntax"
        );
    }

    #[test]
    fn collected_validation_does_not_scan_data_or_raw_syntax() {
        let profile = CommandProfile::new("1.21.11", false);
        for line in [
            r#"tellraw @a {"text":"example @e[limit=-1]"}"#,
            r#"data modify storage pack:test message set value "function not_a_resource_location""#,
            "say function plain-text",
            r#"modded command {payload:\"@e[limit=-1] scoreboard players operation @a a = @a b\"}"#,
        ] {
            assert_eq!(validate_collected_line(line, &profile).unwrap(), line);
        }
    }
}
