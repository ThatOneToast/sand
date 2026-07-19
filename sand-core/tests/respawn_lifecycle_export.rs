//! Regression coverage for the generated death-to-respawn lifecycle (#126).

use sand_core::{EventDescriptor, EventDispatch};

fn death_handler() -> Vec<String> {
    vec!["say death".to_string()]
}

fn respawn_handler_a() -> Vec<String> {
    vec!["say respawn a".to_string()]
}

fn respawn_handler_b() -> Vec<String> {
    vec!["say respawn b".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "death_handler",
        id_override: None,
        make: death_handler,
        dispatch: EventDispatch::DeathTick,
    }
}

// Deliberately register B before A. Generated fan-out order must be stable and
// independent of inventory/linker registration order.
sand_core::inventory::submit! {
    EventDescriptor {
        path: "respawn_handler_b",
        id_override: None,
        make: respawn_handler_b,
        dispatch: EventDispatch::RespawnTick,
    }
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "respawn_handler_a",
        id_override: None,
        make: respawn_handler_a,
        dispatch: EventDispatch::RespawnTick,
    }
}

fn export() -> String {
    sand_core::try_export_components_json("respawnpack").expect("export succeeds")
}

fn records() -> Vec<serde_json::Value> {
    serde_json::from_str(&export()).expect("export is JSON")
}

fn function_content<'a>(records: &'a [serde_json::Value], path: &str) -> &'a str {
    records
        .iter()
        .find(|record| {
            record["dir"].as_str() == Some("function") && record["path"].as_str() == Some(path)
        })
        .unwrap_or_else(|| panic!("missing function {path}"))["content"]
        .as_str()
        .expect("function content")
}

