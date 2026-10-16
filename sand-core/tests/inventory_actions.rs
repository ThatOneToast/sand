//! Cached slot writes retain typed stacks and locations across exports.
use sand::prelude::*;
use std::sync::OnceLock;

static BODY: OnceLock<Actions> = OnceLock::new();
fn body() -> Actions {
    BODY.get_or_init(|| {
        Target::players().each(|player| {
            let inventory = player.inventory();
            let reward = ItemStack::new(ItemId::minecraft("diamond").unwrap())
                .count(3)
                .component(sand::component::ItemComponent::custom_data_marker("reward"));
            mcfunction![
                inventory.offhand().set(reward).unwrap();
                inventory.hotbar(3).unwrap().clear().unwrap();
                when(inventory.mainhand().is_empty().unwrap()).then(
                    inventory.mainhand().replace_from(&inventory.offhand()).unwrap()
                );
                ItemLocation::block(BlockPos::here()).slot(0).unwrap()
                    .replace_from(&inventory.offhand()).unwrap();
            ]
        })
    })
    .clone()
}

sand_core::inventory::submit! {
    sand_core::FunctionDescriptor { owner: module_path!(),
        context: sand::advanced::compiler::ExecutionContext::Server,
        path: "reward",
        make: body,
    }
}

#[test]
fn cached_inventory_actions_export_typed_writes_and_guards_deterministically() {
    let _ = body();
    let first = sand_core::try_export_components_json("inventory_actions").unwrap();
    for _ in 0..2 {
        assert_eq!(
            first,
            sand_core::try_export_components_json("inventory_actions").unwrap()
        );
    }
    let records: Vec<serde_json::Value> = serde_json::from_str(&first).unwrap();
    let functions: Vec<_> = records
        .iter()
        .filter(|r| r["dir"] == "function")
        .map(|r| r["content"].as_str().unwrap())
        .collect();
    let text = functions.join("\n");
    assert!(
        text.contains("item replace entity @s weapon.offhand with minecraft:diamond["),
        "{text}"
    );
    assert!(text.contains("reward"));
    assert!(text.contains("] 3"));
    assert!(text.contains("item replace entity @s hotbar.3 with air"));
    let copy = " run item replace entity @s weapon.mainhand from entity @s weapon.offhand";
    let guards: Vec<_> = text.lines().filter(|line| line.ends_with(copy)).collect();
    assert_eq!(guards.len(), 1, "one guarded copy operation");
    let helper = guards[0]
        .strip_prefix("execute unless function inventory_actions:")
        .and_then(|line| line.strip_suffix(copy))
        .expect("copy must require an empty main hand");
    let predicate = records
        .iter()
        .find(|record| record["dir"] == "function" && record["path"] == helper)
        .expect("guard owns its exported item predicate");
    assert_eq!(
        predicate["content"].as_str().unwrap().trim(),
        "execute if items entity @s weapon.mainhand * run return 1\nreturn 0"
    );
    assert!(text.contains("item replace block ~ ~ ~ container.0 from entity @s weapon.offhand"));
    assert!(!text.contains("data modify entity"));
    assert!(!text.contains("__sand_local:"));
}
