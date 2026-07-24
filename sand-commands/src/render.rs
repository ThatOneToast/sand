//! Shared fallible validation and rendering boundary for typed commands.

use crate::error::{CommandError, CommandResult};

pub use sand_version::CommandProfile;

/// Validate a collected command line and return the exact text to write.
///
/// Collected lines always receive `.mcfunction` line-integrity checks. A small
/// set of known top-level commands receives additional argument-position-aware
/// validation; unknown, macro, and modded commands remain verbatim.
pub fn validate_collected_line(line: &str, profile: &CommandProfile) -> CommandResult<String> {
    validate_line_integrity(line)?;
    crate::execute_ir::validate_registered_line(line, profile)?;
    let trimmed = line.trim_start();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('$') {
        return Ok(line.to_string());
    }

    let Some(tokens) = tokenize_top_level(line) else {
        // An incomplete tokenizer model must never make raw syntax unexportable.
        return Ok(line.to_string());
    };
    validate_known_command(line, &tokens, profile)?;
    Ok(line.to_string())
}

fn validate_line_integrity(line: &str) -> CommandResult<()> {
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
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct TopLevelToken<'a> {
    text: &'a str,
    start: usize,
}

/// Split only on whitespace outside quotes and balanced structured arguments.
/// Returning `None` means the input uses structure this conservative tokenizer
/// cannot prove; callers must preserve the raw line after integrity checks.
fn tokenize_top_level(line: &str) -> Option<Vec<TopLevelToken<'_>>> {
    let mut tokens = Vec::new();
    let mut start = None;
    let mut quote = None;
    let mut escaped = false;
    let mut delimiters = Vec::new();

    for (index, character) in line.char_indices() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }

        match character {
            '\'' | '"' => {
                start.get_or_insert(index);
                quote = Some(character);
            }
            '{' | '[' | '(' => {
                start.get_or_insert(index);
                delimiters.push(character);
            }
            '}' | ']' | ')' => {
                let expected = match character {
                    '}' => '{',
                    ']' => '[',
                    ')' => '(',
                    _ => unreachable!(),
                };
                if delimiters.pop() != Some(expected) {
                    return None;
                }
            }
            c if c.is_ascii_whitespace() && delimiters.is_empty() => {
                if let Some(token_start) = start.take() {
                    tokens.push(TopLevelToken {
                        text: &line[token_start..index],
                        start: token_start,
                    });
                }
            }
            _ => {
                start.get_or_insert(index);
            }
        }
    }

    if quote.is_some() || !delimiters.is_empty() {
        return None;
    }
    if let Some(token_start) = start {
        tokens.push(TopLevelToken {
            text: &line[token_start..],
            start: token_start,
        });
    }
    Some(tokens)
}

