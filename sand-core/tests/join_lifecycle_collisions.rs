//! Private join resources cannot be claimed by user registrations.
use sand::ResourceLocation;
use sand::component::McFunction;
use sand::registration::LifecycleContribution;
use sand_core::{ComponentFactory, DatapackRegistration, EventDescriptor, EventDispatch};
use std::sync::Mutex;

static COLLISION: Mutex<Option<(bool, String)>> = Mutex::new(None);

sand_core::inventory::submit! {
    EventDescriptor {
        path: "joined", id_override: None,
        make: || sand::component::IntoCommands::into_commands(Vec::<String>::new()),
        dispatch: EventDispatch::JoinTick,
    }
}
fn registration() -> DatapackRegistration {
    let collision = COLLISION.lock().unwrap();
    match collision.as_ref() {
        None => DatapackRegistration::new(),
        Some((true, path)) => DatapackRegistration::new().component(McFunction::new(
            ResourceLocation::new("joincollision", path).unwrap(),
        )),
        Some((false, objective)) => DatapackRegistration::new().lifecycle(
            LifecycleContribution::load(format!("scoreboard objectives add {objective} dummy")),
        ),
    }
}
sand_core::inventory::submit! {
    ComponentFactory { owner: "join_collision_fixture", make: registration }
}

#[test]
fn rejects_helper_paths_and_same_criterion_baseline_declarations() {
    let export = sand_core::try_export_components_json("joincollision").unwrap();
    let records: Vec<serde_json::Value> = serde_json::from_str(&export).unwrap();
    let load = records
        .iter()
        .find(|r| r["dir"] == "function" && r["path"] == "__sand_lifecycle_load")
        .unwrap()["content"]
        .as_str()
        .unwrap();
    let baseline = load
        .lines()
        .find_map(|line| line.strip_prefix("scoreboard players reset * "))
        .unwrap();
    for (function, name) in [
        (true, "__sand_join_check"),
        (true, "__sand_join_dispatch"),
        (false, baseline),
    ] {
        *COLLISION.lock().unwrap() = Some((function, name.to_string()));
        let first = sand_core::try_export_components_json("joincollision")
            .unwrap_err()
            .to_string();
        let second = sand_core::try_export_components_json("joincollision")
            .unwrap_err()
            .to_string();
        assert_eq!(first, second);
        assert!(first.contains(name), "{first}");
        assert!(first.contains("collides"), "{first}");
    }
    *COLLISION.lock().unwrap() = None;
    assert_eq!(
        export,
        sand_core::try_export_components_json("joincollision").unwrap()
    );
}
