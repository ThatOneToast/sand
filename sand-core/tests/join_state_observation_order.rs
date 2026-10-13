//! Joins share State initialization and run before owned observation sampling.
use sand::events::OnJoinEvent;
use sand::prelude::*;

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "join_state", scope = player)]
struct PlayerState {
    #[state(default = 10)]
    value: EntityScore<i32>,
}

#[on_event]
fn joined(_: Event<OnJoinEvent>) {
    cmd::say("joined with state");
}
struct HealthChanged;
impl SandEvent for HealthChanged {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::observe_players(|player| StatCurve::from(player.living().health()))
            .changed()
    }
}
#[on_event]
fn health_changed(_: HealthChanged) {
    cmd::say("health changed");
}

#[test]
fn canonical_lifecycle_provisions_before_join_and_samples_after_join() {
    let json = sand_core::try_export_components_json("join_state").unwrap();
    let records: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
    let content = |path: &str| {
        records
            .iter()
            .find(|r| r["dir"] == "function" && r["path"] == path)
            .unwrap()["content"]
            .as_str()
            .unwrap()
    };
    let tick = content("__sand_lifecycle_tick");
    let init = tick
        .find("function join_state:__sand_lifecycle_init")
        .unwrap();
    let join = tick.find("function join_state:__sand_join_check").unwrap();
    let observation = tick.find("function join_state:__sand_transition/").unwrap();
    assert!(init < join && join < observation, "{tick}");
    assert!(content("__sand_lifecycle_init").contains(&PlayerState::value.objective()));
    let tick_tag = records
        .iter()
        .find(|r| {
            r["dir"] == "tags/function" && r["namespace"] == "minecraft" && r["path"] == "tick"
        })
        .unwrap();
    let tag: serde_json::Value =
        serde_json::from_str(tick_tag["content"].as_str().unwrap()).unwrap();
    assert_eq!(
        tag["values"],
        serde_json::json!(["join_state:__sand_lifecycle_tick"])
    );
}
