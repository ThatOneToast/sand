//! Lifecycle and transition validation phase of the export pipeline.
//!
//! Owns the collision checks that keep Sand-generated private lifecycle and
//! transition function paths from overwriting user or component functions,
//! and the diagnostic error constructors for that phase.
#![allow(clippy::result_large_err)]

use super::records::{ComponentRecord, ExportResult};
use crate::component::ComponentExportError;

pub(crate) fn ensure_private_lifecycle_path_available(
    records: &[ComponentRecord],
    path: &str,
) -> ExportResult<()> {
    if records
        .iter()
        .any(|record| record.dir == "function" && record.path == path)
    {
        return Err(lifecycle_export_error(format!(
            "generated private function `{path}` collides with a user or component function"
        )));
    }
    Ok(())
}

pub(crate) fn lifecycle_export_error(message: impl Into<String>) -> ComponentExportError {
    ComponentExportError::ComponentValidation {
        location: sand_components::ResourceLocation::new("sand", "lifecycle")
            .expect("fixed lifecycle resource location is valid"),
        kind: "state_lifecycle".to_string(),
        field: "declarations".to_string(),
        message: message.into(),
    }
}

pub(crate) fn transition_export_error(message: impl Into<String>) -> ComponentExportError {
    ComponentExportError::ComponentValidation {
        location: sand_components::ResourceLocation::new("sand", "transitions")
            .expect("fixed transition resource location is valid"),
        kind: "tracked_transition".to_string(),
        field: "trackers".to_string(),
        message: message.into(),
    }
}

pub(crate) fn ensure_private_transition_path_available(
    records: &[ComponentRecord],
    path: &str,
    tracker_id: &str,
    source: &str,
) -> ExportResult<()> {
    if records
        .iter()
        .any(|record| record.dir == "function" && record.path == path)
    {
        return Err(transition_export_error(format!(
            "tracker `{tracker_id}` source `{source}` generated private function `{path}`, which collides with a user or component function"
        )));
    }
    Ok(())
}

