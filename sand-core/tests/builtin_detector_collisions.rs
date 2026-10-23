//! Private built-in detector resources cannot be claimed by user registrations.
use sand::ResourceLocation;
use sand::component::McFunction;
use sand::registration::LifecycleContribution;
use sand_core::{ComponentFactory, DatapackRegistration, EventDescriptor, EventDispatch};
use std::sync::Mutex;

static COLLISION: Mutex<Option<(bool, String)>> = Mutex::new(None);

sand_core::inventory::submit! {
    EventDescriptor {
        owner: module_path!(),
        path: "died", id_override: None,
        make: || sand::component::IntoCommands::into_commands(Vec::<String>::new()),
        dispatch: EventDispatch::DeathTick,
    }
}
sand_core::inventory::submit! {
    EventDescriptor { owner: module_path!(), path: "respawned", id_override: None,
        make: || sand::component::IntoCommands::into_commands(Vec::<String>::new()),
        dispatch: EventDispatch::RespawnTick }
}
sand_core::inventory::submit! {
    EventDescriptor { owner: module_path!(), path: "leveled", id_override: None,
        make: || sand::component::IntoCommands::into_commands(Vec::<String>::new()),
        dispatch: EventDispatch::XpLevelUp }
}
fn registration() -> DatapackRegistration {
    let collision = COLLISION.lock().unwrap();
    match collision.as_ref() {
        None => DatapackRegistration::new(),
        Some((true, path)) => DatapackRegistration::new().component(McFunction::new(
            ResourceLocation::new("detectorcollision", path).unwrap(),
        )),
        Some((false, objective)) => DatapackRegistration::new().lifecycle(
            LifecycleContribution::load(format!("scoreboard objectives add {objective} dummy")),
        ),
    }
}
sand_core::inventory::submit! {
    ComponentFactory { owner: "detector_collision_fixture", make: registration }
}

#[test]
fn rejects_helper_paths_and_same_criterion_baseline_declarations() {
    let export = sand_core::try_export_components_json("detectorcollision").unwrap();
    let records: Vec<serde_json::Value> = serde_json::from_str(&export).unwrap();
    let load = records
        .iter()
        .find(|r| r["dir"] == "function" && r["path"] == "__sand_lifecycle_load")
        .unwrap()["content"]
        .as_str()
        .unwrap();
    let objectives = load
        .lines()
        .filter_map(|line| line.strip_prefix("scoreboard objectives add "))
        .map(|line| line.split_whitespace().next().unwrap());
    let helpers = [
        "__sand_death_check",
        "__sand_death_dispatch",
        "__sand_respawn_check",
        "__sand_respawn_dispatch",
        "__sand_xp_check",
        "__sand_xp_dispatch",
        "__sand_xp_init",
        "__sand_xp_observe",
    ];
    for (function, name) in helpers
        .into_iter()
        .map(|name| (true, name))
        .chain(objectives.map(|name| (false, name)))
    {
        *COLLISION.lock().unwrap() = Some((function, name.to_string()));
        let first = sand_core::try_export_components_json("detectorcollision")
            .unwrap_err()
            .to_string();
        let second = sand_core::try_export_components_json("detectorcollision")
            .unwrap_err()
            .to_string();
        assert_eq!(first, second);
        assert!(first.contains(name), "{first}");
        assert!(first.contains("collides"), "{first}");
    }
    *COLLISION.lock().unwrap() = None;
    assert_eq!(
        export,
        sand_core::try_export_components_json("detectorcollision").unwrap()
    );
}
