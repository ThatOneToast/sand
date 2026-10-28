//! Private, storage-scoped upgrade of retained owner records.
//!
//! The old list remains authoritative evidence until every row has been copied.
//! A persistent intent precedes each destination write; a completed journal is
//! committed on the next invocation. No executor or loaded entity is required.

use crate::function::{register_dyn_fn, register_dyn_fn_dedup};

pub(crate) fn ready(storage: &str) -> String {
    format!("if data storage {storage} __sand_keyed{{version:2}}")
}

/// Register one bounded migration step, shared by all components in a storage.
/// Returns (objective declaration, provisioning command, tick command).
pub(crate) fn lifecycle(storage: &str) -> (String, String, String) {
    let objective = sand_commands::ObjectiveName::logical(&format!("{storage}::keyed_migration"))
        .as_str()
        .to_owned();
    let root = format!("sand/state_data/migrate/{objective}");
    let call = |name: &str| format!("function __sand_local:{root}/{name}");
    let s = format!("storage {storage}");
    let m = "__sand_keyed";
    let j = "__sand_keyed.journal";
    let error = |reason: &str| -> String {
        let message = format!(
            "Sand keyed migration blocked in {storage}: {reason}. Inspect __sand_keyed.journal and owners at __sand_keyed.cursor; preserve a backup before repairing. Clear __sand_keyed.error and the uncommitted journal only after repair."
        );
        let path = register_dyn_fn_dedup(
            "sand/state_data/migration_error",
            vec![
                format!(
                    "data modify {s} {m}.error set value {}",
                    serde_json::to_string(reason).unwrap()
                ),
                "say Sand keyed migration blocked. Inspect __sand_keyed.error, cursor and journal in the affected storage. Back up before repair.".into(),
                format!("tellraw @a {}", serde_json::to_string(&message).unwrap()),
            ],
        );
        format!("function __sand_local:{path}")
    };
    let malformed = error("legacy row must contain a four-int UUID and compound components");
    let conflict = error("destination exists or differs; duplicate UUIDs are never merged");
    let invalid = error("unsupported version, malformed legacy container, or invalid cursor");
    let finish = vec![
        format!("data modify {s} {m}.finishing set value 1b"),
        format!("data remove {s} owners"),
        // A single replacement makes steady metadata constant and removes all
        // migration scratch. Retention is unchanged: every owner remains indexed.
        format!("data modify {s} {m} set value {{version:2}}"),
    ];
    register_dyn_fn(format!("{root}/finish"), finish);
    register_dyn_fn(
        format!("{root}/check_committed_cursor"),
        vec![
            format!("scoreboard players remove #cursor {objective} 1"),
            format!("execute unless score #next {objective} = #cursor {objective} run {invalid}"),
        ],
    );
    register_dyn_fn(
        format!("{root}/commit"),
        vec![
            format!("execute store result score #cursor {objective} run data get {s} {m}.cursor"),
            format!("execute store result score #next {objective} run data get {s} {j}.next"),
            format!(
                "execute store result {s} {m}.next_number int 1 run scoreboard players get #next {objective}"
            ),
            format!("data modify {s} {m}.next_check set from {s} {j}.next"),
            format!(
                "execute store success {s} {m}.bad_next int 1 run data modify {s} {m}.next_check set from {s} {m}.next_number"
            ),
            format!("execute if data {s} {m}{{bad_next:1}} run {invalid}"),
            format!("scoreboard players add #cursor {objective} 1"),
            format!(
                "execute unless score #next {objective} = #cursor {objective} run {}",
                call("check_committed_cursor")
            ),
            format!("execute if data {s} {m}.error run return fail"),
            format!("data modify {s} {m}.cursor set from {s} {j}.next"),
            format!("data remove {s} {j}"),
        ],
    );
    register_dyn_fn(
        format!("{root}/resume"),
        vec![
            format!("{} with {s} {m}", call("prepare")),
            format!("execute if data {s} {m}.error run return fail"),
            format!("execute if data {s} {j}{{done:1b}} run {}", call("commit")),
        ],
    );
    register_dyn_fn(
        format!("{root}/copy"),
        vec![
            format!(
                "$execute if data {s} owners_v2.\"$(uuid)\" unless data {s} {j}{{intent:1b}} run {conflict}"
            ),
            format!("execute if data {s} {m}.error run return fail"),
            format!("data modify {s} {j}.intent set value 1b"),
            format!(
                "$execute unless data {s} owners_v2.\"$(uuid)\" run data modify {s} owners_v2.\"$(uuid)\" set from {s} {j}.row"
            ),
            format!("$execute unless data {s} owners_v2.\"$(uuid)\" run {conflict}"),
            format!("execute if data {s} {m}.error run return fail"),
            format!("$data modify {s} {m}.compare set from {s} owners_v2.\"$(uuid)\""),
            format!(
                "execute store success storage {storage} {m}.different int 1 run data modify {s} {m}.compare set from {s} {j}.row"
            ),
            format!("execute if data {s} {m}{{different:1}} run {conflict}"),
            format!("execute if data {s} {m}.error run return fail"),
            format!("data modify {s} {j}.done set value 1b"),
        ],
    );
    register_dyn_fn(
        format!("{root}/validate"),
        vec![
            format!(
                "$execute unless data {s} {j}.row{{uuid:[I;$(u0),$(u1),$(u2),$(u3)]}} run {malformed}"
            ),
            format!("execute if data {s} {m}.error run return fail"),
            format!("{} with {s} {j}.row", call("copy")),
        ],
    );
    let mut prepare = vec![
        format!(
            "$execute unless data {s} {j}.row run data modify {s} {j}.row set from {s} owners[$(cursor)]"
        ),
        format!("execute unless data {s} {j}.row.components{{}} run {malformed}"),
        format!("execute if data {s} {m}.error run return fail"),
        format!("execute store result {s} {m}.length int 1 run data get {s} {j}.row.uuid"),
        format!("execute unless data {s} {m}{{length:4}} run {malformed}"),
        format!("execute if data {s} {m}.error run return fail"),
    ];
    for index in 0..4 {
        prepare.push(format!("execute store result {s} {m}.args.u{index} int 1 run data get {s} {j}.row.uuid[{index}]"));
    }
    prepare.extend([
        format!("execute store result score #cursor {objective} run data get {s} {m}.cursor"),
        format!("scoreboard players add #cursor {objective} 1"),
        format!(
            "execute unless data {s} {j}.next store result {s} {j}.next int 1 run scoreboard players get #cursor {objective}"
        ),
        format!("{} with {s} {m}.args", call("validate")),
    ]);
    register_dyn_fn(format!("{root}/prepare"), prepare);
    register_dyn_fn(
        format!("{root}/step"),
        vec![
            format!("execute if data {s} {m}{{version:2}} run return 0"),
            format!("execute if data {s} {m}.error run return fail"),
            format!("execute unless data {s} {m}{{version:1}} run {invalid}"),
            format!("execute if data {s} {m}.error run return fail"),
            format!(
                "execute if data {s} {m}{{finishing:1b}} run return run {}",
                call("finish")
            ),
            format!(
                "execute if data {s} {j}{{done:1b}} run return run {}",
                call("resume")
            ),
            format!("execute store result score #length {objective} run data get {s} owners"),
            format!("execute store result score #cursor {objective} run data get {s} {m}.cursor"),
            format!("execute unless score #cursor {objective} matches 0.. run {invalid}"),
            format!("execute if score #cursor {objective} > #length {objective} run {invalid}"),
            format!("execute if data {s} {m}.error run return fail"),
            format!(
                "execute if score #length {objective} matches 0 unless data {s} {{owners:[]}} run {invalid}"
            ),
            format!("execute if data {s} {m}.error run return fail"),
            format!(
                "execute if score #cursor {objective} = #length {objective} run return run {}",
                call("finish")
            ),
            format!("{} with {s} {m}", call("prepare")),
        ],
    );
    let provision = register_dyn_fn_dedup(
        "sand/state_data/migration_provision",
        vec![
            format!(
                "execute unless data {s} {m} if data {s} owners run data modify {s} {m} set value {{version:1,cursor:0}}"
            ),
            format!(
                "execute unless data {s} {m} unless data {s} owners run data modify {s} {m} set value {{version:2}}"
            ),
        ],
    );
    (
        format!("scoreboard objectives add {objective} dummy"),
        format!("function __sand_local:{provision}"),
        format!(
            "execute unless data {s} {m}{{version:2}} run {}",
            call("step")
        ),
    )
}

