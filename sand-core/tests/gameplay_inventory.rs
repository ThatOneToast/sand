use sand::entity::EntityText;
use sand::inventory::InventoryIndex;
use sand::prelude::*;
use sand::systems::{InventoryTransfer, Journal, Purchase};

fn slot(value: u8) -> InventoryIndex {
    InventoryIndex::new(value).unwrap()
}
#[datapack_component]
fn transfer() -> InventoryTransfer {
    InventoryTransfer::new("inventory_proof:move".parse().unwrap(), slot(0), slot(1))
        .on_success(|| cmd::say("MOVE_OK"))
        .on_rejected(|| cmd::say("MOVE_REJECTED"))
}
#[datapack_component]
fn foreign_transfer() -> InventoryTransfer {
    InventoryTransfer::new(
        "foreign_transfer:cross_namespace_move".parse().unwrap(),
        slot(4),
        slot(5),
    )
    .on_success(|| cmd::say("FOREIGN_MOVE_OK"))
    .on_rejected(|| cmd::say("FOREIGN_MOVE_REJECTED"))
}
#[datapack_component]
fn foreign_purchase() -> Purchase {
    Purchase::new(
        "foreign_purchase:cross_namespace_shop".parse().unwrap(),
        sand::component::ItemStack::new(ItemId::minecraft("emerald").unwrap()),
        slot(6),
    )
    .on_success(|| cmd::say("FOREIGN_PURCHASE_OK"))
    .on_rejected(|| cmd::say("FOREIGN_PURCHASE_REJECTED"))
}
#[datapack_component]
fn foreign_journal() -> Journal {
    Journal::new(
        "foreign_journal:cross_namespace_book".parse().unwrap(),
        "Bound lock",
        "Sand",
        slot(7),
    )
    .page(EntityText::new().literal("One pack owns this inventory lock."))
    .on_success(|| cmd::say("FOREIGN_JOURNAL_OK"))
    .on_rejected(|| cmd::say("FOREIGN_JOURNAL_REJECTED"))
}
#[function]
fn foreign_attempt() {
    foreign_transfer().transfer();
    foreign_purchase().buy();
    foreign_journal().refresh();
}
#[datapack_component]
fn purchase() -> Purchase {
    Purchase::new(
        "inventory_proof:shop".parse().unwrap(),
        sand::component::ItemStack::new(ItemId::minecraft("diamond").unwrap()),
        slot(2),
    )
    .item_cost(slot(3), ItemId::minecraft("emerald").unwrap(), 3)
    .on_success(|| cmd::say("PAYMENT_OK"))
    .on_rejected(|| cmd::say("PAYMENT_REJECTED"))
}
#[datapack_component]
fn component_purchase() -> Purchase {
    Purchase::new(
        "inventory_proof:marked_shop".parse().unwrap(),
        sand::component::ItemStack::new(ItemId::minecraft("diamond").unwrap()),
        slot(2),
    )
    .item_cost(
        slot(3),
        sand::predicate::ItemPredicate::new()
            .item(ItemId::minecraft("emerald").unwrap())
            .custom_data_key("currency"),
        3,
    )
    .on_success(|| cmd::say("COMPONENT_PAYMENT_OK"))
}
#[function]
fn buy_marked() {
    component_purchase().buy();
}
#[function]
fn move_stack() {
    transfer().transfer();
}
#[function]
fn buy() {
    purchase().buy();
}
fn body<'a>(pack: &'a serde_json::Value, path: &str) -> &'a str {
    pack["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["path"] == path)
        .unwrap()["content"]
        .as_str()
        .unwrap()
}
#[test]
fn inventory_workflows_preserve_slots_and_check_before_committing_on_all_targets() {
    for version in ["26.1", "26.2", "26.3"] {
        let json = sand::advanced::try_export_pack_json("inventory_proof", version).unwrap();
        assert_eq!(
            json,
            sand::advanced::try_export_pack_json("inventory_proof", version).unwrap()
        );
        let pack: serde_json::Value = serde_json::from_str(&json).unwrap();
        let move_body = body(&pack, "__sand_system/lifetime/move/inventory_transfer_move");
        assert!(
            move_body.starts_with("execute unless entity @s[type=minecraft:player] run return 0")
        );
        assert!(
            move_body.find("matches 1 run return 0").unwrap()
                < move_body.find("data get entity @s Inventory").unwrap()
        );
        assert!(move_body.contains("transfer.source.components.\"minecraft:max_stack_size\""));
        assert!(move_body.contains("transfer.source.Slot"));
        assert!(move_body.contains("transfer.destination.count"));
        assert!(
            move_body.find("#different").unwrap()
                < move_body.find("inventory_transfer_merge").unwrap()
        );
        let empty = body(
            &pack,
            "__sand_system/lifetime/move/inventory_transfer_empty",
        );
        assert!(
            empty.find("unless function").unwrap()
                < empty.find("inventory_transfer_finish").unwrap()
        );
        let finish = body(
            &pack,
            "__sand_system/lifetime/move/inventory_transfer_finish",
        );
        assert!(
            finish.find("#source_changed").unwrap() < finish.find("hotbar.0 with air").unwrap()
        );
        assert!(
            finish.find("#destination_changed").unwrap()
                < finish.find("hotbar.0 with air").unwrap()
        );
        let copy = body(&pack, "__sand_system/lifetime/move/inventory_transfer_copy");
        assert!(copy.contains("item replace entity @s hotbar.1 from entity @s hotbar.0"));
        let buy = body(&pack, "__sand_system/lifetime/shop/purchase_buy");
        let admission = buy.find("matches 3 run return").unwrap();
        let place = buy
            .find("unless function inventory_proof:__sand_system/lifetime/shop/purchase_place")
            .unwrap();
        let changed = buy.find("payment.after set from storage").unwrap();
        let remove = buy.find("hotbar.3 with air").unwrap();
        let expected = buy.find("payment.reward_expected set value").unwrap();
        assert!(admission < expected && expected < place && place < changed && changed < remove);
        let staged = buy
            .lines()
            .find(|line| line.contains("payment.reward_expected set value "))
            .unwrap()
            .split_once(" set value ")
            .unwrap()
            .1;
        assert_eq!(
            staged, "{count:1,id:\"minecraft:diamond\"}",
            "component-free canonical rewards must omit the native absent components map"
        );
        let compensate = body(
            &pack,
            "__sand_system/lifetime/shop/purchase_payment_changed",
        );
        let clear_reward = compensate.find("hotbar.2 with air").unwrap();
        assert!(compensate.find("payment.reward_actual.Slot").unwrap() < clear_reward);
        assert!(
            compensate
                .find("payment.reward_actual set from storage")
                .unwrap()
                < clear_reward
        );
        assert!(compensate.find("execute if score #reward_changed").unwrap() < clear_reward);
        assert!(compensate.contains("unless data storage"));
        assert!(compensate.contains("payment.reward_expected run return run function"));
        for path in [
            "move/inventory_transfer_success",
            "move/inventory_transfer_rejected",
            "shop/purchase_success",
            "shop/purchase_rejected",
        ] {
            let callback = body(&pack, &format!("__sand_system/lifetime/{path}"));
            assert!(
                callback.find("data remove storage").unwrap() < callback.find("#busy").unwrap()
            );
        }
    }
}
#[test]
fn inventory_lock_covers_all_resource_namespaces_and_isolates_exports() {
    for version in ["26.1", "26.2", "26.3"] {
        let mut locks = Vec::new();
        for namespace in ["inventory_proof", "independent_inventory"] {
            let pack: serde_json::Value = serde_json::from_str(
                &sand::advanced::try_export_pack_json(namespace, version).unwrap(),
            )
            .unwrap();
            let guard = |path: &str| {
                body(&pack, path)
                    .lines()
                    .find(|line| line.starts_with("execute if score #busy "))
                    .unwrap()
                    .split_whitespace()
                    .nth(4)
                    .unwrap()
                    .to_owned()
            };
            let lock = guard("__sand_system/lifetime/move/inventory_transfer_move");
            for path in [
                "__sand_system/lifetime/shop/purchase_buy",
                "__sand_system/lifetime/cross_namespace_move/inventory_transfer_move",
                "__sand_system/lifetime/cross_namespace_shop/purchase_buy",
                "__sand_system/lifetime/cross_namespace_book/journal_refresh",
            ] {
                assert_eq!(guard(path), lock, "{namespace} {version} {path}");
            }
            locks.push(lock);
        }
        assert_ne!(
            locks[0], locks[1],
            "separate pack exports must not share inventory locks"
        );
    }
}
#[test]
fn export_native_inventory_fixture_when_requested() {
    let Some(path) = std::env::var_os("SAND_INVENTORY_PROOF_PACK") else {
        return;
    };
    let version = std::env::var("SAND_INVENTORY_PROOF_VERSION").unwrap_or_else(|_| "26.2".into());
    let root = std::path::PathBuf::from(path);
    std::fs::create_dir_all(&root).unwrap();
    let pack: serde_json::Value = serde_json::from_str(
        &sand::advanced::try_export_pack_json("inventory_proof", &version).unwrap(),
    )
    .unwrap();
    let format = if version == "26.3" {
        121
    } else if version == "26.2" {
        107
    } else {
        101
    };
    std::fs::write(root.join("pack.mcmeta"), serde_json::json!({"pack":{"description":"Sand inventory proof", "min_format":format,"max_format":format}}).to_string()).unwrap();
    for record in pack["components"].as_array().unwrap() {
        let dir = record["dir"].as_str().unwrap();
        let extension = if dir == "function" {
            "mcfunction"
        } else {
            "json"
        };
        let path = root
            .join("data")
            .join(record["namespace"].as_str().unwrap())
            .join(dir)
            .join(format!(
                "{}.{}",
                record["path"].as_str().unwrap(),
                extension
            ));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let content = &record["content"];
        std::fs::write(
            path,
            content
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| content.to_string()),
        )
        .unwrap();
    }
}