fn validate_known_command(
    line: &str,
    tokens: &[TopLevelToken<'_>],
    profile: &CommandProfile,
) -> CommandResult<()> {
    let Some(command) = tokens.first().map(|token| token.text) else {
        return Ok(());
    };
    match command {
        "function" => validate_function_command(tokens),
        "scoreboard" => validate_scoreboard_command(tokens),
        "execute" => validate_execute_command(line, tokens, profile),
        "kill" | "tellraw" => validate_selector_at(tokens, 1),
        _ => Ok(()),
    }
}

fn split_top_level(value: &str) -> Option<Vec<&str>> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut quote = None;
    let mut escaped = false;
    let mut delimiters = Vec::new();
    for (index, character) in value.char_indices() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '{' | '[' | '(' => delimiters.push(character),
            '}' | ']' | ')' => {
                let expected = match character {
                    '}' => '{',
                    ']' => '[',
                    ')' => '(',
                    _ => unreachable!(),
                };
                if delimiters.pop() != Some(expected) {
                    return None;
                }
            }
            ',' if delimiters.is_empty() => {
                result.push(&value[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    if quote.is_some() || !delimiters.is_empty() {
        return None;
    }
    result.push(&value[start..]);
    Some(result)
}

fn validate_scoreboard_command(tokens: &[TopLevelToken<'_>]) -> CommandResult<()> {
    if token_is(tokens, 1, "objectives") && token_is(tokens, 2, "add") {
        let objective = required_token(tokens, 3, "scoreboard", "objective")?;
        return validate_objective_token(objective, "objective");
    }
    if !(token_is(tokens, 1, "players") && token_is(tokens, 2, "operation")) {
        return Ok(());
    }

    let target = required_token(tokens, 3, "scoreboard_players_operation", "target")?;
    let target_objective = required_token(
        tokens,
        4,
        "scoreboard_players_operation",
        "target_objective",
    )?;
    let source = required_token(tokens, 6, "scoreboard_players_operation", "source")?;
    let source_objective = required_token(
        tokens,
        7,
        "scoreboard_players_operation",
        "source_objective",
    )?;
    validate_selector_token_if_present(target)?;
    validate_objective_token(target_objective, "target_objective")?;
    validate_single_score_holder(source, "scoreboard_players_operation", "source")?;
    validate_objective_token(source_objective, "source_objective")?;
    Ok(())
}

fn validate_execute_command(
    line: &str,
    tokens: &[TopLevelToken<'_>],
    profile: &CommandProfile,
) -> CommandResult<()> {
    let run_index = tokens
        .iter()
        .enumerate()
        .skip(1)
        .find_map(|(index, token)| (token.text == "run").then_some(index));
    if run_index == Some(1) {
        return Err(CommandError::new(
            "Execute",
            "operations",
            "execute chains require at least one operation before `run`",
        )
        .with_code("SAND-COMMAND-EXECUTE-EMPTY"));
    }
    let subcommand_end = run_index.unwrap_or(tokens.len());
    let mut index = 1;
    while index < subcommand_end {
        match tokens[index].text {
            "as" | "at" => {
                validate_selector_at(tokens, index + 1)?;
                index += 2;
            }
            "if" | "unless" if token_is(tokens, index + 1, "entity") => {
                validate_selector_at(tokens, index + 2)?;
                index += 3;
            }
            "if" | "unless" if token_is(tokens, index + 1, "score") => {
                index = validate_execute_score(tokens, index)?;
            }
            // The fallback intentionally stops at syntax it cannot model with
            // confidence, but a top-level `run` still safely delimits the
            // nested command. Typed Execute retains structural validation.
            _ => break,
        }
    }
    if let Some(run_index) = run_index {
        required_token(tokens, run_index + 1, "execute", "run")?;
        validate_collected_line(&line[tokens[run_index + 1].start..], profile)?;
    }
    Ok(())
}

fn validate_execute_score(tokens: &[TopLevelToken<'_>], index: usize) -> CommandResult<usize> {
    let holder = required_token(tokens, index + 2, "execute_score_condition", "holder")?;
    let objective = required_token(tokens, index + 3, "execute_score_condition", "objective")?;
    validate_single_score_holder(holder, "execute_score_condition", "holder")?;
    validate_objective_token(objective, "objective")?;

    if token_is(tokens, index + 4, "matches") {
        required_token(tokens, index + 5, "execute_score_condition", "range")?;
        return Ok(index + 6);
    }

    let source = required_token(tokens, index + 5, "execute_score_condition", "source")?;
    let source_objective = required_token(
        tokens,
        index + 6,
        "execute_score_condition",
        "source_objective",
    )?;
    validate_single_score_holder(source, "execute_score_condition", "source")?;
    validate_objective_token(source_objective, "source_objective")?;
    Ok(index + 7)
}

fn validate_function_command(tokens: &[TopLevelToken<'_>]) -> CommandResult<()> {
    let id = required_token(tokens, 1, "function", "id")?;
    if id.contains("$(") {
        return Ok(());
    }
    crate::validate::resource_location_shape(id.strip_prefix('#').unwrap_or(id), "function", "id")?;
    Ok(())
}

fn validate_objective_token(value: &str, field: &'static str) -> CommandResult<()> {
    if value.is_empty()
        || value.len() > 16
        || matches!(value, "\"\"" | "''")
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

fn validate_selector_at(tokens: &[TopLevelToken<'_>], index: usize) -> CommandResult<()> {
    let selector = required_token(tokens, index, "command", "target")?;
    validate_selector_token_if_present(selector)
}

fn validate_selector_token_if_present(token: &str) -> CommandResult<()> {
    if !token.starts_with('@') {
        return Ok(());
    }
    validate_selector_token(token).map(|_| ())
}

fn validate_single_score_holder(
    holder: &str,
    helper: &'static str,
    field: &'static str,
) -> CommandResult<()> {
    let limit = if holder.starts_with('@') {
        selector_limit(holder)?
    } else {
        None
    };
    if matches!(holder, "@a" | "@e")
        || ((holder.starts_with("@a[") || holder.starts_with("@e[")) && limit != Some(1))
    {
        return Err(CommandError::new(
            helper,
            field,
            format!(
                "score holder `{holder}` may resolve to multiple entities; execute as the targets and use `@s`"
            ),
        ));
    }
    Ok(())
}

fn validate_selector_token(token: &str) -> CommandResult<Option<&str>> {
    let bytes = token.as_bytes();
    if bytes.len() < 2 || bytes[0] != b'@' || !matches!(bytes[1], b'a' | b'e' | b'p' | b's' | b'r')
    {
        return Err(CommandError::new(
            "Selector",
            "base",
            format!("invalid selector base `{token}`"),
        ));
    }
    if bytes.len() == 2 {
        return Ok(None);
    }
    if bytes[2] != b'[' || !token.ends_with(']') {
        return Err(CommandError::new(
            "Selector",
            "arguments",
            format!("invalid selector argument list `{token}`"),
        ));
    }

    let arguments = &token[3..token.len() - 1];
    for argument in split_top_level(arguments).ok_or_else(|| {
        CommandError::new("Selector", "arguments", "unbalanced selector arguments")
    })? {
        let Some((key, value)) = argument.split_once('=') else {
            return Err(CommandError::new(
                "Selector",
                "arguments",
                format!("expected `key=value`, got `{argument}`"),
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
        if matches!(key, "x" | "y" | "z" | "dx" | "dy" | "dz") {
            let field = match key {
                "x" => "x",
                "y" => "y",
                "z" => "z",
                "dx" => "dx",
                "dy" => "dy",
                "dz" => "dz",
                _ => unreachable!(),
            };
            let number = value.parse::<f64>().map_err(|_| {
                CommandError::new("Selector", field, format!("invalid number `{value}`"))
            })?;
            if !number.is_finite() {
                return Err(CommandError::new("Selector", field, "must be finite"));
            }
        }
    }
    Ok(Some(arguments))
}

fn selector_limit(token: &str) -> CommandResult<Option<i32>> {
    let Some(arguments) = validate_selector_token(token)? else {
        return Ok(None);
    };
    for argument in split_top_level(arguments).unwrap_or_default() {
        if let Some(value) = argument.strip_prefix("limit=") {
            return value.parse::<i32>().map(Some).map_err(|_| {
                CommandError::new("Selector", "limit", format!("invalid integer `{value}`"))
            });
        }
    }
    Ok(None)
}

fn token_is(tokens: &[TopLevelToken<'_>], index: usize, expected: &str) -> bool {
    tokens
        .get(index)
        .is_some_and(|token| token.text == expected)
}

fn required_token<'a>(
    tokens: &'a [TopLevelToken<'a>],
    index: usize,
    helper: &'static str,
    field: &'static str,
) -> CommandResult<&'a str> {
    tokens.get(index).map(|token| token.text).ok_or_else(|| {
        CommandError::new(
            helper,
            field,
            format!("missing required `{field}` argument"),
        )
    })
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
    fn collected_validation_preserves_literal_raw_and_modded_content() {
        let profile = CommandProfile::new("1.21.11", false);
        let valid = [
            r#"tellraw @a {"text":"example @e[limit=-1]"}"#,
            r#"tellraw @a {"text":"function not_a_resource_location"}"#,
            r#"data modify storage pack:test message set value "function not_a_resource_location""#,
            r#"data modify storage pack:test value set value {message:"@e[limit=-1]"}"#,
            "say function plain-text",
            "say scoreboard players operation @s a = @a b",
            r#"custom_command "@e[limit=-1]""#,
            "modded command syntax",
            r#"tellraw @a {"text":"quoted \"@e[limit=-1]\"","extra":[{"text":"function bad"}]}"#,
            r#"data modify storage pack:test value set value {message:'literal "function bad" and @e[limit=-1]',nested:{values:[1,2,{text:"scoreboard players operation"}]}}"#,
            r#"$data modify storage pack:test value set value "$(payload)""#,
            "tp @s NaN 0 0",
            "scoreboard players operation @s total += #constant constants",
            "execute if score #constant constants matches 1 run say valid",
        ];
        for line in valid {
            assert_eq!(
                validate_collected_line(line, &profile).unwrap(),
                line,
                "line must survive unchanged: {line}"
            );
        }
    }

    #[test]
    fn collected_validation_rejects_confidently_recognized_malformed_output() {
        let profile = CommandProfile::new("1.21.11", false);
        assert!(validate_collected_line("/kill @s", &profile).is_err());
        assert!(validate_collected_line("kill @e[limit=-1]", &profile).is_err());
        assert!(validate_collected_line("scoreboard objectives add \"\"", &profile).is_err());
        assert!(
            validate_collected_line("scoreboard players operation @s a = @a b", &profile).is_err()
        );
        assert!(validate_collected_line("function not_a_resource_location", &profile).is_err());
        assert!(
            validate_collected_line("execute if score @a mana matches 1.. run say no", &profile)
                .is_err()
        );
        for line in [
            "scoreboard players operation @s target = @a source",
            "scoreboard players operation @s target += @e source",
            "scoreboard players operation @s target = @a[scores={source=1..}] source",
            "execute as @a run scoreboard players operation @s target = @a source",
            "execute as @e run scoreboard players operation @s target = @e source",
        ] {
            assert!(
                validate_collected_line(line, &profile).is_err(),
                "multi-holder operation source must be rejected: {line}"
            );
        }
    }

    #[test]
    fn collected_raw_text_still_obeys_file_integrity() {
        let profile = CommandProfile::unprofiled();
        assert!(validate_collected_line("/say no", &profile).is_err());
        assert!(validate_collected_line("say no\r", &profile).is_err());
        assert!(validate_collected_line("say no\0", &profile).is_err());
        assert_eq!(
            validate_collected_line("modded command syntax", &profile).unwrap(),
            "modded command syntax"
        );
    }

    #[test]
    fn execute_recursion_only_validates_the_nested_top_level_command() {
        let profile = CommandProfile::unprofiled();
        for line in [
            "execute as @s run function pack:valid",
            "execute as @s run say function plain-text",
            r#"execute as @s run tellraw @a {"text":"@e[limit=-1] function invalid"}"#,
            r#"execute as @s run data modify storage pack:test value set value {message:"@e[limit=-1]"}"#,
            "execute future_extension value run custom_command @e[limit=-1]",
        ] {
            assert_eq!(validate_collected_line(line, &profile).unwrap(), line);
        }
        assert!(
            validate_collected_line(
                "execute as @s run function not_a_resource_location",
                &profile
            )
            .is_err()
        );
        assert!(
            validate_collected_line(
                "execute positioned ~ ~ ~ run function not_a_resource_location",
                &profile
            )
            .is_err()
        );
    }

    #[test]
    fn tokenizer_keeps_quoted_and_structured_arguments_whole() {
        let line = r#"tellraw @a {"text":"escaped \" quote","extra":[{"text":"two words"}]}"#;
        let tokens = tokenize_top_level(line).unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "tellraw");
        assert_eq!(tokens[1].text, "@a");
        assert_eq!(tokens[2].text, &line[tokens[2].start..]);
    }
}
