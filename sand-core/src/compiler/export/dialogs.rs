//! Dialog callback dispatch phase of the export pipeline.
//!
//! Drains callbacks registered while dialog components were constructed and
//! generates the `__sand_dialog_init`/`__sand_dialog_tick` trigger
//! infrastructure.

use super::records::ComponentRecord;

/// Drain dialog callbacks into generated trigger/load/tick infrastructure.
pub(crate) fn drain_dialog_callbacks_into(
    records: &mut Vec<ComponentRecord>,
    tag_map: &mut std::collections::BTreeMap<String, Vec<String>>,
    namespace: &str,
) {
    let callbacks = sand_components::dialog::drain_dialog_callbacks();
    if callbacks.is_empty() {
        return;
    }

    let trigger = sand_components::dialog::SAND_DIALOG_TRIGGER;

    let init_cmds = [
        format!("scoreboard objectives add {trigger} trigger"),
        format!("scoreboard players enable @a {trigger}"),
    ];
    records.push(ComponentRecord {
        namespace: namespace.to_string(),
        dir: "function".to_string(),
        path: "__sand_dialog_init".to_string(),
        ext: "mcfunction".to_string(),
        content_type: "text".to_string(),
        content: init_cmds.join("\n"),
    });
    tag_map
        .entry("minecraft:load".to_string())
        .or_default()
        .push(format!("{namespace}:__sand_dialog_init"));

    let mut tick_cmds: Vec<String> = Vec::new();
    tick_cmds.push(format!("scoreboard players enable @a {trigger}"));
    for (id, path) in &callbacks {
        tick_cmds.push(format!(
            "execute as @a[scores={{{trigger}={id}}}] at @s run function {path}"
        ));
        tick_cmds.push(format!(
            "scoreboard players set @a[scores={{{trigger}={id}}}] {trigger} 0"
        ));
        tick_cmds.push(format!("scoreboard players enable @a {trigger}"));
    }
    records.push(ComponentRecord {
        namespace: namespace.to_string(),
        dir: "function".to_string(),
        path: "__sand_dialog_tick".to_string(),
        ext: "mcfunction".to_string(),
        content_type: "text".to_string(),
        content: tick_cmds.join("\n"),
    });
    tag_map
        .entry("minecraft:tick".to_string())
        .or_default()
        .push(format!("{namespace}:__sand_dialog_tick"));
}

/// Process-global lock held for the complete factory/export lifecycle so
/// repeated or concurrent exports cannot inherit dialog callback state from
/// one another.
///
/// Recovers from poison: this lock guards a `Mutex<()>` with no invariants
/// of its own, so a panic while it was held (e.g. inside a user
/// `ComponentFactory`) leaves nothing broken to propagate. Without this,
/// one caught factory panic (as happens under `catch_unwind` or a test
/// harness) would permanently poison every later export in the process.
pub(crate) fn dialog_callback_export_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Clears process-global dialog callback state when an export finishes, even
/// when a component factory returns an error or panics.
///
/// Safe to construct before component factories run: callback registration
/// happens at `DialogAction::to_json` (serialization) time, not at
/// `DialogAction::callback` (construction) time, so resetting the registry
/// up front never discards a prebuilt/cached dialog's callback — it just
/// gets re-registered fresh when that dialog is serialized during this
/// export. See `sand_components::dialog::reset_dialog_callbacks_for_export`.
pub(crate) struct DialogCallbackExportReset;