/// Gate the complete component transition, including its presence publication.
pub(crate) fn guard(
    fields: &[super::state::StateDataFieldDescriptor],
    commands: Vec<String>,
) -> Vec<String> {
    let storages = fields
        .iter()
        .filter(|field| field.keyed)
        .map(|field| field.storage)
        .collect::<std::collections::BTreeSet<_>>();
    if storages.is_empty() {
        return commands;
    }
    let condition = storages
        .into_iter()
        .map(ready)
        .collect::<Vec<_>>()
        .join(" ");
    commands
        .into_iter()
        .map(|command| format!("execute {condition} run {command}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::export::{functions, records};

    fn export(namespace: &str) -> Vec<(String, String)> {
        let _scope = crate::function::ExportFunctionRegistryScope::enter_namespace(namespace);
        let first = lifecycle("test:state");
        assert_eq!(
            first,
            lifecycle("test:state"),
            "component reuse is idempotent"
        );
        let mut output = Vec::new();
        functions::drain_generated_resources_into(
            &mut output,
            namespace,
            None,
            &mut Default::default(),
        )
        .unwrap();
        for record in &mut output {
            record.content = functions::resolve_local_refs(&record.content, namespace);
        }
        records::validate_unique_output_identities(&output).unwrap();
        let paths = output
            .iter()
            .map(|r| format!("{namespace}:{}", r.path))
            .collect::<std::collections::BTreeSet<_>>();
        for record in &output {
            assert!(!record.content.contains("__sand_local:"));
            for line in record.content.lines() {
                if let Some((_, call)) = line.split_once("function ") {
                    assert!(
                        paths.contains(call.split_whitespace().next().unwrap()),
                        "unresolved: {line}"
                    );
                }
            }
        }
        output.into_iter().map(|r| (r.path, r.content)).collect()
    }

    #[test]
    fn keyed_migration_helpers_are_complete_deterministic_and_export_isolated() {
        let first = export("first");
        assert_eq!(first, export("first"));
        let second = std::thread::spawn(|| export("second")).join().unwrap();
        assert_eq!(
            first,
            second
                .into_iter()
                .map(|(path, body)| (path, body.replace("second:", "first:")))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            first
                .iter()
                .map(|(_, body)| body.matches("owners[$(cursor)]").count())
                .sum::<usize>(),
            1
        );
        assert!(
            first
                .iter()
                .all(|(_, body)| !body.contains("owners[{uuid:"))
        );
    }

    #[test]
    fn keyed_migration_generated_function_collisions_are_rejected() {
        let _scope = crate::function::ExportFunctionRegistryScope::enter();
        lifecycle("test:state");
        let objective = sand_commands::ObjectiveName::logical("test:state::keyed_migration");
        register_dyn_fn(
            format!("sand/state_data/migrate/{}/step", objective.as_str()),
            vec!["say collision".to_string()],
        );
        let mut output = Vec::new();
        functions::drain_generated_resources_into(
            &mut output,
            "test",
            None,
            &mut Default::default(),
        )
        .unwrap();
        assert!(records::validate_unique_output_identities(&output).is_err());
    }
}
