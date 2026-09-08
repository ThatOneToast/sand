//! Composition coverage for the focused `EntityContext` capability façade.

use std::sync::Mutex;

use sand_commands::Target;
use sand_components::EquipmentSlot;
use sand_core::entity::{EntityContext, EntityScope, PlayerKind, TargetExecution, ZombieKind};

static DYN_FN_REGISTRY_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn target_each_composes_multiple_capability_families() {
    let _guard = DYN_FN_REGISTRY_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let commands = Target::players().each(|player| {
        let helmet = player.equipment().slot(EquipmentSlot::Head).unwrap();
        let hotbar = player.inventory().hotbar(0).unwrap();
        vec![
            player.transform().position().get().to_string(),
            helmet.nbt().get().to_string(),
            hotbar.nbt().get().to_string(),
            player.living().clear_effects(),
            player.mounts().dismount(),
        ]
    });

    assert_eq!(commands.len(), 1);
    let generated = sand_core::function::drain_dyn_fns();
    let body = generated
        .iter()
        .find(|(path, _)| commands[0].ends_with(path))
        .expect("each() registers its capability body");
    assert_eq!(
        body.1,
        [
            "data get entity @s Pos",
            "data get entity @s Inventory[{Slot:103b}]",
            "data get entity @s Inventory[{Slot:0b}]",
            "effect clear @s",
            "ride @s dismount",
        ]
    );
}

#[test]
fn scoped_capabilities_keep_the_bound_selector_across_relationship_traversal() {
    let _guard = DYN_FN_REGISTRY_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let zombie = EntityContext::<ZombieKind>::default();
    let commands = EntityScope::bind(&zombie, |bound| {
        bound
            .owner()
            .if_player(|owner| {
                vec![
                    owner.living().clear_effects(),
                    bound.transform().teleport_to(Target::self_()),
                    bound.data().field::<i32>("Air").set(300).to_string(),
                ]
            })
            .unwrap()
    });

    let scoped_tag = commands[0]
        .strip_prefix("tag @s add ")
        .expect("bind starts by tagging @s");
    let generated = sand_core::function::drain_dyn_fns();
    let relation_call = &commands[1];
    let body = generated
        .iter()
        .find(|(path, _)| relation_call.ends_with(path))
        .expect("relationship traversal registers its body");

    assert_eq!(body.1[0], "effect clear @s");
    assert_eq!(
        body.1[1],
        format!("teleport @e[tag={scoped_tag},limit=1] @s")
    );
    assert_eq!(
        body.1[2],
        format!("data modify entity @e[tag={scoped_tag},limit=1] Air set value 300")
    );
    assert_eq!(
        commands.last().unwrap(),
        &format!("tag @e[tag={scoped_tag}] remove {scoped_tag}")
    );
}

#[test]
fn typed_contexts_expose_only_their_static_capabilities() {
    let zombie = EntityContext::<ZombieKind>::default();
    let player = EntityContext::<PlayerKind>::default();

    assert_eq!(
        zombie.living().health().get().to_string(),
        "data get entity @s Health"
    );
    assert_eq!(
        player.transform().rotation().get().to_string(),
        "data get entity @s Rotation"
    );
    // Compile-fail doctests on the capability types cover MarkerKind living/
    // equipment access and player entity-data mutation.
}

#[test]
fn repeated_capability_bodies_deduplicate_deterministically() {
    let _guard = DYN_FN_REGISTRY_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let build = |tag: &str| {
        Target::players().tag(tag).each(|player| {
            vec![
                player.transform().position().get().to_string(),
                player.living().clear_effects(),
            ]
        })
    };

    let first = build("first");
    let second = build("second");
    let first_path = first[0].rsplit("function ").next().unwrap();
    let second_path = second[0].rsplit("function ").next().unwrap();
    assert_eq!(first_path, second_path);

    let generated = sand_core::function::drain_dyn_fns();
    assert_eq!(
        generated
            .iter()
            .filter(|(path, _)| path == first_path.trim_start_matches("__sand_local:"))
            .count(),
        1
    );
}
