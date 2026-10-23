//! Public event/query/function handles share gameplay capabilities and State.
use sand::events::OnJoinEvent;
use sand::prelude::*;
use sand_core::serde_json::{self, Value};

#[derive(State)]
#[state(namespace = "handles", scope = player)]
#[allow(dead_code)]
struct Supplies {
    diamonds: Score,
    calls: Score,
}

fn observe(player: EntityContext<PlayerKind>) -> Actions {
    mcfunction![
        Supplies::on(player).diamonds.set(player.inventory().count(vanilla::Item::Diamond));
        Supplies::on(player).calls.add(1);
        player.inventory().offhand().clear().unwrap();
    ]
}

#[function("observe", context = player)]
fn observe_player(player: EntityContext<PlayerKind>) {
    observe(player);
}

#[function("invoke", context = player)]
fn invoke(player: EntityContext<PlayerKind>) {
    player.call(observe_player);
}

#[function]
fn query_players() {
    Target::players().each(|player| player.call(observe_player));
}

#[on_event]
fn joined(event: Event<OnJoinEvent>) {
    event.player().call(observe_player);
}

#[on_event]
fn legacy_joined(event: OnJoinEvent) {
    observe(event.player());
}

struct Damaged;
impl AdvancementEvent for Damaged {
    type Trigger = AdvancementTrigger;
    fn trigger() -> Self::Trigger {
        AdvancementTrigger::EntityHurtPlayer {
            entity: None,
            damage: None,
        }
    }
}
impl DamageAdvancementEvent for Damaged {}

#[on_event]
fn damaged(event: DamageEvent<Damaged>) {
    observe(event.player());
}

// Context-independent functions also inherit the current executor and frame.
#[function]
fn mark_position() {
    WorldContext::current()
        .block(BlockPos::new(
            Coord::local(),
            Coord::local(),
            Coord::local_n(1),
        ))
        .replace(vanilla::Block::GoldBlock);
}

#[function("player_position", context = player)]
fn player_position(player: EntityContext<PlayerKind>) {
    player.call(mark_position);
}

#[function("position", context = player)]
fn position(player: EntityContext<PlayerKind>) {
    player.call(player_position);
}

// Default is deliberately an authoring assumption. The retained call's runtime
// guard prevents a player-only callee from running with server/nonplayer @s.
#[function]
fn assumed_context() {
    EntityContext::<PlayerKind>::default().call(observe_player);
}

fn snapshot(namespace: &str) -> Value {
    serde_json::from_str(&sand::advanced::try_export_pack_json(namespace, "26.2").unwrap()).unwrap()
}
fn body<'a>(pack: &'a Value, path: &str) -> &'a str {
    pack["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["dir"] == "function" && record["path"] == path)
        .unwrap()["content"]
        .as_str()
        .unwrap()
}

#[test]
fn canonical_handles_retain_calls_inventory_state_and_namespace_isolation() {
    let first_json = sand::advanced::try_export_pack_json("handles", "26.2").unwrap();
    let other_json = sand::advanced::try_export_pack_json("other_handles", "26.2").unwrap();
    assert_eq!(
        first_json,
        sand::advanced::try_export_pack_json("handles", "26.2").unwrap()
    );
    assert_eq!(
        other_json,
        sand::advanced::try_export_pack_json("other_handles", "26.2").unwrap()
    );
    let first: Value = serde_json::from_str(&first_json).unwrap();
    let other: Value = serde_json::from_str(&other_json).unwrap();
    for (pack, namespace) in [(&first, "handles"), (&other, "other_handles")] {
        let call =
            format!("execute if entity @s[type=minecraft:player] run function {namespace}:observe");
        assert!(body(pack, "invoke").ends_with(&call));
        assert_eq!(body(pack, "assumed_context"), call);
        assert!(body(pack, "position").ends_with(&format!(
            "execute if entity @s[type=minecraft:player] run function {namespace}:player_position"
        )));
        assert!(
            body(pack, "player_position").ends_with(&format!("function {namespace}:mark_position"))
        );
        assert_eq!(
            body(pack, "mark_position"),
            "setblock ^ ^ ^1 minecraft:gold_block"
        );
        let text = pack.to_string();
        assert!(text.contains("clear @s minecraft:diamond 0"));
        assert!(text.contains("item replace entity @s weapon.offhand with air"));
        assert!(text.contains("execute as @a"));
        assert!(!text.contains("__sand_local:"));
        assert!(!text.contains(if namespace == "handles" {
            "other_handles:"
        } else {
            "function handles:"
        }));
    }
    // The macro's authored parameter never becomes a Rust/runtime argument.
    let _: fn() -> Actions = observe_player;
    let _: EntityContext<PlayerKind> = Event::<OnJoinEvent>::context().player();
    let _: EntityContext<PlayerKind> = OnJoinEvent.player();
    let _: EntityContext<PlayerKind> = DamageEvent::<Damaged>::context().player();
}

#[test]
#[ignore = "used by the vanilla two-player harness"]
fn dump_player_context_pack() {
    let pack = snapshot("handles");
    std::fs::write(
        std::env::var("SAND_PLAYER_CONTEXT_DUMP_PATH").unwrap(),
        serde_json::to_string_pretty(&serde_json::json!({
            "components": pack["components"],
            "objectives": {
                "calls": sand::entity::EntityStateField::objective(Supplies::calls),
                "diamonds": sand::entity::EntityStateField::objective(Supplies::diamonds),
            },
        }))
        .unwrap(),
    )
    .unwrap();
}
