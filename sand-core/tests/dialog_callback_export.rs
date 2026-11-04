//! Regression coverage for deterministic dialog callback IDs across
//! repeated in-process exports (#131).
//!
//! Before this fix, `DialogAction::callback` registered its callback (and
//! consumed a trigger ID) at *construction* time, and the process-global
//! counter/registry were only drained — never reset — by export. Repeated
//! exports of the same dialog graph in one process therefore assigned
//! increasing trigger IDs, and a prebuilt/cached dialog (e.g. behind a
//! `LazyLock`, built once) lost its callback registration entirely once an
//! earlier export drained it.

use sand_components::dialog::{Dialog, DialogAction, DialogButton};
use sand_core::component::export_components_json;
use sand_core::function::ComponentFactory;
use sand_core::{DatapackRegistration, FunctionId, IntoDatapack, inventory};
use std::sync::{LazyLock, Mutex};

/// The real export path serializes every export through
/// `dialog_callback_export_lock` (crate-private), so two exports in the same
/// process never interleave. That lock isn't reachable from this
/// integration-test crate, and tests in one file run on parallel threads of
/// one process by default — so tests here that touch the dialog callback
/// registry directly (not just through `export_components_json`) serialize
/// on this test-local lock instead, to avoid stealing each other's IDs.
static TEST_LOCK: Mutex<()> = Mutex::new(());

fn function_id(value: &str) -> FunctionId {
    value.parse().expect("valid callback function ID")
}

fn callback_dialog() -> DatapackRegistration {
    static DIALOG: LazyLock<Dialog> = LazyLock::new(|| {
        Dialog::multi_action_local("callback_menu")
            .button(
                DialogButton::new("First").action(DialogAction::callback(function_id(
                    "__sand_local:first_callback",
                ))),
            )
            .button(
                DialogButton::new("Second").action(DialogAction::callback(function_id(
                    "example:second_callback",
                ))),
            )
    });
    DIALOG.clone().into_datapack()
}

inventory::submit! {
    ComponentFactory { owner: "callback_dialog", make: callback_dialog }
}

fn record_content<'a>(records: &'a [serde_json::Value], path: &str) -> &'a str {
    records
        .iter()
        .find(|record| record["path"] == path)
        .and_then(|record| record["content"].as_str())
        .unwrap_or_else(|| panic!("missing generated record `{path}`"))
}

#[test]
fn repeated_exports_assign_stable_unique_dialog_callback_ids() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let first = export_components_json("dialogpack");
    assert!(
        sand_components::dialog::drain_dialog_callbacks().is_empty(),
        "export must leave the callback registry empty"
    );

    // Pollute the process-global lifecycle between exports. The next export
    // must discard this unrelated registration and still begin at trigger 1.
    let _ =
        sand_components::dialog::register_dialog_callback("example:unrelated_callback".to_string());
    let second = export_components_json("dialogpack");

    assert_eq!(first, second, "repeated exports must be byte-identical");
    assert!(
        sand_components::dialog::drain_dialog_callbacks().is_empty(),
        "repeated export must leave the callback registry empty"
    );

    let records: Vec<serde_json::Value> = serde_json::from_str(&second).unwrap();
    let dialog = record_content(&records, "callback_menu");
    let objective = sand_commands::ObjectiveName::logical("sand dialog trigger dialogpack");
    assert!(
        dialog.contains(&format!("/trigger {objective} set 1")),
        "{dialog}"
    );
    assert!(
        dialog.contains(&format!("/trigger {objective} set 2")),
        "{dialog}"
    );
    let dispatcher = record_content(&records, "__sand_dialog_dispatch");
    assert!(dispatcher.contains(&format!("if score @s {objective} matches 1 run return run function dialogpack:__sand_dialog_action/1")));
    for (id, callback) in [
        (1, "dialogpack:first_callback"),
        (2, "example:second_callback"),
    ] {
        let action = record_content(&records, &format!("__sand_dialog_action/{id}"));
        assert_eq!(
            action,
            format!("scoreboard players set @s {objective} 0\nreturn run function {callback}")
        );
    }
    assert!(!dispatcher.contains("unrelated_callback"));
}

