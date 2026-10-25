//! Public-facade live replacement fixture; native setup/oracles live in Python.
use sand::events::OnJoinEvent;
use sand::prelude::*;
use sand::registration::{ComponentContent, DatapackComponent};
use sand_core::serde_json::{self, Value};
use std::sync::OnceLock;

#[derive(State)]
#[state(namespace = "live_copy", scope = player)]
#[allow(dead_code)]
struct CopyState {
    enabled: Flag,
    joined: Score,
    diamonds: Score,
}

#[on_event]
fn joined(event: Event<OnJoinEvent>) {
    let player = event.player();
    player
        .inventory()
        .offhand()
        .replace_from(&player.inventory().mainhand())
        .unwrap();
    CopyState::on(player).joined.add(1);
}

#[function("copy", context = player)]
fn copy(player: EntityContext<PlayerKind>) {
    player
        .inventory()
        .offhand()
        .replace_from(&player.inventory().mainhand())
        .unwrap();
}

#[function("guarded", context = player)]
fn guarded(player: EntityContext<PlayerKind>) {
    when(CopyState::on(player).enabled.is_enabled()).then(
        player
            .inventory()
            .offhand()
            .replace_from(&player.inventory().mainhand())
            .unwrap(),
    );
    CopyState::on(player)
        .diamonds
        .set(player.inventory().count(vanilla::Item::Diamond));
}

#[function]
fn query_players() {
    Target::players().each(|player| player.call(guarded));
}

#[function("scoped", context = player)]
fn scoped(player: EntityContext<PlayerKind>) {
    EntityScope::bind(&player, |owner| {
        Target::players().tag("copy_other").each(|other| {
            mcfunction![
                other.inventory().offhand().replace_from(&owner.inventory().mainhand()).unwrap();
                owner.inventory().offhand().replace_from(&other.inventory().mainhand()).unwrap();
            ]
        })
    });
}

#[function]
fn zombie() {
    let zombie = EntityContext::<ZombieKind>::default();
    zombie
        .equipment()
        .slot(EquipmentSlot::Offhand)
        .unwrap()
        .replace_from(&zombie.equipment().slot(EquipmentSlot::Mainhand).unwrap())
        .unwrap();
}

#[function]
fn to_block() {
    ItemLocation::block(BlockPos::here())
        .slot(0)
        .unwrap()
        .replace_from(&ItemLocation::entity(Target::current_player()).mainhand())
        .unwrap();
}

#[function]
fn from_block() {
    ItemLocation::entity(Target::current_player())
        .offhand()
        .replace_from(&ItemLocation::block(BlockPos::here()).slot(0).unwrap())
        .unwrap();
}

#[function]
fn invalid_block_source() {
    ItemLocation::entity(Target::current_player())
        .offhand()
        .replace_from(&ItemLocation::block(BlockPos::here()).slot(53).unwrap())
        .unwrap();
}

#[function]
fn invalid_block_destination() {
    ItemLocation::block(BlockPos::here())
        .slot(53)
        .unwrap()
        .replace_from(&ItemLocation::entity(Target::current_player()).mainhand())
        .unwrap();
}

#[function]
fn block_to_block() {
    ItemLocation::block(BlockPos::new(Coord::rel_n(1), Coord::rel(), Coord::rel()))
        .slot(0)
        .unwrap()
        .replace_from(&ItemLocation::block(BlockPos::here()).slot(0).unwrap())
        .unwrap();
}

#[function]
fn frame() {
    ItemLocation::block(BlockPos::new(
        Coord::local(),
        Coord::local(),
        Coord::local_n(1),
    ))
    .slot(0)
    .unwrap()
    .replace_from(&ItemLocation::entity(Target::current_player()).mainhand())
    .unwrap();
}

#[function]
fn self_copy() {
    let hand = ItemLocation::entity(Target::current_player()).mainhand();
    hand.replace_from(&hand).unwrap();
}

// The broad destination intentionally includes entities whose slot setter rejects
// hotbar.0. This is a runtime acceptance proof, not a new entity capability.
#[function]
fn plural() {
    ItemLocation::entity(Target::entities().tag("copy_destination"))
        .hotbar(0)
        .unwrap()
        .replace_from(&ItemLocation::entity(Target::current_player()).mainhand())
        .unwrap();
}

#[function]
fn missing_source() {
    ItemLocation::entity(Target::current_player())
        .offhand()
        .replace_from(
            &ItemLocation::entity(Target::entities().tag("copy_absent").limit(1).unwrap())
                .mainhand(),
        )
        .unwrap();
}

#[function]
fn incompatible_source() {
    ItemLocation::entity(Target::current_player())
        .offhand()
        .replace_from(
            &ItemLocation::entity(Target::entities().tag("copy_marker").limit(1).unwrap())
                .hotbar(0)
                .unwrap(),
        )
        .unwrap();
}

#[function]
fn incompatible_destination() {
    ItemLocation::entity(Target::entities().tag("copy_marker"))
        .hotbar(0)
        .unwrap()
        .replace_from(&ItemLocation::entity(Target::current_player()).mainhand())
        .unwrap();
}

