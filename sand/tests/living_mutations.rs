//! Native living actions compose through the same facade as State and Conditions.
use sand::events::OnJoinEvent;
use sand::prelude::*;
use sand::registration::{ComponentContent, DatapackComponent};
use sand_core::serde_json::{self, Value};
use std::sync::OnceLock;

#[derive(State)]
#[state(namespace = "living", scope = player)]
#[allow(dead_code)]
struct Combat {
    enabled: Flag,
    applications: Score,
}

fn apply(player: EntityContext<PlayerKind>) -> Actions {
    let state = Combat::on(player);
    mcfunction![
        when(state.enabled.is_enabled().and(player.living().has_effect(EffectId::Speed)))
            .then(mcfunction![
                player.living().damage(DamageAmount::hearts(1.0), DamageKind::Generic);
                player.living().give_effect(EffectId::Glowing).seconds(30).amplifier(0).particles(false);
                player.living().clear_effect(EffectId::Speed);
                state.applications.add(1);
            ]);
    ]
}

#[function("apply", context = player)]
fn apply_player(player: EntityContext<PlayerKind>) {
    apply(player);
}

#[on_event]
fn joined(event: Event<OnJoinEvent>) {
    event
        .player()
        .living()
        .give_effect(EffectId::Speed)
        .seconds(60);
}

#[function]
fn query_players() {
    Target::players().each(|player| apply(*player));
}

#[function("clear", context = player)]
fn clear(player: EntityContext<PlayerKind>) {
    when(player.living().has_effect(EffectId::Glowing)).then(player.living().clear_effects());
}

#[function]
fn nonplayer() {
    let zombie = EntityContext::<ZombieKind>::default();
    zombie.living().damage(2.0, DamageKind::Magic);
    zombie.living().give_effect(EffectId::Glowing).infinite();
}

#[function("scoped", context = player)]
fn scoped(player: EntityContext<PlayerKind>) {
    EntityScope::bind(&player, |bound| {
        Target::players().tag("living_other").each(|other| {
            mcfunction![
                bound.living().give_effect(EffectId::Glowing).seconds(20);
                bound.living().damage(3.0, DamageKind::Generic);
                other.living().clear_effects();
            ]
        })
    });
}

#[function("frame", context = player)]
fn frame(player: EntityContext<PlayerKind>) {
    player.living().damage(0.0, DamageKind::Magic);
    player.living().give_effect(EffectId::Speed).seconds(20);
    player.living().clear_effect(EffectId::Speed);
    WorldContext::current()
        .block(BlockPos::new(
            Coord::local(),
            Coord::local(),
            Coord::local_n(1),
        ))
        .replace(vanilla::Block::GoldBlock);
}

fn cached() -> Actions {
    static BODY: OnceLock<Actions> = OnceLock::new();
    BODY.get_or_init(|| {
        let player = EntityContext::<PlayerKind>::default();
        mcfunction![
            Execute::new().if_entity(Target::current_player()).run_actions(player.living().give_effect(EffectId::Speed).seconds(10));
            when(player.living().has_effect(EffectId::Speed)).then(player.living().clear_effect(EffectId::Speed));
            player.living().damage(0.0, DamageKind::Magic);
            player.living().clear_effects();
        ]
    }).clone()
}

