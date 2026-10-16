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
    // Macro syntax remains opaque, but a known typed origin still carries
    // invariants independent of runtime substitution (for example slot kind).
    let mut command = line.trim_start();
    loop {
        validate_registered_origins(command, profile)?;
        let Some(inner) = collected_command_body(command) else {
            break;
        };
        command = inner.trim_start();
    }
    validate_standalone_line(line, profile)
}

fn validate_registered_origins(line: &str, profile: &CommandProfile) -> CommandResult<()> {
    crate::nbt::validate_registered_line(line, profile)?;
    crate::blocks::validate_registered_line(line, profile)?;
    crate::inventory::validate_registered_line(line, profile)?;
    crate::particles::validate_registered_line(line, profile)?;
    crate::sound::validate_registered_line(line, profile)?;
    crate::display::validate_registered_line(line, profile)?;
    crate::text::validate_registered_line(line, profile)?;
    crate::effect::validate_registered_line(line, profile)?;
    Ok(())
}

/// Locate one known executable body, including the body of a macro line.
/// Quoted data and arguments are never interpreted as command positions.
#[doc(hidden)]
pub fn collected_command_body(line: &str) -> Option<&str> {
    line.strip_prefix('$')
        .or_else(|| collected_execute_command(line))
        .or_else(|| collected_return_command(line))
}

/// Validate line integrity and supported syntax without consulting Rust's
/// export-scoped typed-node registries. Owned compiler backends validate their
/// typed nodes directly before using this final text boundary; explicit raw
/// operations receive only these documented syntactic checks.
pub fn validate_standalone_line(line: &str, profile: &CommandProfile) -> CommandResult<String> {
    validate_line_integrity(line)?;
    let trimmed = line.trim_start();
    if trimmed.starts_with('$') {
        return Ok(line.to_string());
    }
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Ok(line.to_string());
    }

    let Some(tokens) = tokenize_top_level(line) else {
        // An incomplete tokenizer model must never make raw syntax unexportable.
        return Ok(line.to_string());
    };
    validate_known_command(line, &tokens, profile)?;
    Ok(line.to_string())
}

pub(crate) fn validate_line_integrity(line: &str) -> CommandResult<()> {
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

/// Inspect top-level arguments without splitting quoted or structured data.
///
/// Compiler analyses share this lexical boundary with collected validation.
/// Tokens are borrowed from the original line. `None` denotes unsupported or
/// unbalanced structure, not an empty command or proof of zero runtime work.
#[doc(hidden)]
pub fn collected_tokens(line: &str) -> Option<Vec<&str>> {
    tokenize_top_level(line).map(|tokens| tokens.into_iter().map(|token| token.text).collect())
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
        "return" if token_is(tokens, 1, "run") => {
            required_token(tokens, 2, "return", "command")?;
            validate_collected_line(&line[tokens[2].start..], profile)?;
            Ok(())
        }
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

/// Locate the terminal command of a recognized collected `execute` chain.
///
/// Compiler validation uses this after typed commands have been rendered.
/// Reuses the collected-command tokenizer so quoted and structured arguments
/// stay opaque. Unknown operation grammar returns `None`, never a guessed
/// boundary in an argument or arbitrary raw command text.
#[doc(hidden)]
pub fn collected_execute_command(line: &str) -> Option<&str> {
    let tokens = tokenize_top_level(line)?;
    let index = execute_run_index(&tokens)?;
    Some(&line[tokens.get(index + 1)?.start..])
}

/// Borrowed lexical view of one recognized collected execute operation.
///
/// This is compiler inspection data, not an authoring API or proof that an
/// operation succeeds at runtime. Arguments retain quoted/structured tokens.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectedExecuteOperation<'a> {
    /// Operation keyword, such as `as`, `if`, or `store`.
    pub keyword: &'a str,
    /// Complete top-level argument tokens, excluding the operation keyword.
    pub arguments: Vec<&'a str>,
}

/// Recognized execute operations and their terminal command, in source order.
///
/// Conditions may invoke functions before the terminal command. Forking and
/// failure semantics belong to consumers of this lexical view; this structure
/// makes no claims about entity populations or execution cost.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectedExecute<'a> {
    /// Operations before the terminal `run` keyword.
    pub operations: Vec<CollectedExecuteOperation<'a>>,
    /// Original executable tail after `run`, preserving its exact spelling.
    pub command: &'a str,
}

