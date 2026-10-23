use sand::entity::EntityStateField;
use serde_json::Value;
use unified_state::{Progression, Supplies};

#[test]
fn join_inventory_snapshot_does_not_reset_persistent_progression() {
    let pack: Value =
        serde_json::from_str(&sand::advanced::try_export_pack_json("rpg", "26.2").unwrap())
            .unwrap();
    let functions = pack["components"].as_array().unwrap();
    let body = |path: &str| {
        functions
            .iter()
            .find(|record| record["dir"] == "function" && record["path"] == path)
            .unwrap()["content"]
            .as_str()
            .unwrap()
    };
    assert!(body("welcome_player").contains("run function rpg:count_diamonds"));
    // Numeric expressions and lifecycle initialization retain helper functions.
    // Inspect the complete reachable join path, rather than only its entry body.
    let mut pending = vec!["welcome_player".to_owned()];
    let mut visited = std::collections::BTreeSet::new();
    let mut reachable = String::new();
    while let Some(path) = pending.pop() {
        if !visited.insert(path.clone()) {
            continue;
        }
        let commands = body(&path);
        reachable.push_str(commands);
        reachable.push('\n');
        for command in commands.lines() {
            if let Some((_, call)) = command.split_once("function rpg:") {
                pending.push(call.split_whitespace().next().unwrap().to_owned());
            }
        }
    }
    let count = &reachable;
    assert!(count.contains("clear @s minecraft:diamond 0"), "{count}");
    let supplies = EntityStateField::objective(Supplies::diamonds);
    assert!(
        count.contains(&format!("scoreboard players operation @s {supplies} =")),
        "{count}"
    );
    let experience = EntityStateField::objective(Progression::experience);
    for command in reachable.lines().filter(|line| line.contains(&experience)) {
        // Both supported initialization forms preserve any seeded experience:
        // add zero, or set the default only when that score does not exist.
        let add_zero = command == format!("scoreboard players add @s {experience} 0");
        let initialize_missing = command.contains(&format!(
            "unless score @s {experience} matches -2147483648.. "
        )) && command
            .ends_with(&format!("run scoreboard players set @s {experience} 0"));
        assert!(add_zero || initialize_missing, "{command}");
    }
}