#[function]
fn cached_actions() {
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
fn facade_actions_export_exact_native_commands_and_cached_namespace_isolation() {
    let _ = cached();
    let a = sand::advanced::try_export_pack_json("living", "26.2").unwrap();
    let b = sand::advanced::try_export_pack_json("other", "26.2").unwrap();
    assert_eq!(
        a,
        sand::advanced::try_export_pack_json("living", "26.2").unwrap()
    );
    for json in [&a, &b] {
        let pack: Value = serde_json::from_str(json).unwrap();
        assert_eq!(
            body(&pack, "nonplayer"),
            "damage @s 2 minecraft:magic\neffect give @s minecraft:glowing infinite"
        );
        assert!(body(&pack, "clear").contains("run effect clear @s"));
        assert!(body(&pack, "frame").ends_with("damage @s 0 minecraft:magic\neffect give @s minecraft:speed 20\neffect clear @s minecraft:speed\nsetblock ^ ^ ^1 minecraft:gold_block"));
        assert!(
            body(&pack, "cached_actions")
                .contains("execute if entity @s run effect give @s minecraft:speed 10")
        );
        assert!(json.contains("damage @s 2 minecraft:generic"));
        assert!(json.contains("effect give @s minecraft:glowing 30 0 true"));
        assert!(json.contains("effect clear @s minecraft:speed"));
        assert!(json.contains("effect give @e[tag="));
        assert!(json.contains("damage @e[tag="));
        assert!(!json.contains("__sand_local:"));
        assert!(!json.contains("sand/observation"));
    }
    assert!(!b.contains("function living:"));
}

fn render(action: impl sand::component::IntoCommands) -> Result<String, String> {
    let function =
        sand::component::McFunction::new("diagnostic:mutation".parse().unwrap()).command(action);
    match function.try_content().map_err(|e| e.to_string())? {
        ComponentContent::Text(text) => Ok(text),
        _ => panic!("expected function text"),
    }
}

#[test]
fn invalid_cached_actions_fail_at_each_export_with_owner_and_field() {
    let living = EntityContext::<PlayerKind>::default().living();
    for amount in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let action = living.damage(amount, DamageKind::Magic);
        for _ in 0..3 {
            let error = render(action.clone()).unwrap_err();
            assert!(
                error.contains("diagnostic:mutation") && error.contains("amount"),
                "{error}"
            );
        }
    }
    for ticks in [0, 1, 21] {
        let action = living
            .give_effect(EffectId::Speed)
            .duration(Ticks::new(ticks));
        for _ in 0..3 {
            let error = render(action.clone()).unwrap_err();
            assert!(
                error.contains("diagnostic:mutation") && error.contains("DURATION"),
                "{error}"
            );
        }
    }
    for seconds in [0, 1_000_001] {
        let action = living.give_effect(EffectId::Speed).seconds(seconds);
        for _ in 0..3 {
            let error = render(action.clone()).unwrap_err();
            assert!(
                error.contains("diagnostic:mutation") && error.contains("DURATION"),
                "{error}"
            );
        }
    }
    let error = render(cmd::effect_give(
        Target::players().tag("invalid tag"),
        EffectId::Speed,
    ))
    .unwrap_err();
    assert!(
        error.contains("diagnostic:mutation") && error.contains("target"),
        "{error}"
    );
    assert_eq!(
        render(living.damage(DamageAmount::hearts(0.5), DamageKind::Magic)).unwrap(),
        "damage @s 1 minecraft:magic"
    );
    assert_eq!(
        render(
            living
                .give_effect(EffectId::Speed)
                .duration(Ticks::seconds(2))
        )
        .unwrap(),
        "effect give @s minecraft:speed 2"
    );
    // Explicit command text is still available at the command layer.
    assert_eq!(
        living
            .give_effect(EffectId::Speed)
            .seconds(2)
            .try_build()
            .unwrap(),
        "effect give @s minecraft:speed 2"
    );
}

#[test]
fn effect_builder_composes_in_execute_when_macros_and_grouped_actions() {
    let living = EntityContext::<PlayerKind>::default().living();
    let effect = || living.give_effect(EffectId::Speed).seconds(4);
    assert_eq!(
        render(
            Execute::new()
                .if_entity(Target::self_())
                .run_actions(effect())
        )
        .unwrap(),
        "execute if entity @s run effect give @s minecraft:speed 4"
    );
    assert_eq!(
        render(Execute::new().if_entity(Target::self_()).run(effect())).unwrap(),
        "execute if entity @s run effect give @s minecraft:speed 4"
    );
    let group = mcfunction![effect(); living.clear_effect(EffectId::Speed);];
    assert_eq!(
        render(group.clone()).unwrap(),
        "effect give @s minecraft:speed 4\neffect clear @s minecraft:speed"
    );
    let conditional = when(living.has_effect(EffectId::Glowing)).then(group);
    let rendered = render(conditional).unwrap();
    assert!(rendered.contains("execute if function"), "{rendered}");
    assert!(rendered.contains("run function"), "{rendered}");
    let invalid = effect().seconds(0);
    assert!(
        render(
            Execute::new()
                .if_entity(Target::self_())
                .run_actions(invalid)
        )
        .unwrap_err()
        .contains("DURATION")
    );
}

#[test]
#[ignore = "used by the maintained Minecraft 26.2 two-player harness"]
fn dump_living_mutations_pack() {
    let pack: Value =
        serde_json::from_str(&sand::advanced::try_export_pack_json("living", "26.2").unwrap())
            .unwrap();
    std::fs::write(
        std::env::var("SAND_LIVING_MUTATIONS_DUMP_PATH").unwrap(),
        serde_json::to_string_pretty(&serde_json::json!({
            "components": pack["components"],
            "objectives": {
                "enabled": sand::entity::EntityStateField::objective(Combat::enabled),
                "applications": sand::entity::EntityStateField::objective(Combat::applications),
            }
        }))
        .unwrap(),
    )
    .unwrap();
}
