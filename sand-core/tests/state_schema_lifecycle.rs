use sand::prelude::*;

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "state_test", scope = player)]
struct PlayerState {
    #[state(default = 100, min = 0, max = 100)]
    mana: EntityScore<i32>,
    #[state(default = false)]
    enabled: EntityFlag,
    #[state(default = 0, auto_tick)]
    timer: EntityTimer,
    #[state(auto_tick)]
    dash: EntityCooldown,
}

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "state_test", scope = global)]
struct GlobalState {
    #[state(
        default = 1,
        criterion = "playerKillCount",
        display_name = "World wave"
    )]
    wave: EntityScore<i32>,
}

fn records() -> Vec<serde_json::Value> {
    serde_json::from_str(&sand_core::try_export_components_json("statepack").unwrap()).unwrap()
}

fn function<'a>(records: &'a [serde_json::Value], path: &str) -> &'a str {
    records
        .iter()
        .find(|record| record["dir"] == "function" && record["path"] == path)
        .and_then(|record| record["content"].as_str())
        .unwrap_or_else(|| panic!("missing generated function {path}"))
}

#[test]
fn derived_state_lifecycle_is_scoped_deterministic_and_deduplicated() {
    let first = sand_core::try_export_components_json("statepack").unwrap();
    let second = sand_core::try_export_components_json("statepack").unwrap();
    assert_eq!(first, second);

    let records = records();
    let load = function(&records, "__sand_lifecycle_load");
    let objectives: Vec<_> = load
        .lines()
        .filter(|line| line.starts_with("scoreboard objectives add "))
        .collect();
    let unique: std::collections::BTreeSet<_> = objectives.iter().copied().collect();
    assert_eq!(objectives.len(), 5);
    assert_eq!(unique.len(), objectives.len());
    assert!(load.contains("#sand_state_test_global_state"));
    assert!(load.contains("playerKillCount \"World wave\""));

    let init = function(&records, "__sand_lifecycle_init");
    assert_eq!(init.lines().count(), 4);
    assert!(init.lines().all(|line| line.contains("score @s ")));

    let tick = function(&records, "__sand_lifecycle_tick");
    assert!(tick.contains("execute as @a run function statepack:__sand_lifecycle_init"));
    assert_eq!(tick.matches("scoreboard players remove @s").count(), 2);
    assert!(!tick.contains("execute as @e"));
}

#[test]
fn bound_views_expose_concrete_fields_and_independent_schema_identities() {
    let player = PlayerState::on(PlayerContext::default());
    assert!(player.mana.set(25)[0].contains("@s"));
    assert!(player.enabled.enable()[0].contains("@s"));
    assert!(player.timer.start(Ticks::new(5))[0].contains("@s"));
    assert!(player.dash.start(Ticks::new(10))[0].contains("@s"));

    let global = GlobalState::global();
    assert!(global.wave.add(1)[0].contains("#sand_state_test_global_state"));
    assert_ne!(PlayerState::mana.objective(), GlobalState::wave.objective());
}
