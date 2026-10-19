//! Slot capability branches retain one runtime match across destructive writes.
use sand::prelude::*;
use std::sync::OnceLock;

static BODY: OnceLock<Actions> = OnceLock::new();
fn body() -> Actions {
    BODY.get_or_init(|| {
        Target::players().each(|player| {
            let inventory = player.inventory();
            let token = CustomItem::new("minecraft:paper")
                .custom_data("match_actions:token")
                .item_predicate()
                .unwrap();
            inventory
                .offhand()
                .when_matches(token, || {
                    mcfunction![
                        inventory.offhand().clear().unwrap();
                        inventory.mainhand().set(vanilla::Item::Diamond).unwrap();
                    ]
                })
                .unwrap()
        })
    })
    .clone()
}

sand_core::inventory::submit! {
    sand_core::FunctionDescriptor {
        owner: module_path!(),
        context: sand::advanced::compiler::ExecutionContext::Server,
        path: "consume_token", make: body,
    }
}

#[test]
fn slot_match_is_evaluated_once_before_the_entire_mutating_branch() {
    let _ = body();
    let first = sand_core::try_export_components_json("match_actions").unwrap();
    assert_eq!(
        first,
        sand_core::try_export_components_json("match_actions").unwrap()
    );
    let records: Vec<serde_json::Value> = serde_json::from_str(&first).unwrap();
    let functions: Vec<_> = records.iter().filter(|r| r["dir"] == "function").collect();
    let branch = functions
        .iter()
        .find(|r| {
            r["content"]
                .as_str()
                .unwrap()
                .contains("item replace entity @s weapon.offhand with air")
        })
        .unwrap();
    assert_eq!(
        branch["content"].as_str().unwrap().trim(),
        "item replace entity @s weapon.offhand with air\nitem replace entity @s weapon.mainhand with minecraft:diamond 1"
    );
    let call = format!(
        " run function match_actions:{}",
        branch["path"].as_str().unwrap()
    );
    let callers: Vec<_> = functions
        .iter()
        .flat_map(|r| r["content"].as_str().unwrap().lines())
        .filter(|line| line.ends_with(&call))
        .collect();
    assert_eq!(callers.len(), 1);
    assert!(
        callers[0].starts_with("execute if function match_actions:"),
        "{}",
        callers[0]
    );
    let text = functions
        .iter()
        .map(|r| r["content"].as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(text.matches("execute if items entity @s weapon.offhand minecraft:paper[minecraft:custom_data~{\"match_actions:token\":1b}]").count(), 1, "{text}");
    assert!(text.contains("execute as @a"), "{text}");
    assert!(!text.contains("__sand_local:"));
    assert!(
        !records
            .iter()
            .any(|r| r["dir"] == "tags/function" && r["path"] == "tick")
    );
}

#[test]
fn unsupported_slot_rejects_before_building_the_branch() {
    let slot = ItemLocation::entity(Target::current_player())
        .ender_chest(0)
        .unwrap();
    let mut called = false;
    let result = slot.when_matches(vanilla::Item::Diamond, || {
        called = true;
        Actions::default()
    });
    assert!(result.is_err());
    assert!(!called);
}
