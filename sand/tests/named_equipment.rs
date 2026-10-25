//! Named equipment slots retain canonical item ownership and cached actions.
use sand::prelude::*;
use sand_core::serde_json::{self, Value};
use std::sync::OnceLock;

fn locations<K: EquipmentEntityKind>(equipment: &EntityEquipmentHandle<K>) -> [ItemLocation; 6] {
    [
        equipment.mainhand(),
        equipment.offhand(),
        equipment.helmet(),
        equipment.chestplate(),
        equipment.leggings(),
        equipment.boots(),
    ]
}

fn equip<K: EquipmentEntityKind>(equipment: &EntityEquipmentHandle<K>) -> Actions {
    let items = [
        vanilla::Item::DiamondSword,
        vanilla::Item::Shield,
        vanilla::Item::DiamondHelmet,
        vanilla::Item::DiamondChestplate,
        vanilla::Item::DiamondLeggings,
        vanilla::Item::DiamondBoots,
    ];
    let mut actions = Actions::default();
    for (location, item) in locations(equipment).into_iter().zip(items) {
        actions.extend(location.set(item).unwrap());
    }
    actions
}

fn cached() -> Actions {
    static ACTIONS: OnceLock<Actions> = OnceLock::new();
    ACTIONS
        .get_or_init(|| equip(&EntityContext::<PlayerKind>::default().equipment()))
        .clone()
}

#[function("equip", context = player)]
fn equip_player(player: EntityContext<PlayerKind>) {
    when(
        player
            .equipment()
            .offhand()
            .matches(vanilla::Item::Stick)
            .unwrap(),
    )
    .then(cached());
}

#[function]
fn nonplayer() {
    equip(&EntityContext::<ZombieKind>::default().equipment());
}

#[function("scoped", context = player)]
fn scoped(player: EntityContext<PlayerKind>) {
    EntityScope::bind(&player, |bound| {
        Target::players().tag("equipment_other").each(|other| {
            mcfunction![
                bound.equipment().boots().set(vanilla::Item::GoldenBoots).unwrap();
                other.equipment().helmet().clear().unwrap();
            ]
        })
    });
}

#[function]
fn cached_equipment() {
    cached();
}

fn body<'a>(pack: &'a Value, path: &str) -> &'a str {
    pack["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["dir"] == "function" && r["path"] == path)
        .unwrap()["content"]
        .as_str()
        .unwrap()
}

#[test]
fn all_named_slots_preserve_canonical_kind_paths_and_runtime_slot_validation() {
    fn check<K: EquipmentEntityKind>(player: bool) {
        let equipment = EntityContext::<K>::default().equipment();
        let slots = [
            EquipmentSlot::Mainhand,
            EquipmentSlot::Offhand,
            EquipmentSlot::Head,
            EquipmentSlot::Chest,
            EquipmentSlot::Legs,
            EquipmentSlot::Feet,
        ];
        let paths = [
            if player {
                "SelectedItem"
            } else {
                "equipment.mainhand"
            },
            "equipment.offhand",
            "equipment.head",
            "equipment.chest",
            "equipment.legs",
            "equipment.feet",
        ];
        for ((location, slot), path) in locations(&equipment).into_iter().zip(slots).zip(paths) {
            assert_eq!(location, equipment.slot(slot).unwrap());
            assert_eq!(
                location.nbt().get().to_string(),
                format!("data get entity @s {path}")
            );
        }
        assert!(equipment.slot(EquipmentSlot::Body).is_err());
    }
    check::<PlayerKind>(true);
    check::<ZombieKind>(false);
}

#[test]
fn cached_equipment_exports_exact_commands_and_scoped_ownership_a_b_a() {
    let _ = cached();
    let a = sand::advanced::try_export_pack_json("equipment", "26.2").unwrap();
    let b = sand::advanced::try_export_pack_json("other", "26.2").unwrap();
    assert_eq!(
        a,
        sand::advanced::try_export_pack_json("equipment", "26.2").unwrap()
    );
    let expected = [
        "item replace entity @s weapon.mainhand with minecraft:diamond_sword 1",
        "item replace entity @s weapon.offhand with minecraft:shield 1",
        "item replace entity @s armor.head with minecraft:diamond_helmet 1",
        "item replace entity @s armor.chest with minecraft:diamond_chestplate 1",
        "item replace entity @s armor.legs with minecraft:diamond_leggings 1",
        "item replace entity @s armor.feet with minecraft:diamond_boots 1",
    ]
    .join("\n");
    for json in [&a, &b] {
        let pack: Value = serde_json::from_str(json).unwrap();
        assert_eq!(body(&pack, "nonplayer"), expected);
        assert_eq!(body(&pack, "cached_equipment"), expected);
        assert!(json.contains("if items entity @s weapon.offhand minecraft:stick"));
        let text = pack["components"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|r| r["content"].as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let scoped = text
            .lines()
            .find(|line| line.contains("armor.feet with minecraft:golden_boots"))
            .unwrap();
        assert!(
            scoped.starts_with("item replace entity @e[tag="),
            "{scoped}"
        );
        assert!(scoped.contains(",limit=1] armor.feet"), "{scoped}");
        assert!(text.contains("item replace entity @s armor.head with air"));
        assert!(!json.contains("__sand_local:"));
    }
    assert!(!b.contains("function equipment:"));
}

#[test]
#[ignore = "maintained two-player and nonplayer native equipment fixture"]
fn dump_named_equipment_pack() {
    let pack: Value =
        serde_json::from_str(&sand::advanced::try_export_pack_json("equipment", "26.2").unwrap())
            .unwrap();
    std::fs::write(
        std::env::var("SAND_EQUIPMENT_DUMP_PATH").unwrap(),
        serde_json::to_string_pretty(&pack).unwrap(),
    )
    .unwrap();
}
