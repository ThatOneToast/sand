//! Temporary-scoreboard lowering phase of the export pipeline.
//!
//! Owns the validated lowering of [`crate::TempScoreboard`] declarations
//! (registered through the [`temp_score!`](crate::temp_score) macro) into the
//! generated `__sand_temp_scores` function.
//!
//! # Validation
//! Before this phase existed, `temp_score!` metadata reached
//! `scoreboard objectives add …` completely unchecked (see
//! [#146](https://github.com/ThatOneToast/sand/issues/146)). Three classes of
//! invalid output were reachable:
//!
//! - objective names longer than Minecraft's 16-character limit, or carrying
//!   characters an objective name cannot use;
//! - criterion tokens containing whitespace or control characters, which would
//!   silently shift the rest of the command into the wrong argument position;
//! - display names interpolated as bare text, even though vanilla requires a
//!   JSON text component in that position.
//!
//! Objective names now route through the canonical
//! [`sand_commands::ObjectiveName`] rules — the same deterministic long-name
//! hashing used by `ScoreVar`, `Flag`, `Timer`, and `Cooldown` — so the whole
//! state family agrees on how a logical name becomes an emitted one. Display
//! names are rendered through the canonical [`sand_commands::TextComponent`]
//! model rather than a second text representation.
#![allow(clippy::result_large_err)]

use std::collections::BTreeSet;

use super::records::ExportResult;
use crate::component::ComponentExportError;

/// Characters a scoreboard criterion may use, e.g. `dummy`, `health`,
/// `playerKillCount`, `minecraft.mined:minecraft.stone`, `teamkill.blue`.
fn is_criterion_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | ':' | '-')
}

/// One temporary-scoreboard declaration, decoupled from `inventory` so the
/// lowering rules can be tested directly without process-global registration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TempScoreEntry<'a> {
    /// The logical objective name as written in `temp_score!`.
    pub(crate) name: &'a str,
    /// The scoreboard criterion token.
    pub(crate) criteria: &'a str,
    /// Optional display name, rendered as a JSON text component.
    pub(crate) display_name: Option<&'a str>,
}

/// Diagnostic error for the temporary-scoreboard phase of the export pipeline.
pub(crate) fn temp_score_export_error(
    field: impl Into<String>,
    message: impl Into<String>,
) -> ComponentExportError {
    ComponentExportError::ComponentValidation {
        location: sand_components::ResourceLocation::new("sand", "temp_scores")
            .expect("fixed temp score resource location is valid"),
        kind: "temp_score".to_string(),
        field: field.into(),
        message: message.into(),
    }
}

/// Resolve a `temp_score!` name to the objective name actually emitted.
///
/// Empty, whitespace-bearing, and control-character names are rejected
/// outright rather than being silently hashed into a valid-looking objective.
/// Everything else goes through [`sand_commands::ObjectiveName::logical`], so a
/// name that is too long for Minecraft's 16-character limit is deterministically
/// hashed exactly the way `ScoreVar`/`Flag`/`Timer`/`Cooldown` hash theirs.
fn emitted_objective_name(name: &str) -> ExportResult<String> {
    if name.is_empty() {
        return Err(temp_score_export_error(
            "name",
            "temp score objective name must not be empty",
        ));
    }
    if name.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(temp_score_export_error(
            "name",
            format!(
                "temp score objective name `{}` must not contain whitespace or control characters",
                name.escape_debug()
            ),
        ));
    }
    Ok(sand_commands::ObjectiveName::logical(name)
        .as_str()
        .to_string())
}

/// Validate a scoreboard criterion token.
fn validated_criterion(name: &str, criteria: &str) -> ExportResult<()> {
    if criteria.is_empty() {
        return Err(temp_score_export_error(
            "criteria",
            format!("temp score `{name}` has an empty criterion"),
        ));
    }
    if let Some(bad) = criteria.chars().find(|c| !is_criterion_char(*c)) {
        return Err(temp_score_export_error(
            "criteria",
            format!(
                "temp score `{name}` has criterion `{}` containing invalid character `{}`; \
                 criteria may only use [A-Za-z0-9_.:-] (e.g. `dummy`, `health`, \
                 `minecraft.mined:minecraft.stone`)",
                criteria.escape_debug(),
                bad.escape_debug()
            ),
        ));
    }
    Ok(())
}