/// Inspect an execute chain using the same grammar as collected validation.
///
/// Unknown or incomplete operation grammar returns `None`. An argument named
/// `run` is never treated as an operation boundary. Callers must retain unknown
/// work explicitly rather than interpreting failure to parse as zero work.
#[doc(hidden)]
pub fn collected_execute(line: &str) -> Option<CollectedExecute<'_>> {
    let tokens = tokenize_top_level(line)?;
    let run = execute_run_index(&tokens)?;
    let command = &line[tokens.get(run + 1)?.start..];
    let mut index = 1;
    let mut operations = Vec::new();
    while index < run {
        let width = execute_operation_width(&tokens, index)?;
        operations.push(CollectedExecuteOperation {
            keyword: tokens[index].text,
            arguments: tokens[index + 1..index + width]
                .iter()
                .map(|token| token.text)
                .collect(),
        });
        index += width;
    }
    Some(CollectedExecute {
        operations,
        command,
    })
}

/// Locate a collected `return run` command's executable tail.
#[doc(hidden)]
pub fn collected_return_command(line: &str) -> Option<&str> {
    let tokens = tokenize_top_level(line)?;
    (token_is(&tokens, 0, "return") && token_is(&tokens, 1, "run"))
        .then(|| tokens.get(2).map(|token| &line[token.start..]))
        .flatten()
}