fn tag_values(records: &[serde_json::Value], path: &str) -> Vec<String> {
    let content = records
        .iter()
        .find(|record| {
            record["dir"].as_str() == Some("tags/function") && record["path"].as_str() == Some(path)
        })
        .unwrap_or_else(|| panic!("missing function tag {path}"))["content"]
        .as_str()
        .expect("tag content");
    serde_json::from_str::<serde_json::Value>(content).expect("tag is JSON")["values"]
        .as_array()
        .expect("tag values")
        .iter()
        .map(|value| value.as_str().expect("function reference").to_string())
        .collect()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Phase {
    #[default]
    Alive,
    WaitingForRespawn,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct PlayerLifecycle {
    phase: Phase,
    deaths: usize,
    respawn_handler_a: usize,
    respawn_handler_b: usize,
}

impl PlayerLifecycle {
    /// Mirrors the generated coordinator: observe a completed prior respawn
    /// first, then observe this cycle's new death. That ordering is what makes
    /// same-cycle death/respawn dispatch impossible for one death lifecycle.
    fn tick(&mut self, death_observed: bool, time_since_death: u32) {
        if self.phase == Phase::WaitingForRespawn && time_since_death > 0 {
            self.respawn_handler_a += 1;
            self.respawn_handler_b += 1;
            self.phase = Phase::Alive;
        }
        if death_observed {
            self.deaths += 1;
            self.phase = Phase::WaitingForRespawn;
        }
    }
}

#[test]
fn lifecycle_model_waits_for_alive_statistic_and_resets_for_a_second_death() {
    let mut player = PlayerLifecycle::default();

    player.tick(false, 80);
    assert_eq!(
        player,
        PlayerLifecycle::default(),
        "ordinary alive ticks do not fire"
    );

    player.tick(true, 0);
    assert_eq!(player.deaths, 1);
    assert_eq!(player.phase, Phase::WaitingForRespawn);
    assert_eq!(
        player.respawn_handler_a, 0,
        "death observation is not respawn"
    );

    for _ in 0..200 {
        player.tick(false, 0);
    }
    assert_eq!(
        player.respawn_handler_a, 0,
        "death-screen waiting never fires"
    );

    player.tick(false, 1);
    assert_eq!(player.respawn_handler_a, 1);
    assert_eq!(player.respawn_handler_b, 1);
    assert_eq!(player.phase, Phase::Alive);

    player.tick(false, 2);
    assert_eq!(
        player.respawn_handler_a, 1,
        "one lifecycle fires at most once"
    );

    player.tick(true, 0);
    player.tick(false, 0);
    player.tick(false, 1);
    assert_eq!(player.deaths, 2);
    assert_eq!(player.respawn_handler_a, 2);
    assert_eq!(player.respawn_handler_b, 2);
}

#[test]
fn immediate_respawn_still_requires_a_later_observation_cycle() {
    let mut player = PlayerLifecycle::default();
    player.tick(true, 1);
    assert_eq!(player.deaths, 1);
    assert_eq!(player.respawn_handler_a, 0);
    assert_eq!(player.phase, Phase::WaitingForRespawn);

    player.tick(false, 1);
    assert_eq!(player.respawn_handler_a, 1);
    assert_eq!(player.respawn_handler_b, 1);
}

#[test]
fn two_players_progress_independently() {
    let mut alice = PlayerLifecycle::default();
    let mut bob = PlayerLifecycle::default();

    alice.tick(true, 0);
    bob.tick(false, 40);
    bob.tick(true, 0);
    alice.tick(false, 1);
    bob.tick(false, 0);

    assert_eq!(alice.respawn_handler_a, 1);
    assert_eq!(alice.phase, Phase::Alive);
    assert_eq!(bob.respawn_handler_a, 0);
    assert_eq!(bob.phase, Phase::WaitingForRespawn);

    bob.tick(false, 1);
    assert_eq!(bob.respawn_handler_a, 1);
    assert_eq!(bob.respawn_handler_b, 1);
    assert_eq!(alice.respawn_handler_a, 1);
}

#[test]
fn generated_state_machine_uses_phase_and_time_since_death() {
    let records = records();
    assert_eq!(
        function_content(&records, "__sand_death_init"),
        "scoreboard objectives add __sand_dc deathCount\n\
         scoreboard objectives add __sand_tsd minecraft.custom:minecraft.time_since_death\n\
         scoreboard objectives add __sand_rp dummy"
    );
    assert_eq!(
        function_content(&records, "__sand_respawn_check"),
        "execute as @a[scores={__sand_rp=1,__sand_tsd=1..}] run function \
         respawnpack:__sand_respawn_dispatch"
    );

    let death = function_content(&records, "__sand_death_check");
    let respawn_pos = death
        .find("function respawnpack:__sand_respawn_check")
        .expect("coordinator invokes respawn check");
    let death_pos = death
        .find("scores={__sand_dc=1..}")
        .expect("coordinator observes deaths");
    let waiting_pos = death
        .find("scoreboard players set @s __sand_rp 1")
        .expect("death enters waiting phase as the selected player");
    assert!(
        respawn_pos < death_pos && death_pos < waiting_pos,
        "{death}"
    );
}

#[test]
fn every_respawn_handler_runs_once_before_the_phase_resets() {
    let records = records();
    assert_eq!(
        function_content(&records, "__sand_respawn_dispatch"),
        "function respawnpack:respawn_handler_a\n\
         function respawnpack:respawn_handler_b\n\
         scoreboard players set @s __sand_rp 0"
    );
}

#[test]
fn correctness_does_not_depend_on_function_tag_sorting() {
    let records = records();
    let tick = tag_values(&records, "tick");
    assert!(tick.contains(&"respawnpack:__sand_death_check".to_string()));
    assert!(
        !tick.contains(&"respawnpack:__sand_respawn_check".to_string()),
        "respawn ordering is an explicit function call, not a second tick-tag entry: {tick:?}"
    );
}

#[test]
fn old_non_spectator_tag_gate_is_absent() {
    let generated = records()
        .iter()
        .filter_map(|record| record["content"].as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!generated.contains("__sand_was_dead"));
    assert!(!generated.contains("gamemode=!spectator"));
}

#[test]
fn generated_commands_validate_across_the_supported_range() {
    for version in ["1.18.0", "1.21.4", "26.2"] {
        let resolved = sand_core::version::resolve_export_caps(version).expect("known profile");
        sand_core::try_export_components_json_for_version(
            "respawnpack",
            &resolved.caps,
            &resolved.version,
            resolved.is_fallback,
        )
        .unwrap_or_else(|error| panic!("{version} export must validate: {error}"));
    }
}

#[test]
fn repeated_exports_are_byte_identical() {
    assert_eq!(export(), export());
}