/// Render an optional display name as a JSON text component.
///
/// Vanilla requires a text component in the `<displayName>` position of
/// `scoreboard objectives add`. The raw string that `temp_score!` accepts is
/// wrapped in the canonical [`sand_commands::TextComponent`] literal form so
/// the emitted command is well-formed and correctly escaped, including for
/// text containing spaces or quotes. Control characters are rejected.
fn rendered_display_name(name: &str, display: &str) -> ExportResult<String> {
    if let Some(bad) = display.chars().find(|c| c.is_control()) {
        return Err(temp_score_export_error(
            "display_name",
            format!(
                "temp score `{name}` has display name `{}` containing control character `{}`",
                display.escape_debug(),
                bad.escape_debug()
            ),
        ));
    }
    Ok(sand_commands::TextComponent::literal(display).to_string())
}

/// Lower a set of temporary-scoreboard declarations into validated
/// `scoreboard objectives add` commands.
///
/// Declarations are de-duplicated on `(name, criteria)` and emitted in the
/// caller's iteration order, matching the behavior this phase replaced. Every
/// entry is validated before **any** command is returned, so a single invalid
/// declaration fails the export rather than contributing partial output.
pub(crate) fn lower_temp_scores<'a, I>(entries: I) -> ExportResult<Vec<String>>
where
    I: IntoIterator<Item = TempScoreEntry<'a>>,
{
    let mut seen: BTreeSet<(&str, &str)> = BTreeSet::new();
    let mut cmds: Vec<String> = Vec::new();

    for entry in entries {
        if !seen.insert((entry.name, entry.criteria)) {
            continue;
        }
        let objective = emitted_objective_name(entry.name)?;
        validated_criterion(entry.name, entry.criteria)?;

        match entry.display_name {
            Some(display) => {
                let rendered = rendered_display_name(entry.name, display)?;
                cmds.push(format!(
                    "scoreboard objectives add {objective} {} {rendered}",
                    entry.criteria
                ));
            }
            None => cmds.push(format!(
                "scoreboard objectives add {objective} {}",
                entry.criteria
            )),
        }
    }

    Ok(cmds)
}