impl Drop for DialogCallbackExportReset {
    fn drop(&mut self) {
        sand_components::dialog::reset_dialog_callbacks_for_export();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn late_dialog_callback_drain_emits_dispatcher_after_component_construction() {
        let _lock = super::dialog_callback_export_lock();
        let _ = sand_components::dialog::drain_dialog_callbacks();

        assert!(
            sand_components::dialog::drain_dialog_callbacks().is_empty(),
            "test starts from the old early-drain state"
        );

        let dialog = sand_components::dialog::Dialog::multi_action_local("welcome").button(
            sand_components::dialog::DialogButton::new("Grant").action(
                sand_components::dialog::DialogAction::callback("__sand_local:grant_reward"),
            ),
        );
        let dialog_json = dialog.to_json();
        let command = dialog_json["actions"][0]["action"]["command"]
            .as_str()
            .expect("dialog callback button should emit a command action");
        let callback_id = command
            .strip_prefix("/trigger sand.dialog set ")
            .expect("callback action should use the Sand dialog trigger");

        let mut records = Vec::new();
        let mut tag_map = std::collections::BTreeMap::new();
        super::drain_dialog_callbacks_into(&mut records, &mut tag_map, "dialogpack");

        let init_recs: Vec<_> = records
            .iter()
            .filter(|r| r.path == "__sand_dialog_init")
            .collect();
        assert_eq!(init_recs.len(), 1, "dialog init function should be emitted");
        assert!(
            init_recs[0]
                .content
                .contains("scoreboard objectives add sand.dialog trigger"),
            "dialog init function must create the trigger objective"
        );

        let tick_recs: Vec<_> = records
            .iter()
            .filter(|r| r.path == "__sand_dialog_tick")
            .collect();
        assert_eq!(tick_recs.len(), 1, "dialog tick function should be emitted");
        let tick_content = &tick_recs[0].content;
        assert!(
            tick_content.contains(&format!(
                "execute as @a[scores={{sand.dialog={callback_id}}}] at @s run function __sand_local:grant_reward"
            )),
            "dialog tick function must dispatch the registered callback, got: {tick_content}"
        );
        assert!(
            tick_content.contains(&format!(
                "scoreboard players set @a[scores={{sand.dialog={callback_id}}}] sand.dialog 0"
            )),
            "dialog tick function must reset the callback score, got: {tick_content}"
        );

        assert_eq!(
            tag_map.get("minecraft:load").cloned().unwrap_or_default(),
            vec!["dialogpack:__sand_dialog_init".to_string()]
        );
        assert_eq!(
            tag_map.get("minecraft:tick").cloned().unwrap_or_default(),
            vec!["dialogpack:__sand_dialog_tick".to_string()]
        );

        let _ = sand_components::dialog::drain_dialog_callbacks();
    }

    #[test]
    fn empty_dialog_callback_registry_emits_no_dispatcher() {
        let _lock = super::dialog_callback_export_lock();
        let _ = sand_components::dialog::drain_dialog_callbacks();

        let mut records = Vec::new();
        let mut tag_map = std::collections::BTreeMap::new();
        super::drain_dialog_callbacks_into(&mut records, &mut tag_map, "dialogpack");

        assert!(
            records.iter().all(|r| r.path != "__sand_dialog_init"),
            "no dialog init function should appear with no callbacks"
        );
        assert!(
            !tag_map.contains_key("minecraft:load"),
            "no load tag entry should appear with no callbacks"
        );
        assert!(
            !tag_map.contains_key("minecraft:tick"),
            "no tick tag entry should appear with no callbacks"
        );
    }

    #[test]
    fn dialog_callback_export_lock_recovers_after_a_caught_panic() {
        let panic = std::panic::catch_unwind(|| {
            let _lock = super::dialog_callback_export_lock();
            panic!("simulated component factory panic");
        });
        assert!(panic.is_err(), "the simulated factory must panic");

        // A caught panic while the lock was held must not permanently poison
        // later exports in the same process.
        let _lock = super::dialog_callback_export_lock();
        let _ = sand_components::dialog::drain_dialog_callbacks();
    }

    #[test]
    fn export_scope_reset_guard_clears_state_on_success_and_on_panic() {
        {
            let _lock = super::dialog_callback_export_lock();
            let _ = sand_components::dialog::drain_dialog_callbacks();
            let _reset = super::DialogCallbackExportReset;
            let _ = sand_components::dialog::register_dialog_callback(
                "example:successful_export".to_string(),
            );
            // `_reset` drops here (success path) and must clear state.
        }
        assert!(
            sand_components::dialog::drain_dialog_callbacks().is_empty(),
            "a successful export scope must leave no callback state behind"
        );

        let panic = std::panic::catch_unwind(|| {
            let _lock = super::dialog_callback_export_lock();
            let _reset = super::DialogCallbackExportReset;
            let _ = sand_components::dialog::register_dialog_callback(
                "example:panicking_export".to_string(),
            );
            panic!("simulated failure mid-export");
            // `_reset` drops during unwind and must clear state.
        });
        assert!(panic.is_err());

        let _lock = super::dialog_callback_export_lock();
        assert!(
            sand_components::dialog::drain_dialog_callbacks().is_empty(),
            "a panicking export scope must still leave no callback state behind"
        );
    }
}
