use sand_core::sand_state;
use sand_core::state::{Cooldown, Flag, GameState, ScoreVar, Ticks, Timer, TypedGameState};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Idle = 0,
    Active = 1,
}

impl TypedGameState for Phase {
    fn to_score(self) -> i32 {
        self as i32
    }

    fn from_score(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Idle),
            1 => Some(Self::Active),
            _ => None,
        }
    }
}

mod player_state {
    use super::*;

    sand_state! {
        pub static MANA: ScoreVar<i32> = ScoreVar::new("auto_mana") =>
            MANA.lifecycle().default(100);
        pub static ENABLED: Flag = Flag::new("auto_enabled") =>
            ENABLED.lifecycle().default(0);
        pub static PHASE: GameState<super::Phase> = GameState::with_default_score("auto_phase", 0) =>
            PHASE.lifecycle();
    }
}

mod countdowns {
    use super::*;

    sand_state! {
        pub static TIMER: Timer = Timer::new("auto_timer", Ticks::new(40)) =>
            TIMER.lifecycle().default(0).auto_tick();
        pub static DASH: Cooldown = Cooldown::new("auto_dash", Ticks::new(60)) =>
            DASH.lifecycle().default(0).auto_tick();
        #[allow(dead_code)]
        static DASH_DUPLICATE: Cooldown = Cooldown::new("auto_dash", Ticks::new(60)) =>
            DASH_DUPLICATE.lifecycle().default(0).auto_tick();
    }
}

fn exported_records() -> Vec<serde_json::Value> {
    serde_json::from_str(&sand_core::try_export_components_json("autopack").unwrap()).unwrap()
}

fn function_content<'a>(records: &'a [serde_json::Value], path: &str) -> &'a str {
    records
        .iter()
        .find(|record| record["dir"] == "function" && record["path"] == path)
        .and_then(|record| record["content"].as_str())
        .unwrap_or_else(|| panic!("missing generated function {path}"))
}

fn tag_values(records: &[serde_json::Value], path: &str) -> Vec<String> {
    let content = records
        .iter()
        .find(|record| record["namespace"] == "minecraft" && record["path"] == path)
        .and_then(|record| record["content"].as_str())
        .unwrap_or_else(|| panic!("missing generated tag {path}"));
    serde_json::from_str::<serde_json::Value>(content).unwrap()["values"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn declarations_generate_deterministic_load_tick_and_first_seen_wiring() {
    // Referencing the statics proves the macro leaves the normal typed API intact.
    assert_eq!(player_state::MANA.objective_name(), "auto_mana");
    assert_eq!(player_state::ENABLED.objective_name(), "auto_enabled");
    assert_eq!(player_state::PHASE.objective_name(), "auto_phase");
    assert_eq!(countdowns::TIMER.objective_name(), "auto_timer");
    assert_eq!(countdowns::DASH.objective_name(), "auto_dash");

    let first_json = sand_core::try_export_components_json("autopack").unwrap();
    let second_json = sand_core::try_export_components_json("autopack").unwrap();
    assert_eq!(
        first_json, second_json,
        "repeated exports must be identical"
    );
    let records: Vec<serde_json::Value> = serde_json::from_str(&first_json).unwrap();

    assert_eq!(
        function_content(&records, "__sand_lifecycle_load"),
        [
            "scoreboard objectives add auto_dash dummy",
            "scoreboard objectives add auto_enabled dummy",
            "scoreboard objectives add auto_mana dummy",
            "scoreboard objectives add auto_phase dummy",
            "scoreboard objectives add auto_timer dummy",
        ]
        .join("\n")
    );

    let init = function_content(&records, "__sand_lifecycle_init");
    assert!(init.contains("unless score @s auto_mana matches -2147483648.."));
    assert!(init.contains("scoreboard players set @s auto_mana 100"));
    assert!(!init.contains("scoreboard players set @a"));

    assert_eq!(
        function_content(&records, "__sand_lifecycle_tick"),
        [
            "execute as @a run function autopack:__sand_lifecycle_init",
            "execute as @a run execute if score @s auto_dash matches 1.. run scoreboard players remove @s auto_dash 1",
            "execute as @a run execute if score @s auto_timer matches 1.. run scoreboard players remove @s auto_timer 1",
        ]
        .join("\n")
    );

    assert!(tag_values(&records, "load").contains(&"autopack:__sand_lifecycle_load".to_string()));
    assert!(tag_values(&records, "tick").contains(&"autopack:__sand_lifecycle_tick".to_string()));

    // Missing-score guards preserve any existing value on reload or rejoin.
    assert!(
        init.lines()
            .all(|line| line.starts_with("execute unless score @s "))
    );
    assert_eq!(exported_records(), records);
}
