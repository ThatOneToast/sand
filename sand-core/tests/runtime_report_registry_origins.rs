//! Deferred registries retain semantic owners through repeated export assembly.
use sand::events::{SandEvent, SandEventDispatch};
use sand::prelude::*;
use sand::registration::DatapackComponent;
use std::sync::OnceLock;

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "registry_owners", scope = player)]
struct AutomaticState {
    value: EntityScore<i32>,
}

/// Remain implementation-owned even when a separate factory contributes a tag.
#[function]
fn ordinary() {
    cmd::say("ordinary callback");
}

static DIALOG: OnceLock<sand::component::Dialog> = OnceLock::new();
/// Reuse a prebuilt dialog so attribution must occur during serialization.
#[datapack_component]
fn cached_dialog() -> sand::component::Dialog {
    DIALOG
        .get_or_init(|| {
            sand::component::Dialog::multi_action_local("cached").button(
                sand::component::DialogButton::new("Run")
                    .action(sand::component::DialogAction::callback(ordinary)),
            )
        })
        .clone()
}

struct NestedDialog(ResourceLocation);
impl DatapackComponent for NestedDialog {
    /// Identify the compound resource independently from its nested dialog.
    fn resource_location(&self) -> &ResourceLocation {
        &self.0
    }
    /// Emit a valid predicate as the compound component root.
    fn component_dir(&self) -> &'static str {
        "predicate"
    }
    /// Keep the root valid while its nested component registers the callback.
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({"condition":"minecraft:random_chance","chance":1.0})
    }
    /// Serialize callbacks after the original factory invocation has returned.
    fn nested_components(&self) -> Vec<Box<dyn DatapackComponent>> {
        vec![Box::new(
            sand::component::Dialog::multi_action_local("nested").button(
                sand::component::DialogButton::new("Run")
                    .action(sand::component::DialogAction::callback(ordinary)),
            ),
        )]
    }
}
/// Nested serialization must keep the root factory's actual origin.
#[datapack_component]
fn nested_dialog() -> NestedDialog {
    NestedDialog("__sand_local:dialog_bundle".parse().unwrap())
}

static CONSTANT: OnceLock<sand_core::state::ScoreOperand> = OnceLock::new();
/// Cached score operands replay initialization requests for each caller.
fn constant() -> sand_core::state::ScoreOperand {
    CONSTANT
        .get_or_init(|| sand_core::state::ScoreConst::<i32>::new("report_shared", 7).ref_())
        .clone()
}
/// First caller of shared score setup.
#[function]
fn numeric_one() {
    sand_core::state::ScoreVar::<i32>::new("report_value")
        .of("@s")
        .assign(constant());
}
/// Second caller must be retained even when the constant already exists.
#[function]
fn numeric_two() {
    sand_core::state::ScoreVar::<i32>::new("report_value")
        .of("@s")
        .assign(constant());
}

struct HealthChange;
impl SandEvent for HealthChange {
    /// Both subscribers share a single native health observation backend.
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::observe_players(|player| StatCurve::from(player.living().health()))
            .changed()
    }
}
/// First subscriber to a shared transition detector.
#[on_event]
fn tracked_one(_: HealthChange) {
    cmd::say("tracked one");
}
/// Second subscriber must own the same detector and lifecycle work.
#[on_event]
fn tracked_two(_: HealthChange) {
    cmd::say("tracked two");
}

/// Each deferred backend keeps all contributors without relabeling unrelated work.
#[test]
fn deferred_registry_resources_keep_origins() {
    let _ = constant();
    // The macro replaces the callable item, so prebuild through its shared cache.
    DIALOG.get_or_init(|| {
        sand::component::Dialog::multi_action_local("cached").button(
            sand::component::DialogButton::new("Run")
                .action(sand::component::DialogAction::callback(ordinary)),
        )
    });
    for namespace in ["registry_owners", "other_registry"] {
        let json = sand::advanced::try_export_pack_json(namespace, "26.2").unwrap();
        let export: serde_json::Value = serde_json::from_str(&json).unwrap();
        let report = &export["runtime_report"];
        let owner = |name| serde_json::Value::String(format!("{}::{name}", module_path!()));
        let state =
            serde_json::Value::String(<AutomaticState as sand::entity::EntityState>::schema().id());
        for (phase, path) in [
            ("load", "__sand_lifecycle_load"),
            ("tick", "__sand_lifecycle_tick"),
        ] {
            let id = format!("{namespace}:{path}");
            let owners = report["functions"][&id]["owners"].as_array().unwrap();
            for expected in [&state, &owner("tracked_one"), &owner("tracked_two")] {
                assert!(owners.contains(expected), "{id}: missing {expected}");
            }
            let entry = report[phase]
                .as_array()
                .unwrap()
                .iter()
                .find(|entry| entry["reference"] == id)
                .unwrap();
            assert_eq!(entry["contributors"], report["functions"][&id]["owners"]);
        }
        assert!(
            report["functions"][format!("{namespace}:__sand_lifecycle_init")]["owners"]
                .as_array()
                .unwrap()
                .contains(&state)
        );
        let expected = serde_json::json!([owner("tracked_one"), owner("tracked_two")]);
        let trackers: Vec<_> = report["functions"]
            .as_object()
            .unwrap()
            .iter()
            .filter(|(id, _)| id.contains(":__sand_transition/"))
            .collect();
        assert_eq!(trackers.len(), 1);
        assert_eq!(trackers[0].1["owners"], expected);
        for (phase, path) in [
            ("load", "__sand_dialog_init"),
            ("tick", "__sand_dialog_tick"),
        ] {
            let id = format!("{namespace}:{path}");
            let expected = serde_json::json!([owner("cached_dialog"), owner("nested_dialog")]);
            assert_eq!(report["functions"][&id]["owners"], expected);
            let entry = report[phase]
                .as_array()
                .unwrap()
                .iter()
                .find(|entry| entry["reference"] == id)
                .unwrap();
            assert_eq!(entry["contributors"], expected);
        }
        let score = format!("{namespace}:__sand_score_init");
        let score_owners = report["functions"][&score]["owners"].as_array().unwrap();
        for name in ["numeric_one", "numeric_two"] {
            assert!(score_owners.contains(&owner(name)));
        }
        assert!(!score_owners.contains(&owner("ordinary")));
        let score_entry = report["load"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["reference"] == score)
            .unwrap();
        assert_eq!(
            score_entry["contributors"],
            report["functions"][&score]["owners"]
        );
        assert_eq!(
            json,
            sand::advanced::try_export_pack_json(namespace, "26.2").unwrap()
        );
    }
}
