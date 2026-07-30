use sand::prelude::*;

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "state_test", scope = player)]
struct PlayerState {
    #[state(
        default = 100,
        min = 0,
        max = 100,
        criterion = "dummy",
        display_name = "Player mana"
    )]
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

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "state_test", scope = entity)]
struct EntityRuntimeState {
    #[state(default = 0, min = 0, max = 10)]
    charge: EntityScore<i32>,
    cooldown: EntityCooldown,
}

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "state_test", scope = living)]
struct LivingRuntimeState {
    timer: EntityTimer,
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
    assert!(load.contains("dummy \"Player mana\""));
    assert!(load.contains("playerKillCount \"World wave\""));

    let init = function(&records, "__sand_lifecycle_init");
    assert_eq!(init.lines().count(), 4);
    assert!(init.lines().all(|line| line.contains("score @s ")));

    let tick = function(&records, "__sand_lifecycle_tick");
    assert!(tick.contains("execute as @a run function statepack:__sand_lifecycle_init"));
    assert_eq!(tick.matches("scoreboard players remove @s").count(), 2);
    assert!(!tick.contains("execute as @e"));
    for dirty in [
        PlayerState::timer.dirty_objective(),
        PlayerState::dash.dirty_objective(),
    ] {
        assert!(!tick.contains(&dirty));
    }
}

#[test]
fn bound_views_emit_complete_scope_aware_command_vectors() {
    let player = PlayerState::on(PlayerContext::default());
    let mana = PlayerState::mana.objective();
    assert_eq!(
        player.mana.set(25),
        vec![
            format!("scoreboard players set @s {mana} 25"),
            format!(
                "execute if score @s {mana} matches ..-1 run scoreboard players set @s {mana} 0"
            ),
            format!(
                "execute if score @s {mana} matches 101.. run scoreboard players set @s {mana} 100"
            ),
        ]
    );
    let enabled = PlayerState::enabled.objective();
    assert_eq!(
        player.enabled.enable(),
        vec![
            format!("scoreboard players set @s {enabled} 1"),
            format!(
                "execute if score @s {enabled} matches ..-1 run scoreboard players set @s {enabled} 0"
            ),
            format!(
                "execute if score @s {enabled} matches 2.. run scoreboard players set @s {enabled} 1"
            ),
        ]
    );
    assert_eq!(
        player.enabled.disable(),
        vec![
            format!("scoreboard players set @s {enabled} 0"),
            format!(
                "execute if score @s {enabled} matches ..-1 run scoreboard players set @s {enabled} 0"
            ),
            format!(
                "execute if score @s {enabled} matches 2.. run scoreboard players set @s {enabled} 1"
            ),
        ]
    );
    let timer = PlayerState::timer.objective();
    assert_eq!(
        player.timer.start(Ticks::new(5)),
        vec![
            format!("scoreboard players set @s {timer} 5"),
            format!(
                "execute if score @s {timer} matches ..-1 run scoreboard players set @s {timer} 0"
            ),
        ]
    );
    assert_eq!(
        player.timer.tick(),
        vec![format!(
            "execute if score @s {timer} matches 1.. run scoreboard players remove @s {timer} 1"
        )]
    );
    let dash = PlayerState::dash.objective();
    assert_eq!(
        player.dash.start(Ticks::new(10)),
        vec![
            format!("scoreboard players set @s {dash} 10"),
            format!(
                "execute if score @s {dash} matches ..-1 run scoreboard players set @s {dash} 0"
            ),
        ]
    );

    let global = GlobalState::global();
    let wave = GlobalState::wave.objective();
    let holder = "#sand_state_test_global_state_e53d9a32";
    assert_eq!(
        global.wave.add(1),
        vec![format!("scoreboard players add {holder} {wave} 1")]
    );
    assert_eq!(
        global.wave.get().command(),
        format!("scoreboard players get {holder} {wave}")
    );
    assert_ne!(PlayerState::mana.objective(), GlobalState::wave.objective());
}

