//! Public custom events share canonical player-scoped sources and owned resources.
use sand::prelude::*;

struct InventoryChanged;
impl SandEvent for InventoryChanged {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::observe_players(|player| {
            player.inventory().count(vanilla::Item::Diamond)
        })
        .changed()
    }
}
struct InventoryIncreased;
impl SandEvent for InventoryIncreased {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::observe_players(|player| {
            player.inventory().count(vanilla::Item::Diamond)
        })
        .increased()
    }
}
struct SpeedStarted;
impl SandEvent for SpeedStarted {
    fn dispatch() -> impl Into<SandEventDispatch> {
        static SOURCE: std::sync::OnceLock<Condition> = std::sync::OnceLock::new();
        SandEventDispatch::observe_players(|player| {
            SOURCE
                .get_or_init(|| player.living().has_effect(EffectId::Speed))
                .clone()
        })
        .became_true()
    }
}
struct HealthLost;
impl SandEvent for HealthLost {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::observe_players(|player| StatCurve::from(player.living().health()))
            .decreased()
    }
}

#[on_event]
fn inventory_changed(_: InventoryChanged) {
    cmd::say("inventory changed");
}
#[on_event]
fn inventory_increased(_: InventoryIncreased) {
    cmd::say("inventory increased");
}
#[on_event]
fn speed_started(_: SpeedStarted) {
    cmd::say("speed started");
}
#[on_event]
fn health_lost(_: HealthLost) {
    cmd::say("health lost");
}

fn verify(namespace: &str, json: &str) {
    let records: Vec<serde_json::Value> = serde_json::from_str(json).unwrap();
    let functions: Vec<_> = records.iter().filter(|r| r["dir"] == "function").collect();
    let samplers: Vec<_> = functions
        .iter()
        .filter(|r| {
            r["path"]
                .as_str()
                .unwrap()
                .starts_with("__sand_transition/")
        })
        .collect();
    assert_eq!(samplers.len(), 3, "inventory consumers share one sampler");
    let inventory = samplers
        .iter()
        .find(|r| {
            r["content"]
                .as_str()
                .unwrap()
                .contains(&format!("function {namespace}:inventory_changed"))
        })
        .unwrap();
    assert!(
        inventory["content"]
            .as_str()
            .unwrap()
            .contains(&format!("function {namespace}:inventory_increased"))
    );
    let body = functions
        .iter()
        .map(|r| r["content"].as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        body.matches(&format!(
            "execute as @a run function {namespace}:__sand_observation_tick"
        ))
        .count(),
        1,
        "distinct observations must share one positioned player scan"
    );
    assert_eq!(body.matches("clear @s minecraft:diamond 0").count(), 1);
    assert_eq!(body.matches("from entity @s Health").count(), 1);
    assert_eq!(
        records.iter().filter(|r| r["dir"] == "predicate").count(),
        1
    );
    assert!(!json.contains("__sand_local:"));
    for sampler in samplers {
        assert!(!sampler["content"].as_str().unwrap().contains("@a"));
        assert!(!sampler["content"].as_str().unwrap().contains("@e"));
    }
}

#[test]
fn subscriptions_share_sources_and_replay_cached_resources_across_exports() {
    let first = sand_core::try_export_components_json("observations").unwrap();
    verify("observations", &first);
    assert_eq!(
        first,
        sand_core::try_export_components_json("observations").unwrap()
    );
    let second = sand_core::try_export_components_json("other_pack").unwrap();
    verify("other_pack", &second);
    assert!(!second.contains("observations:"));
}
