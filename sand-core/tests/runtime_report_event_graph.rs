//! Graph provenance follows actual occurrence consumers and lifecycle wiring.
use sand::events::{EventSetup, SandEvent, SandEventDispatch};
use sand::prelude::*;

struct First;
struct Second;
struct Combined;
struct Independent;

impl SandEvent for First {
    /// Observe this source independently once per tick.
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
    /// Make this source's setup and observation resources identifiable in output.
    fn setup() -> EventSetup {
        EventSetup {
            objectives: vec!["scoreboard objectives add origin_first dummy".into()],
            pre_observation: vec!["say first observation".into()],
            post_observation: vec![],
        }
    }
}
impl SandEvent for Second {
    /// Observe this source independently once per tick.
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
}
impl SandEvent for Combined {
    /// Consume either parent occurrence through the staged coordinator.
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::after_any::<(First, Second)>()
    }
}
impl SandEvent for Independent {
    /// Observe this source independently once per tick.
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
    /// Make this source's setup and observation resources identifiable in output.
    fn setup() -> EventSetup {
        EventSetup {
            objectives: vec!["scoreboard objectives add origin_other dummy".into()],
            pre_observation: vec!["say independent observation".into()],
            post_observation: vec![],
        }
    }
}

/// Register a distinct semantic subscriber for attribution assertions.
#[on_event]
fn first_one(_: First) {
    cmd::say("first one");
}
/// Register a distinct semantic subscriber for attribution assertions.
#[on_event]
fn first_two(_: First) {
    cmd::say("first two");
}
/// Register a distinct semantic subscriber for attribution assertions.
#[on_event]
fn combined(_: Combined) {
    cmd::say("combined");
}
/// Register a distinct semantic subscriber for attribution assertions.
#[on_event]
fn independent(_: Independent) {
    cmd::say("independent");
}

/// Shared roots inherit dependent subscribers without contaminating unrelated nodes.
#[test]
fn graph_helpers_and_lifecycle_entries_retain_their_subscribers() {
    for namespace in ["graph_owners", "other_graph"] {
        let json = sand::advanced::try_export_pack_json(namespace, "26.2").unwrap();
        let export: serde_json::Value = serde_json::from_str(&json).unwrap();
        let report = &export["runtime_report"];
        let owner = |name| format!("{}::{name}", module_path!());
        let first_owners =
            serde_json::json!([owner("combined"), owner("first_one"), owner("first_two")]);
        let independent_owners = serde_json::json!([owner("independent")]);
        let all_owners = serde_json::json!([
            owner("combined"),
            owner("first_one"),
            owner("first_two"),
            owner("independent")
        ]);
        let records = export["components"].as_array().unwrap();
        for (marker, expected) in [
            (
                "scoreboard objectives add origin_first dummy",
                &first_owners,
            ),
            ("say first observation", &first_owners),
            (
                "scoreboard objectives add origin_other dummy",
                &independent_owners,
            ),
            ("say independent observation", &independent_owners),
        ] {
            let record = records
                .iter()
                .find(|record| {
                    record["dir"] == "function"
                        && record["content"].as_str().unwrap().contains(marker)
                })
                .unwrap();
            let function = format!(
                "{}:{}",
                record["namespace"].as_str().unwrap(),
                record["path"].as_str().unwrap()
            );
            assert_eq!(
                &report["functions"][function]["owners"], expected,
                "{marker}"
            );
        }
        for record in records.iter().filter(|record| {
            record["dir"] == "function"
                && record["path"]
                    .as_str()
                    .unwrap()
                    .starts_with("__sand_event_")
        }) {
            let function = format!("{namespace}:{}", record["path"].as_str().unwrap());
            assert!(
                !report["functions"][&function]["owners"]
                    .as_array()
                    .unwrap()
                    .is_empty(),
                "unowned {function}"
            );
            for phase in ["load", "tick"] {
                if let Some(entry) = report[phase]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|entry| entry["reference"] == function)
                {
                    assert_eq!(
                        entry["contributors"],
                        report["functions"][&function]["owners"]
                    );
                }
            }
        }
        assert_eq!(
            report["functions"][format!("{namespace}:__sand_event_cycle")]["owners"],
            all_owners
        );
        assert_eq!(
            json,
            sand::advanced::try_export_pack_json(namespace, "26.2").unwrap()
        );
    }
}
