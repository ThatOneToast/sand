//! Typed item queries share native lowering and own generated tag resources.
use sand::prelude::*;
use std::sync::OnceLock;

#[expect(
    dead_code,
    reason = "State fields are accessed through generated bindings."
)]
#[derive(State)]
#[state(namespace = "inventory_predicates", scope = player)]
struct Counts {
    matches: Score,
}

static BODY: OnceLock<Actions> = OnceLock::new();
fn body() -> Actions {
    BODY.get_or_init(|| Target::players().each(|player| {
        let predicate = ItemPredicate::id(vanilla::Item::Diamond)
            .item(vanilla::Item::Emerald).item(vanilla::Item::Diamond)
            .count_min(2).custom_data_key("reward");
        let wand = CustomItem::new("minecraft:stick").custom_data("pack:wand")
            .item_predicate().unwrap();
        let inventory = player.inventory();
        mcfunction![
            Counts::on(*player).matches.set(inventory.count(predicate.clone()));
            when(inventory.offhand().matches(&predicate).unwrap()).then_all(["say reward equipped"]);
            when(!inventory.offhand().matches(&predicate).unwrap()).then_all(["say reward absent"]);
            when(inventory.contains(&predicate)).then_all(["say reward present"]);
            when(inventory.contains(wand)).then_all(["say wand present"]);
            when(inventory.mainhand().is_empty().unwrap()).then_all(["say mainhand empty"]);
        ]
    })).clone()
}

sand_core::inventory::submit! {
    sand_core::FunctionDescriptor {
        context: sand::advanced::compiler::ExecutionContext::Server,
        path: "queries", make: body,
    }
}

#[test]
fn cached_predicates_own_shared_tags_and_preserve_native_constraints() {
    let _ = body();
    let first = sand_core::try_export_components_json("inventory_predicates").unwrap();
    for _ in 0..2 {
        assert_eq!(
            first,
            sand_core::try_export_components_json("inventory_predicates").unwrap()
        );
    }
    let records: Vec<serde_json::Value> = serde_json::from_str(&first).unwrap();
    let tags: Vec<_> = records.iter().filter(|r| r["dir"] == "tags/item").collect();
    assert_eq!(
        tags.len(),
        1,
        "the slot and numeric queries must share one item tag"
    );
    let tag: serde_json::Value =
        serde_json::from_str(tags[0]["content"].as_str().unwrap()).unwrap();
    assert_eq!(
        tag["values"],
        serde_json::json!(["minecraft:diamond", "minecraft:emerald"])
    );
    assert_eq!(tag["replace"], true);
    let argument = format!(
        "#inventory_predicates:{}[minecraft:count={{\"min\":2}},minecraft:custom_data~{{reward:1b}}]",
        tags[0]["path"].as_str().unwrap()
    );
    let functions = records
        .iter()
        .filter(|r| r["dir"] == "function")
        .map(|r| r["content"].as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(functions.contains(&format!("clear @s {argument} 0")));
    assert!(functions.contains(&format!(
        "if items entity @s weapon.offhand {argument} run return 1"
    )));
    assert!(
        functions.contains("clear @s minecraft:stick[minecraft:custom_data~{\"pack:wand\":1b}] 0")
    );
    // Follow payload ownership rather than merely finding branch messages:
    // cached positive/negative guards must call the same exported predicate.
    let branch_guard = |payload: &str, polarity: &str| {
        let branch = records
            .iter()
            .find(|record| {
                record["dir"] == "function"
                    && record["content"]
                        .as_str()
                        .unwrap()
                        .lines()
                        .any(|line| line == payload)
            })
            .expect("branch payload function");
        let call = format!(
            " run function inventory_predicates:{}",
            branch["path"].as_str().unwrap()
        );
        let callers: Vec<_> = functions
            .lines()
            .filter(|line| line.ends_with(&call))
            .collect();
        assert_eq!(callers.len(), 1, "one guarded caller for {payload}");
        callers[0]
            .strip_prefix(&format!("execute {polarity} function "))
            .and_then(|line| line.strip_suffix(&call))
            .expect("correct guard polarity")
            .to_owned()
    };
    assert_eq!(
        branch_guard("say reward equipped", "if"),
        branch_guard("say reward absent", "unless")
    );
    let empty_helper = branch_guard("say mainhand empty", "unless");
    let empty_path = empty_helper.strip_prefix("inventory_predicates:").unwrap();
    let empty_record = records
        .iter()
        .find(|record| record["dir"] == "function" && record["path"] == empty_path)
        .unwrap();
    assert_eq!(
        empty_record["content"].as_str().unwrap().trim(),
        "execute if items entity @s weapon.mainhand * run return 1\nreturn 0"
    );
    assert!(!functions.contains("__sand_local:"));

    let other = sand_core::try_export_components_json("other_inventory").unwrap();
    assert!(!other.contains("#inventory_predicates:"));
    assert!(other.contains("#other_inventory:sand/items/"));
}

#[test]
fn unsupported_profile_fails_without_poisoning_the_next_export() {
    let caps = sand::version::VersionCaps::all_disabled();
    let error =
        sand_core::try_export_components_for_version("inventory_predicates", &caps, "999.0", true)
            .unwrap_err();
    assert!(error.to_string().contains("999.0"));
    let records = sand_core::try_export_components("inventory_predicates").unwrap();
    assert_eq!(records.iter().filter(|r| r.dir == "tags/item").count(), 1);
}