/// Recognize operation boundaries, not arbitrary tokens named `run`: holders,
/// objectives, NBT paths, and other arguments may legitimately have that name.
fn execute_run_index(tokens: &[TopLevelToken<'_>]) -> Option<usize> {
    if !token_is(tokens, 0, "execute") {
        return None;
    }
    let mut index = 1;
    while index < tokens.len() {
        if token_is(tokens, index, "run") {
            return Some(index);
        }
        let width = execute_operation_width(tokens, index)?;
        index += width;
    }
    None
}

/// Count a complete operation using the shared conservative execute grammar.
fn execute_operation_width(tokens: &[TopLevelToken<'_>], index: usize) -> Option<usize> {
    let arg = |offset: usize| tokens.get(index + offset).map(|token| token.text);
    let width = match arg(0)? {
        "as" | "at" | "align" | "anchored" | "in" | "on" | "summon" => 2,
        "positioned" => match arg(1)? {
            "as" | "over" => 3,
            _ => 4,
        },
        "rotated" => 3,
        "facing" => 4,
        "if" | "unless" => match arg(1)? {
            "entity" | "predicate" | "dimension" | "function" => 3,
            "stopwatch" => 4,
            "loaded" => 5,
            "block" | "biome" => 6,
            "blocks" => 12,
            "score" => match arg(4)? {
                "matches" => 6,
                "=" | "<" | ">" | "<=" | ">=" => 7,
                _ => return None,
            },
            "data" => match arg(2)? {
                "entity" | "storage" => 5,
                "block" => 7,
                _ => return None,
            },
            "items" => match arg(2)? {
                "entity" => 6,
                "block" => 8,
                _ => return None,
            },
            _ => return None,
        },
        "store" if matches!(arg(1)?, "result" | "success") => match arg(2)? {
            "score" | "bossbar" => 5,
            "entity" | "storage" => 7,
            "block" => 9,
            _ => return None,
        },
        _ => return None,
    };
    (index + width <= tokens.len()).then_some(width)
}

fn validate_execute_command(
    line: &str,
    tokens: &[TopLevelToken<'_>],
    profile: &CommandProfile,
) -> CommandResult<()> {
    let run_index = execute_run_index(tokens);
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
            // Stop argument validation where this pass cannot model it;
            // execute_run_index independently recognizes operation boundaries.
            // Typed Execute retains structural validation.
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
    crate::ObjectiveName::try_dynamic(value.to_owned())
        .map(|_| ())
        .map_err(|error| CommandError::new("scoreboard", field, error.to_string()))
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
    if limit.is_some_and(|limit| limit != 1)
        || matches!(holder, "@a" | "@e")
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

pub(crate) fn validate_selector_token(token: &str) -> CommandResult<Option<&str>> {
    let bytes = token.as_bytes();
    if bytes.len() < 2
        || bytes[0] != b'@'
        || !matches!(bytes[1], b'a' | b'e' | b'p' | b's' | b'r' | b'n')
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
    if arguments.is_empty() {
        return Err(CommandError::new(
            "Selector",
            "arguments",
            "selector argument list must not be empty",
        ));
    }
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
        if matches!(key, "type" | "predicate") {
            let id = value.strip_prefix('!').unwrap_or(value);
            let id = id.strip_prefix('#').unwrap_or(id);
            let field = if key == "type" { "type" } else { "predicate" };
            crate::validate::resource_location_shape(id, "Selector", field)?;
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::Validate",
    aliases = ["sand::cmd::Validate", "sand::prelude::Validate", "sand::prelude::cmd::Validate"],
    module = "sand::command",
    summary = "Validate a typed command value against the active Minecraft profile.",
    context = "Validate a typed command value against the active Minecraft profile. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::Validate;",
)]
/// Validate a typed command value against the active Minecraft profile.
pub trait Validate {
    /// Reject invalid state before it can become command text.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Validate::validate",
        aliases = ["sand::cmd::Validate::validate", "sand::prelude::Validate::validate", "sand::prelude::cmd::Validate::validate"],
        module = "sand::command",
        summary = "Reject invalid state before it can become command text.",
        context = "Reject invalid state before it can become command text. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(profile = "`profile` is used to reject invalid state before it can become command text."),
        returns = "On success, the value produced to reject invalid state before it can become command text; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::command::Validate>(validate_value: &T, profile: & sand::command::CommandProfile)  {\n    let validate = validate_value.validate(profile);\n}",
    )]
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()>;
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::RenderCommand",
    aliases = ["sand::cmd::RenderCommand", "sand::prelude::RenderCommand", "sand::prelude::cmd::RenderCommand"],
    module = "sand::command",
    summary = "Deterministically render a validated Minecraft command or command fragment.",
    context = "Deterministically render a validated Minecraft command or command fragment. [`render`](Self::render) is the normal path. `render_unchecked` exists so compatibility `Display`/`Build` implementations can retain their historical output while exporters and new APIs use the fallible boundary.",
    minecraft = "[`render`](Self::render) is the normal path. `render_unchecked` exists so compatibility `Display`/`Build` implementations can retain their historical output while exporters and new APIs use the fallible boundary.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::RenderCommand;",
)]
/// Deterministically render a validated Minecraft command or command fragment.
///
/// [`render`](Self::render) is the normal path. `render_unchecked` exists so
/// compatibility `Display`/`Build` implementations can retain their historical
/// output while exporters and new APIs use the fallible boundary.
pub trait RenderCommand: Validate {
    /// Render without validation. Normal callers should use [`render`](Self::render).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::RenderCommand::render_unchecked",
        aliases = ["sand::cmd::RenderCommand::render_unchecked", "sand::prelude::RenderCommand::render_unchecked", "sand::prelude::cmd::RenderCommand::render_unchecked"],
        module = "sand::command",
        summary = "Render without validation. Normal callers should use [`render`](Self::render).",
        context = "Render without validation. Normal callers should use [`render`](Self::render). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(profile = "`profile` provides the profile rendered when without validation. Normal callers should use [`render`](Self::render)."),
        returns = "The string value produced to render without validation. Normal callers should use [`render`](Self::render).",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::command::RenderCommand>(render_command_value: &T, profile: & sand::command::CommandProfile)  {\n    let render_unchecked = render_command_value.render_unchecked(profile);\n}",
    )]
    fn render_unchecked(&self, profile: &CommandProfile) -> String;

    /// Validate, then render with the active Minecraft profile.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::RenderCommand::render", kind = "trait_method",
        aliases = ["sand::cmd::RenderCommand::render", "sand::prelude::RenderCommand::render", "sand::prelude::cmd::RenderCommand::render"],
        module = "sand::command",
        summary = "Validate, then render with the active Minecraft profile.",
        context = "Validate, then render with the active Minecraft profile. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(profile = "`profile` is used to validate, then render with the active Minecraft profile."),
        returns = "On success, the value produced to validate, then render with the active Minecraft profile; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::command::RenderCommand>(render_command_value: &T, profile: & sand::command::CommandProfile)  {\n    let render = render_command_value.render(profile);\n}",
    )]
    fn render(&self, profile: &CommandProfile) -> CommandResult<String> {
        self.validate(profile)?;
        Ok(self.render_unchecked(profile))
    }

    /// Validate and render using the unprofiled compatibility target.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::RenderCommand::try_build", kind = "trait_method",
        aliases = ["sand::cmd::RenderCommand::try_build", "sand::prelude::RenderCommand::try_build", "sand::prelude::cmd::RenderCommand::try_build"],
        module = "sand::command",
        summary = "Validate and render using the unprofiled compatibility target.",
        context = "Validate and render using the unprofiled compatibility target. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "On success, the value produced to validate and render using the unprofiled compatibility target; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::command::RenderCommand>(render_command_value: &T)  {\n    let try_build = render_command_value.try_build();\n}",
    )]
    fn try_build(&self) -> CommandResult<String> {
        self.render(&CommandProfile::unprofiled())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collected_validation_preserves_literal_raw_and_modded_content() {
        let profile = CommandProfile::new("26.2", false);
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
    fn collected_objectives_use_the_canonical_exact_name_boundary() {
        let profile = CommandProfile::new("26.2", false);
        for objective in ["__sd_nyelfbnatvtz.c".to_owned(), "x".repeat(32_767)] {
            for line in [
                format!("scoreboard objectives add {objective} dummy"),
                format!("execute if score @s {objective} matches 1 run say valid"),
                format!("scoreboard players operation @s {objective} = @s {objective}"),
            ] {
                assert_eq!(validate_collected_line(&line, &profile).unwrap(), line);
            }
        }
        for objective in ["x".repeat(32_768), "bad:name".to_owned()] {
            assert!(
                validate_collected_line(
                    &format!("scoreboard objectives add {objective} dummy"),
                    &profile
                )
                .is_err()
            );
        }
    }

    #[test]
    fn collected_validation_rejects_confidently_recognized_malformed_output() {
        let profile = CommandProfile::new("26.2", false);
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

#[cfg(test)]
mod nearest_entity_tests {
    use super::*;

    #[test]
    fn collected_numeric_sources_honor_nearest_entity_and_explicit_limits() {
        let profile = CommandProfile::unprofiled();
        for holder in [
            "@n",
            "@n[type=minecraft:zombie]",
            "@p[limit=1]",
            "@r[limit=1]",
        ] {
            validate_collected_line(
                &format!("execute if score {holder} value matches 1 run say yes"),
                &profile,
            )
            .unwrap();
        }
        for holder in ["@n[limit=2]", "@p[limit=2]", "@r[limit=2]"] {
            assert!(
                validate_collected_line(
                    &format!("execute if score {holder} value matches 1 run say yes"),
                    &profile
                )
                .is_err()
            );
        }
    }
}

#[cfg(test)]
mod execution_inspection_tests {
    use super::*;

    /// Preserve predicate calls and independent entity-selection operations.
    #[test]
    fn execution_view_keeps_operations_in_order() {
        let parsed = collected_execute("execute as @e[tag=actor] if function demo:ready at @e[tag=target] run function demo:act").unwrap();
        assert_eq!(parsed.command, "function demo:act");
        assert_eq!(
            parsed.operations,
            vec![
                CollectedExecuteOperation {
                    keyword: "as",
                    arguments: vec!["@e[tag=actor]"]
                },
                CollectedExecuteOperation {
                    keyword: "if",
                    arguments: vec!["function", "demo:ready"]
                },
                CollectedExecuteOperation {
                    keyword: "at",
                    arguments: vec!["@e[tag=target]"]
                },
            ]
        );
    }

    /// Quoted data and score identifiers named run remain opaque arguments.
    #[test]
    fn execution_view_does_not_guess_from_argument_keywords() {
        let line = r#"execute if score run run matches 1 if data storage demo:state values[{text:"run now"}] run say run"#;
        let parsed = collected_execute(line).unwrap();
        assert_eq!(parsed.command, "say run");
        assert_eq!(
            parsed.operations[0].arguments,
            ["score", "run", "run", "matches", "1"]
        );
        assert_eq!(
            parsed.operations[1].arguments,
            [
                "data",
                "storage",
                "demo:state",
                r#"values[{text:"run now"}]"#
            ]
        );
        assert_eq!(collected_execute_command(line), Some(parsed.command));
    }

    /// Unknown and incomplete commands cannot yield a misleading cost model.
    #[test]
    fn execution_view_rejects_unknown_or_incomplete_grammar() {
        for line in [
            "say run",
            "execute custom run say hello",
            "execute as",
            "execute as @s run",
            "execute if score @s value matches",
            "execute if data storage demo:state { run say nope",
        ] {
            assert!(collected_execute(line).is_none(), "{line}");
        }
    }
}