#[test]
fn entity_and_living_bound_views_retain_archetype_dirty_semantics() {
    let entity = EntityRuntimeState::on(EntityContext::<AnyEntity>::default());
    let charge = EntityRuntimeState::charge.objective();
    let charge_dirty = EntityRuntimeState::charge.dirty_objective();
    assert_eq!(
        entity.charge.add(2),
        vec![
            format!("scoreboard players add @s {charge} 2"),
            format!(
                "execute if score @s {charge} matches ..-1 run scoreboard players set @s {charge} 0"
            ),
            format!(
                "execute if score @s {charge} matches 11.. run scoreboard players set @s {charge} 10"
            ),
            format!("scoreboard players set @s {charge_dirty} 1"),
        ]
    );
    let cooldown = EntityRuntimeState::cooldown.objective();
    let cooldown_dirty = EntityRuntimeState::cooldown.dirty_objective();
    assert_eq!(
        entity.cooldown.start(Ticks::new(8)),
        vec![
            format!("scoreboard players set @s {cooldown} 8"),
            format!(
                "execute if score @s {cooldown} matches ..-1 run scoreboard players set @s {cooldown} 0"
            ),
            format!("scoreboard players set @s {cooldown_dirty} 1"),
        ]
    );

    let living = LivingRuntimeState::on(EntityContext::<ZombieKind>::default());
    let timer = LivingRuntimeState::timer.objective();
    let timer_dirty = LivingRuntimeState::timer.dirty_objective();
    assert_eq!(
        living.timer.start(Ticks::new(4)),
        vec![
            format!("scoreboard players set @s {timer} 4"),
            format!(
                "execute if score @s {timer} matches ..-1 run scoreboard players set @s {timer} 0"
            ),
            format!("scoreboard players set @s {timer_dirty} 1"),
        ]
    );
    assert_eq!(
        living.timer.tick(),
        vec![
            format!(
                "execute if score @s {timer} matches 1.. run scoreboard players set @s {timer_dirty} 1"
            ),
            format!(
                "execute if score @s {timer} matches 1.. run scoreboard players remove @s {timer} 1"
            ),
        ]
    );
}

fn referenced_objectives(
    commands: impl IntoIterator<Item = String>,
) -> std::collections::BTreeSet<String> {
    let mut objectives = std::collections::BTreeSet::new();
    for command in commands {
        let tokens: Vec<_> = command.split_whitespace().collect();
        for window in tokens.windows(5) {
            if window[0] == "scoreboard" && window[1] == "players" {
                objectives.insert(window[4].to_string());
            }
            if window[0] == "score" {
                objectives.insert(window[2].to_string());
            }
        }
    }
    objectives
}

#[test]
fn player_and_global_commands_reference_only_provisioned_objectives() {
    let records = records();
    let provisioned: std::collections::BTreeSet<_> = function(&records, "__sand_lifecycle_load")
        .lines()
        .filter_map(|line| {
            line.strip_prefix("scoreboard objectives add ")
                .and_then(|rest| rest.split_whitespace().next())
                .map(str::to_owned)
        })
        .collect();

    let player = PlayerState::on(PlayerContext::default());
    let global = GlobalState::global();
    let commands = [
        player.mana.set(25),
        player.enabled.enable(),
        player.enabled.disable(),
        player.timer.start(Ticks::new(5)),
        player.timer.tick(),
        player.dash.start(Ticks::new(10)),
        global.wave.add(1),
        vec![global.wave.get().command()],
    ]
    .concat();
    let referenced = referenced_objectives(commands.clone());
    assert!(
        referenced.is_subset(&provisioned),
        "unprovisioned objectives: {:?}",
        referenced.difference(&provisioned).collect::<Vec<_>>()
    );
    for dirty in [
        PlayerState::mana.dirty_objective(),
        PlayerState::enabled.dirty_objective(),
        PlayerState::timer.dirty_objective(),
        PlayerState::dash.dirty_objective(),
        GlobalState::wave.dirty_objective(),
    ] {
        assert!(
            commands.iter().all(|command| !command.contains(&dirty)),
            "player/global commands referenced dirty objective {dirty}"
        );
    }
}
