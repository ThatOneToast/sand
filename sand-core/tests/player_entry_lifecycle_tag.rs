//! Generalized registration cannot bypass declared function execution context.
use sand::prelude::*;
use sand::registration::{DatapackRegistration, FunctionTagContribution};

#[function("player_entry", context = player)]
fn player_entry() {
    cmd::say("player body");
}

#[sand::datapack_component]
fn invalid_lifecycle_member() -> DatapackRegistration {
    DatapackRegistration::new().function_tag(FunctionTagContribution::new(
        "minecraft:load".parse().unwrap(),
        player_entry.function_id(),
    ))
}

#[test]
fn registration_rejects_player_entries_in_server_lifecycle_tags() {
    for namespace in ["first", "second", "first"] {
        let error = sand::advanced::try_export_components_json(namespace, "26.2")
            .unwrap_err()
            .to_string();
        assert!(error.contains("minecraft:load"), "{error}");
        assert!(
            error.contains(&format!("{namespace}:player_entry")),
            "{error}"
        );
        assert!(error.contains("server-context wrapper"), "{error}");
    }
}