// Regenerate this dialog *fresh* for every call (not cached behind a
// `LazyLock`), so its callback registration happens anew on every construction
// as well as every export. Used to confirm a third, later export is not
// shifted by a prior unrelated export's dialogs.
fn build_solo_callback_dialog() -> Dialog {
    Dialog::multi_action_local("solo_menu").button(DialogButton::new("Only").action(
        DialogAction::callback(function_id("__sand_local:solo_callback")),
    ))
}

#[test]
fn an_unrelated_prior_export_does_not_shift_a_later_exports_ids() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // First: export the multi-callback pack (defined above) once to leave
    // process history behind.
    let _ = export_components_json("dialogpack");

    // Then: independently render a fresh dialog and drive it through the
    // reset lifecycle exactly like an export would, without touching the
    // shared inventory-registered fixture.
    sand_components::dialog::reset_dialog_callbacks_for_export();
    let dialog = build_solo_callback_dialog();
    let json = dialog.to_json();
    let command = json["actions"][0]["action"]["command"]
        .as_str()
        .expect("solo dialog button should emit a run_command action");
    assert_eq!(
        command, "/trigger __sand_dialog_trigger__ set 1",
        "a solo export-scoped registration must start at 1 regardless of prior exports"
    );
    let _ = sand_components::dialog::drain_dialog_callbacks();
}

#[test]
fn prebuilt_dialog_callback_survives_a_reset_issued_before_it_is_serialized() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // Simulate a dialog that was constructed (and its DialogAction::callback
    // called) *before* reset_dialog_callbacks_for_export runs — e.g. a
    // LazyLock forced by unrelated code, or a unit test building a component
    // ahead of any export.
    let prebuilt =
        Dialog::multi_action_local("prebuilt_menu").button(DialogButton::new("Go").action(
            DialogAction::callback(function_id("__sand_local:prebuilt_callback")),
        ));

    // An export boundary starts: reset must not need to know this dialog
    // exists yet, and must not prevent it from registering when serialized.
    sand_components::dialog::reset_dialog_callbacks_for_export();

    let json = prebuilt.to_json();
    let command = json["actions"][0]["action"]["command"]
        .as_str()
        .expect("prebuilt dialog button should emit a run_command action");
    assert_eq!(
        command, "/trigger __sand_dialog_trigger__ set 1",
        "a prebuilt dialog constructed before reset must still register when serialized"
    );

    let drained = sand_components::dialog::drain_dialog_callbacks();
    assert_eq!(
        drained,
        vec![(1, "__sand_local:prebuilt_callback".to_string())],
        "the prebuilt dialog's callback must be present at drain time"
    );
}

#[test]
fn independently_exported_packs_have_distinct_callback_channels() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut channels = Vec::new();
    for namespace in ["first_pack", "second_pack"] {
        let records: Vec<serde_json::Value> =
            serde_json::from_str(&export_components_json(namespace)).unwrap();
        let dialog: serde_json::Value =
            serde_json::from_str(record_content(&records, "callback_menu")).unwrap();
        let command = dialog["actions"][0]["action"]["command"].as_str().unwrap();
        let channel = command.split_whitespace().nth(1).unwrap().to_string();
        let load = record_content(&records, "__sand_dialog_init");
        assert!(load.contains(&format!("scoreboard objectives add {channel} trigger")));
        assert!(load.contains(&format!("scoreboard players reset * {channel}")));
        let tick = record_content(&records, "__sand_dialog_tick");
        assert_eq!(tick.matches("scoreboard players enable").count(), 1);
        assert!(tick.contains(&format!("unless score @s {channel} matches 0")));
        channels.push(channel);
    }
    assert_ne!(channels[0], channels[1]);
}