/// Assemble owned lifecycle contributions after frontend collection.
pub(crate) fn assemble_lifecycle(
    namespace: &str,
    records: &mut Vec<ComponentRecord>,
    tag_map: &mut std::collections::BTreeMap<String, Vec<String>>,
    mut automatic: crate::state::registry::AutomaticLifecycle,
    mut registration_load_commands: Vec<(String, usize, String)>,
    mut registration_tick_commands: Vec<(String, usize, String)>,
    transition_global_tick_commands: Vec<String>,
) -> ExportResult<()> {
    use std::collections::BTreeMap;
    registration_load_commands
        .sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    registration_tick_commands
        .sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));

    let mut load_definitions: BTreeMap<String, (String, String, String)> = BTreeMap::new();
    for command in automatic.load_commands {
        let mut parts = command.splitn(6, ' ');
        let parsed = match (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) {
            (
                Some("scoreboard"),
                Some("objectives"),
                Some("add"),
                Some(objective),
                Some(criterion),
                display_name,
            ) if display_name
                .is_none_or(|json| serde_json::from_str::<serde_json::Value>(json).is_ok()) =>
            {
                Some((objective.to_string(), criterion.to_string()))
            }
            _ => None,
        };

        if let Some((objective, criterion)) = parsed {
            match load_definitions.get(&objective) {
                Some((existing, _, _)) if existing == &criterion => {}
                Some((existing, _, existing_owner)) => {
                    return Err(lifecycle_export_error(format!(
                        "conflicting objective `{objective}`: `{existing_owner}` declares criterion `{existing}`, while automatic state lifecycle declares `{criterion}`"
                    )));
                }
                None => {
                    load_definitions.insert(
                        objective,
                        (criterion, command, "automatic state lifecycle".to_string()),
                    );
                }
            }
        } else {
            return Err(lifecycle_export_error(format!(
                "invalid registered load command `{command}`"
            )));
        }
    }
    let mut registration_definitions: BTreeMap<String, (String, String)> = load_definitions
        .iter()
        .map(|(objective, (criterion, _, owner))| {
            (objective.clone(), (criterion.clone(), owner.clone()))
        })
        .collect();
    let mut load_cmds: Vec<String> = load_definitions
        .into_values()
        .map(|(_, command, _)| command)
        .collect();
    for (owner, _, command) in registration_load_commands {
        let mut parts = command.splitn(6, ' ');
        let parsed = match (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) {
            (
                Some("scoreboard"),
                Some("objectives"),
                Some("add"),
                Some(objective),
                Some(criterion),
                None,
            ) => Some((objective.to_string(), criterion.to_string())),
            _ => None,
        };

        if let Some((objective, criterion)) = parsed {
            match registration_definitions.get(&objective) {
                Some((existing, _)) if existing == &criterion => {}
                Some((existing, existing_owner)) => {
                    return Err(lifecycle_export_error(format!(
                        "conflicting objective `{objective}`: `{existing_owner}` declares criterion `{existing}`, while `{owner}` declares `{criterion}`"
                    )));
                }
                None => {
                    registration_definitions.insert(objective, (criterion, owner));
                }
            }
        }
        load_cmds.push(command);
    }
    load_cmds.append(&mut automatic.provision_commands);
    load_cmds.append(&mut automatic.global_init_commands);
    if !load_cmds.is_empty() {
        let path = "__sand_lifecycle_load";
        ensure_private_lifecycle_path_available(records, path)?;
        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: path.to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: load_cmds.join("\n"),
        });
        tag_map
            .entry("minecraft:load".to_string())
            .or_default()
            .push(format!("{namespace}:{path}"));
    }

    let init_path = "__sand_lifecycle_init";
    if !automatic.player_init_commands.is_empty() {
        ensure_private_lifecycle_path_available(records, init_path)?;
        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: init_path.to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: automatic.player_init_commands.join("\n"),
        });
    }

    let mut tick_cmds = Vec::new();
    if !automatic.player_init_commands.is_empty() {
        tick_cmds.push(format!(
            "execute as @a run function {namespace}:{init_path}"
        ));
    }
    tick_cmds.extend(
        automatic
            .player_tick_commands
            .into_iter()
            .map(|command| format!("execute as @a run {command}")),
    );
    tick_cmds.extend(automatic.entity_tick_commands);
    tick_cmds.extend(automatic.global_tick_commands);
    tick_cmds.extend(transition_global_tick_commands);
    tick_cmds.extend(
        registration_tick_commands
            .into_iter()
            .map(|(_, _, command)| command),
    );
    if !tick_cmds.is_empty() {
        let path = "__sand_lifecycle_tick";
        ensure_private_lifecycle_path_available(records, path)?;
        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: path.to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: tick_cmds.join("\n"),
        });
        tag_map
            .entry("minecraft:tick".to_string())
            .or_default()
            .push(format!("{namespace}:{path}"));
    }
    Ok(())
}

/// Initialize canonical player State at declared player-context entry points.
pub(crate) fn initialize_player_entries(
    records: &mut [ComponentRecord],
    entries: &std::collections::BTreeMap<String, crate::compiler::program::model::ExecutionContext>,
    namespace: &str,
) {
    if !records.iter().any(|record| {
        record.namespace == namespace
            && record.dir == "function"
            && record.path == "__sand_lifecycle_init"
    }) {
        return;
    }
    for record in records.iter_mut().filter(|record| record.dir == "function") {
        if entries.get(&format!("{}:{}", record.namespace, record.path))
            == Some(&crate::compiler::program::model::ExecutionContext::Player)
        {
            let prefix = format!("function {namespace}:__sand_lifecycle_init");
            record.content = if record.content.is_empty() {
                prefix
            } else {
                format!("{prefix}\n{}", record.content)
            };
        }
    }
}
