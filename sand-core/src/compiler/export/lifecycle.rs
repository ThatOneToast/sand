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

/// Lifecycle output retained by both registered and portable frontends.
pub(crate) struct LifecycleAssembly {
    /// All automatic and explicit owners, including work without command spans.
    load_contributors: std::collections::BTreeSet<String>,
    init_contributors: std::collections::BTreeSet<String>,
    tick_contributors: std::collections::BTreeSet<String>,
    /// Initializer that player-context entry functions must invoke.
    pub(crate) player_initializer: Option<String>,
    /// Load-command spans and their original registration owners.
    pub(crate) load_owners: Vec<crate::compiler::runtime::CommandOrigin>,
    /// Tick-command spans and their original registration owners.
    pub(crate) tick_owners: Vec<crate::compiler::runtime::CommandOrigin>,
}

impl LifecycleAssembly {
    /// Apply phase-specific provenance identically for Rust and portable frontends.
    pub(crate) fn attribute(
        &self,
        namespace: &str,
        runtime: &mut crate::compiler::runtime::RuntimePlan,
    ) {
        for (phase, owners) in [
            ("load", &self.load_contributors),
            ("tick", &self.tick_contributors),
        ] {
            runtime.own_lifecycle(
                format!("{namespace}:__sand_lifecycle_{phase}"),
                phase,
                owners.iter().cloned(),
            );
        }
        if let Some(initializer) = &self.player_initializer {
            for owner in &self.init_contributors {
                runtime.own_function(initializer.clone(), owner.clone());
            }
        }
    }
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
) -> ExportResult<LifecycleAssembly> {
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
    let mut load_owners = Vec::new();
    let mut load_first_line = 1 + load_cmds
        .iter()
        .map(|command| command.bytes().filter(|byte| *byte == b'\n').count() + 1)
        .sum::<usize>();
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
                    registration_definitions.insert(objective, (criterion, owner.clone()));
                }
            }
        }
        append_owned_command(
            &mut load_cmds,
            &mut load_owners,
            owner,
            command,
            &mut load_first_line,
        );
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
    let mut tick_owners = Vec::new();
    let mut first_line = 1 + tick_cmds
        .iter()
        .map(|command| command.bytes().filter(|byte| *byte == b'\n').count() + 1)
        .sum::<usize>();
    for (owner, _, command) in registration_tick_commands {
        append_owned_command(
            &mut tick_cmds,
            &mut tick_owners,
            owner,
            command,
            &mut first_line,
        );
    }
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
    let mut load_contributors = automatic.load_origins;
    load_contributors.extend(load_owners.iter().map(|origin| origin.owner.clone()));
    let mut tick_contributors = automatic.tick_origins;
    tick_contributors.extend(tick_owners.iter().map(|origin| origin.owner.clone()));
    Ok(LifecycleAssembly {
        load_contributors,
        init_contributors: automatic.init_origins,
        tick_contributors,
        player_initializer: (!automatic.player_init_commands.is_empty())
            .then(|| format!("{namespace}:{init_path}")),
        load_owners,
        tick_owners,
    })
}

/// Retain the original owner at the exact emitted line span for either phase.
fn append_owned_command(
    commands: &mut Vec<String>,
    origins: &mut Vec<crate::compiler::runtime::CommandOrigin>,
    owner: String,
    command: String,
    first_line: &mut usize,
) {
    let line_count = command.lines().count();
    if line_count != 0 {
        origins.push(crate::compiler::runtime::CommandOrigin {
            owner,
            first_line: *first_line,
            line_count,
        });
    }
    *first_line += command.bytes().filter(|byte| *byte == b'\n').count() + 1;
    commands.push(command);
}

/// Initialize canonical player State at declared player-context entry points.
pub(crate) fn initialize_player_entries(
    records: &mut [ComponentRecord],
    entries: &std::collections::BTreeMap<String, crate::compiler::program::model::ExecutionContext>,
    initializer: Option<&str>,
) {
    let Some(initializer) = initializer else {
        return;
    };
    for record in records.iter_mut().filter(|record| record.dir == "function") {
        if entries.get(&format!("{}:{}", record.namespace, record.path))
            == Some(&crate::compiler::program::model::ExecutionContext::Player)
        {
            let prefix = format!("function {initializer}");
            record.content = if record.content.is_empty() {
                prefix
            } else {
                format!("{prefix}\n{}", record.content)
            };
        }
    }
}