/// Lower every inventory-registered [`crate::TempScoreboard`] declaration.
pub(crate) fn temp_score_commands() -> ExportResult<Vec<String>> {
    lower_temp_scores(
        inventory::iter::<crate::TempScoreboard>().map(|ts| TempScoreEntry {
            name: ts.name,
            criteria: ts.criteria,
            display_name: ts.display_name,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry<'a>(name: &'a str, criteria: &'a str) -> TempScoreEntry<'a> {
        TempScoreEntry {
            name,
            criteria,
            display_name: None,
        }
    }

    #[test]
    fn valid_objective_and_criterion_render_the_vanilla_command() {
        let cmds = lower_temp_scores([entry("player_hp_tmp", "dummy")]).unwrap();
        assert_eq!(cmds, vec!["scoreboard objectives add player_hp_tmp dummy"]);
    }

    #[test]
    fn compound_vanilla_criteria_are_accepted() {
        let cmds = lower_temp_scores([entry("mined", "minecraft.mined:minecraft.stone")]).unwrap();
        assert_eq!(
            cmds,
            vec!["scoreboard objectives add mined minecraft.mined:minecraft.stone"]
        );
    }

    #[test]
    fn long_objective_names_are_hashed_by_the_canonical_rules() {
        let long = "an_extremely_long_temp_objective_name";
        let cmds = lower_temp_scores([entry(long, "dummy")]).unwrap();
        let expected = sand_commands::ObjectiveName::logical(long)
            .as_str()
            .to_string();

        assert!(expected.len() <= 16, "hashed name must fit: {expected}");
        assert_eq!(
            cmds,
            vec![format!("scoreboard objectives add {expected} dummy")]
        );
    }

    #[test]
    fn declaration_order_and_dedup_are_deterministic_across_repeats() {
        let declare = || {
            lower_temp_scores([
                entry("beta", "dummy"),
                entry("alpha", "health"),
                entry("beta", "dummy"),
            ])
            .unwrap()
        };
        let first = declare();
        assert_eq!(
            first,
            vec![
                "scoreboard objectives add beta dummy",
                "scoreboard objectives add alpha health",
            ],
            "dedup must keep declaration order, not sort"
        );
        for _ in 0..64 {
            assert_eq!(declare(), first);
        }
    }

    #[test]
    fn empty_objective_name_is_rejected() {
        let err = lower_temp_scores([entry("", "dummy")]).unwrap_err();
        assert!(err.to_string().contains("must not be empty"), "got: {err}");
    }

    #[test]
    fn whitespace_objective_name_is_rejected_not_hashed() {
        let err = lower_temp_scores([entry("my objective", "dummy")]).unwrap_err();
        assert!(
            err.to_string().contains("whitespace or control"),
            "got: {err}"
        );
    }

    #[test]
    fn control_character_objective_name_is_rejected() {
        let err = lower_temp_scores([entry("bad\u{7}name", "dummy")]).unwrap_err();
        assert!(
            err.to_string().contains("whitespace or control"),
            "got: {err}"
        );
    }

    #[test]
    fn empty_criterion_is_rejected() {
        let err = lower_temp_scores([entry("obj", "")]).unwrap_err();
        assert!(err.to_string().contains("empty criterion"), "got: {err}");
    }

    #[test]
    fn malformed_criterion_is_rejected() {
        let err = lower_temp_scores([entry("obj", "dummy extra")]).unwrap_err();
        assert!(err.to_string().contains("invalid character"), "got: {err}");
    }

    #[test]
    fn control_character_criterion_is_rejected() {
        let err = lower_temp_scores([entry("obj", "dum\u{0}my")]).unwrap_err();
        assert!(err.to_string().contains("invalid character"), "got: {err}");
    }

    #[test]
    fn safe_display_text_renders_as_a_json_text_component() {
        let cmds = lower_temp_scores([TempScoreEntry {
            name: "kills",
            criteria: "dummy",
            display_name: Some("Total Kills"),
        }])
        .unwrap();
        assert_eq!(
            cmds,
            vec![r#"scoreboard objectives add kills dummy {"text":"Total Kills"}"#],
            "a display name must be a JSON text component, not bare text"
        );
    }

    #[test]
    fn display_text_with_quotes_is_escaped_rather_than_breaking_the_command() {
        let cmds = lower_temp_scores([TempScoreEntry {
            name: "kills",
            criteria: "dummy",
            display_name: Some(r#"The "Best" Score"#),
        }])
        .unwrap();
        assert_eq!(
            cmds,
            vec![r#"scoreboard objectives add kills dummy {"text":"The \"Best\" Score"}"#]
        );
    }

    #[test]
    fn unsafe_control_character_display_text_is_rejected() {
        let err = lower_temp_scores([TempScoreEntry {
            name: "kills",
            criteria: "dummy",
            display_name: Some("bad\u{1b}[0m"),
        }])
        .unwrap_err();
        assert!(err.to_string().contains("control character"), "got: {err}");
    }

    #[test]
    fn invalid_declaration_yields_no_commands_at_all() {
        let result = lower_temp_scores([entry("good_one", "dummy"), entry("bad one", "dummy")]);
        assert!(
            result.is_err(),
            "an invalid declaration must fail the whole lowering"
        );
    }

    #[test]
    fn diagnostic_identifies_the_temp_score_phase() {
        let err = lower_temp_scores([entry("obj", "")]).unwrap_err();
        let rendered = err.to_string();
        assert!(rendered.contains("sand:temp_scores"), "got: {rendered}");
        assert!(rendered.contains("temp_score"), "got: {rendered}");
    }
}
