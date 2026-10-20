//! Live-server fixture: a mutating handler followed by typed snapshot assertions.
use sand::events::PlayerLevelUpEvent;
use sand::prelude::*;

fn mutate() -> Actions {
    sand::component::IntoCommands::into_commands(vec![
        "scoreboard players add @s xp_calls 1",
        "execute if entity @s[tag=xp_mutate] run experience set @s 9 levels",
    ])
}

fn snapshots() -> Actions {
    let player = Target::current_player();
    let snapshot = |current, previous| {
        PlayerLevelUpEvent::current_level(player.clone())
            .eq(current)
            .and(PlayerLevelUpEvent::previous_level(player.clone()).eq(previous))
            .and(PlayerLevelUpEvent::level_delta(player.clone()).eq(2))
    };
    mcfunction![
        when(snapshot(7, 5)).then_all(["scoreboard players add @s xp_first 1"]);
        when(snapshot(9, 7)).then_all(["scoreboard players add @s xp_second 1"]);
    ]
}

sand_core::inventory::submit! {
    sand_core::EventDescriptor { owner: module_path!(), path: "a_mutate", id_override: None,
        make: mutate, dispatch: sand_core::EventDispatch::XpLevelUp }
}
sand_core::inventory::submit! {
    sand_core::EventDescriptor { owner: module_path!(), path: "b_snapshots", id_override: None,
        make: snapshots, dispatch: sand_core::EventDispatch::XpLevelUp }
}

#[test]
fn typed_callback_snapshots_keep_the_existing_owned_objectives() {
    let records: Vec<serde_json::Value> =
        serde_json::from_str(&sand_core::try_export_components_json("xp_level").unwrap()).unwrap();
    let functions = records
        .iter()
        .filter(|r| r["dir"] == "function")
        .map(|r| r["content"].as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(functions.contains("scoreboard objectives add __sx_xp_level.s dummy"));
    for role in ["c", "p", "d"] {
        assert!(functions.contains(&format!(
            "run scoreboard players get @s __sx_xp_level.{role}"
        )));
    }
    assert!(!functions.contains("run scoreboard players get @s __sx_xp_level.s"));
}

#[test]
#[ignore = "external live-server fixture export"]
fn dump_xp_level_pack() {
    std::fs::write(
        std::env::var("SAND_XP_DUMP_PATH").expect("dump path"),
        sand_core::try_export_components_json("xp_level").unwrap(),
    )
    .unwrap();
}