fn cached() -> Actions {
    static BODY: OnceLock<Actions> = OnceLock::new();
    BODY.get_or_init(|| {
        let inventory = EntityContext::<PlayerKind>::default().inventory();
        when(
            inventory
                .mainhand()
                .matches(vanilla::Item::Diamond)
                .unwrap(),
        )
        .then(
            inventory
                .offhand()
                .replace_from(&inventory.mainhand())
                .unwrap(),
        )
    })
    .clone()
}

#[function]
fn cached_copy() {
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
fn facade_copy_is_native_and_cached_exports_are_isolated_a_b_a() {
    let _ = cached();
    let a = sand::advanced::try_export_pack_json("live_copy", "26.2").unwrap();
    let b = sand::advanced::try_export_pack_json("other_copy", "26.2").unwrap();
    assert_eq!(
        a,
        sand::advanced::try_export_pack_json("live_copy", "26.2").unwrap()
    );
    for json in [&a, &b] {
        let pack: Value = serde_json::from_str(json).unwrap();
        for (path, expected) in [
            (
                "zombie",
                "item replace entity @s weapon.offhand from entity @s weapon.mainhand",
            ),
            (
                "to_block",
                "item replace block ~ ~ ~ container.0 from entity @s weapon.mainhand",
            ),
            (
                "from_block",
                "item replace entity @s weapon.offhand from block ~ ~ ~ container.0",
            ),
            (
                "block_to_block",
                "item replace block ~1 ~ ~ container.0 from block ~ ~ ~ container.0",
            ),
            (
                "frame",
                "item replace block ^ ^ ^1 container.0 from entity @s weapon.mainhand",
            ),
            (
                "self_copy",
                "item replace entity @s weapon.mainhand from entity @s weapon.mainhand",
            ),
            (
                "plural",
                "item replace entity @e[tag=copy_destination] hotbar.0 from entity @s weapon.mainhand",
            ),
        ] {
            assert_eq!(body(&pack, path), expected, "{path}");
        }
        assert!(json.contains("execute as @a"));
        assert!(
            json.contains("from entity @e[tag="),
            "scoped source retains owner"
        );
        assert!(
            json.contains("item replace entity @e[tag="),
            "scoped destination retains owner"
        );
        assert!(!json.contains("__sand_local:"));
        assert!(!json.contains("data modify entity"));
        assert!(!json.contains("with air"), "copy must not append a clear");
    }
    assert!(!b.contains("function live_copy:"));
    assert!(!a.contains("other_copy:"));
}

fn render(owner: &str, action: Actions) -> Result<String, String> {
    let function = sand::component::McFunction::new(owner.parse().unwrap()).command(action);
    match function.try_content().map_err(|e| e.to_string())? {
        ComponentContent::Text(text) => Ok(text),
        _ => panic!("expected function"),
    }
}

#[test]
fn endpoint_validation_and_cached_owner_diagnostics() {
    let inventory = EntityContext::<PlayerKind>::default().inventory();
    let valid = inventory.offhand();
    let ender = inventory.ender_chest(0).unwrap();
    assert!(valid.replace_from(&ender).is_err());
    assert!(ender.replace_from(&valid).is_err());
    let invalid_sources = [
        ItemLocation::entity(Target::players()).mainhand(),
        ItemLocation::entity(Target::players().tag("invalid tag").limit(1).unwrap()).mainhand(),
        ItemLocation::block(BlockPos::new(Coord::local(), Coord::rel(), Coord::local()))
            .slot(0)
            .unwrap(),
    ];
    for source in invalid_sources {
        let action = valid.replace_from(&source).unwrap();
        let a = render("first:copy", action.clone()).unwrap_err();
        let b = render("second:copy", action.clone()).unwrap_err();
        assert_eq!(a, render("first:copy", action).unwrap_err());
        assert!(
            a.contains("first:copy") && a.contains("inventory copy"),
            "{a}"
        );
        assert!(
            b.contains("second:copy") && !b.contains("first:copy"),
            "{b}"
        );
    }
    // Destination validation must also remain owned until lowering.
    let destination = ItemLocation::entity(Target::players().tag("invalid tag")).offhand();
    let error = render(
        "destination:copy",
        destination.replace_from(&valid).unwrap(),
    )
    .unwrap_err();
    assert!(error.contains("destination:copy"), "{error}");
}

#[test]
#[ignore = "maintained Minecraft 26.2 two-player fixture export"]
fn dump_live_item_copy_pack() {
    let pack: Value =
        serde_json::from_str(&sand::advanced::try_export_pack_json("live_copy", "26.2").unwrap())
            .unwrap();
    std::fs::write(
        std::env::var("SAND_LIVE_ITEM_COPY_DUMP_PATH").unwrap(),
        serde_json::to_string_pretty(&serde_json::json!({
            "components": pack["components"],
            "objectives": {
                "enabled": sand::entity::EntityStateField::objective(CopyState::enabled),
                "joined": sand::entity::EntityStateField::objective(CopyState::joined),
                "diamonds": sand::entity::EntityStateField::objective(CopyState::diamonds),
            },
        }))
        .unwrap(),
    )
    .unwrap();
}