/// Server lifecycle tags must not directly invoke known player-only entries.
/// Resolve local references before checking so both registration and descriptor
/// contributions follow the same declared function context contract.
pub(crate) fn validate_server_lifecycle_tags(
    namespace: &str,
    tags: &std::collections::BTreeMap<String, Vec<String>>,
    entries: &[(String, String)],
    contexts: &std::collections::BTreeMap<
        String,
        crate::compiler::program::model::ExecutionContext,
    >,
) -> ExportResult<()> {
    for (tag, function) in tags
        .iter()
        .flat_map(|(tag, functions)| functions.iter().map(move |function| (tag, function)))
        .chain(entries.iter().map(|(tag, function)| (tag, function)))
    {
        let tag = super::functions::resolve_local_refs(tag, namespace);
        let function = super::functions::resolve_local_refs(function, namespace);
        if matches!(tag.as_str(), "minecraft:load" | "minecraft:tick")
            && contexts.get(&function)
                == Some(&crate::compiler::program::model::ExecutionContext::Player)
        {
            return Err(lifecycle_export_error(format!(
                "server lifecycle tag `{tag}` cannot invoke player-context function `{function}`; register a server-context wrapper that selects players"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod runtime_origin_tests {
    use super::*;
    use crate::compiler::runtime::CommandOrigin;
    use crate::state::registry::AutomaticLifecycle;
    use std::collections::BTreeMap;

    /// Ownership follows emitted lines after sorting, including multiline bodies.
    #[test]
    fn lifecycle_origins_match_emitted_function_spans() {
        let mut records = Vec::new();
        let mut tags = BTreeMap::new();
        let automatic = AutomaticLifecycle {
            load_commands: vec!["scoreboard objectives add auto dummy".into()],
            global_init_commands: vec!["say automatic init".into()],
            global_tick_commands: vec!["say automatic\nsay next".into()],
            ..AutomaticLifecycle::default()
        };
        let assembled = assemble_lifecycle(
            "audit",
            &mut records,
            &mut tags,
            automatic,
            vec![
                ("module:b".into(), 0, "say b load".into()),
                ("module:a".into(), 0, "say a load\n# load comment\n".into()),
            ],
            vec![
                ("module:b".into(), 0, "say b".into()),
                ("module:a".into(), 1, "say a2".into()),
                ("module:a".into(), 0, "say a1\n# retained comment\n".into()),
            ],
            Vec::new(),
        )
        .unwrap();
        let load = records
            .iter()
            .find(|record| record.path == "__sand_lifecycle_load")
            .unwrap();
        assert_eq!(
            load.content,
            "scoreboard objectives add auto dummy\nsay a load\n# load comment\n\nsay b load\nsay automatic init"
        );
        assert_eq!(
            assembled.load_owners,
            vec![
                CommandOrigin {
                    owner: "module:a".into(),
                    first_line: 2,
                    line_count: 2
                },
                CommandOrigin {
                    owner: "module:b".into(),
                    first_line: 5,
                    line_count: 1
                },
            ]
        );
        assert!(assembled.player_initializer.is_none());
        let tick = records
            .iter()
            .find(|record| record.path == "__sand_lifecycle_tick")
            .unwrap();
        assert_eq!(
            tick.content,
            "say automatic\nsay next\nsay a1\n# retained comment\n\nsay a2\nsay b"
        );
        assert_eq!(
            assembled.tick_owners,
            vec![
                CommandOrigin {
                    owner: "module:a".into(),
                    first_line: 3,
                    line_count: 2
                },
                CommandOrigin {
                    owner: "module:a".into(),
                    first_line: 6,
                    line_count: 1
                },
                CommandOrigin {
                    owner: "module:b".into(),
                    first_line: 7,
                    line_count: 1
                },
            ]
        );
        assert_eq!(tags["minecraft:tick"], ["audit:__sand_lifecycle_tick"]);
    }
}

#[cfg(test)]
mod context_tests {
    use super::*;
    use crate::compiler::program::model::ExecutionContext;
    use std::collections::BTreeMap;

    #[test]
    fn lifecycle_members_obey_context_after_local_resolution() {
        let contexts = BTreeMap::from([
            ("pack:player".into(), ExecutionContext::Player),
            ("pack:server".into(), ExecutionContext::Server),
        ]);
        for tag in ["minecraft:load", "minecraft:tick"] {
            for function in ["pack:player", "__sand_local:player"] {
                let entries = vec![(tag.into(), function.into())];
                assert!(
                    validate_server_lifecycle_tags("pack", &BTreeMap::new(), &entries, &contexts)
                        .is_err()
                );
                let tags = BTreeMap::from([(tag.into(), vec![function.into()])]);
                assert!(validate_server_lifecycle_tags("pack", &tags, &[], &contexts).is_err());
            }
            let entries = vec![
                (tag.into(), "pack:server".into()),
                (tag.into(), "external:unknown".into()),
            ];
            validate_server_lifecycle_tags("pack", &BTreeMap::new(), &entries, &contexts).unwrap();
        }
        validate_server_lifecycle_tags(
            "pack",
            &BTreeMap::new(),
            &[("pack:custom".into(), "pack:player".into())],
            &contexts,
        )
        .unwrap();
    }
}
