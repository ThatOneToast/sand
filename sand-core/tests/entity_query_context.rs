//! Integration coverage for issue #227: cardinality-aware entity queries,
//! execution-scoped contexts, typed relationship traversal, and scoped
//! bindings that preserve context across traversal.

use std::sync::Mutex;

use sand_core::entity::{EntityQuery, EntityScope, PlayerQuery};
use sand_core::version::{MinecraftVersion, VersionProfile};

fn latest() -> VersionProfile {
    VersionProfile::resolve(&MinecraftVersion::parse("latest").unwrap()).unwrap()
}

// `drain_dyn_fns()` reads a process-global registry shared by every test in
// this binary; serialize the tests that touch it so they don't observe each
// other's generated helper functions.
static DYN_FN_REGISTRY_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn each_lowers_query_iteration_without_a_manual_execute_chain() {
    // Acceptance: "Users can iterate a typed entity query without manually
    // writing an execute chain."
    let cmds = EntityQuery::entities()
        .entity_type("minecraft:zombie")
        .without_tag("friendly")
        .within_blocks(15.0)
        .nearest()
        .each(|entity| vec![entity.add_tag("observed")]);

    assert_eq!(cmds.len(), 1);
    assert!(cmds[0].starts_with(
        "execute as @e[type=minecraft:zombie,tag=!friendly,distance=..15,sort=nearest,limit=1] at @s run function __sand_local:sand/entity_query/"
    ));
}

#[test]
fn nested_relationship_traversal_retains_original_context() {
    let _guard = DYN_FN_REGISTRY_LOCK.lock().unwrap();
    // Mirrors the issue's worked example: bind the current entity (e.g. an
    // arrow), traverse to its owner, and — if the owner is a player holding
    // a specific item — tag the *original* bound entity, not the owner.
    let profile = latest();

    let cmds = EntityQuery::entities()
        .entity_type("minecraft:arrow")
        .each(|arrow| {
            EntityScope::bind(arrow, |arrow_ref| {
                arrow_ref
                    .owner()
                    .if_player(&profile, |_owner| vec![arrow_ref.add_tag("special")])
                    .unwrap()
            })
        });

    assert_eq!(cmds.len(), 1);
    let outer = &cmds[0];
    assert!(
        outer.starts_with("execute as @e[type=minecraft:arrow] at @s run function __sand_local:")
    );

    // Drain the generated helper functions and confirm the scoped tag/
    // untag pair wraps the relation traversal, and that the relation
    // traversal refers back to the *tagged* entity, not `@s`.
    let generated = sand_core::function::drain_dyn_fns();
    let outer_fn = generated
        .iter()
        .find(|(path, _)| outer.ends_with(path))
        .expect("outer each() function should be registered");
    assert_eq!(
        outer_fn.1.len(),
        3,
        "tag add, relation traversal, tag remove"
    );
    assert!(outer_fn.1[0].starts_with("tag @s add __sand_scope_"));
    assert!(outer_fn.1[1].starts_with(
        "execute on owner if entity @s[type=minecraft:player] run function __sand_local:"
    ));
    assert!(outer_fn.1[2].starts_with("tag @e[tag=__sand_scope_"));
    assert!(outer_fn.1[2].contains("] remove __sand_scope_"));

    let relation_fn = generated
        .iter()
        .find(|(path, _)| outer_fn.1[1].ends_with(path))
        .expect("relation traversal function should be registered");
    // The tag command inside the relation branch targets the scoped entity
    // by tag, not `@s` (which is now the owner).
    assert!(relation_fn.1[0].starts_with("tag @e[tag=__sand_scope_"));
    assert!(relation_fn.1[0].contains("] add special"));
}

#[test]
fn version_gated_relation_fails_with_actionable_diagnostic_before_export() {
    // Acceptance: "Unsupported relationships fail during export with
    // actionable diagnostics."
    let old = VersionProfile::resolve(&MinecraftVersion::parse("1.19.4").unwrap()).unwrap();
    let ctx: sand_core::entity::EntityContext<sand_core::entity::AnyEntity> = Default::default();

    let err = ctx
        .attacker()
        .if_present(&old, |a| vec![a.add_tag("hit_something")])
        .expect_err("attacker relation should be gated on 1.19.4")
        .to_string();

    assert!(err.contains("entity_relation_attacker"));
    assert!(err.contains("1.20.2"));
    assert!(err.contains("1.19.4"));
}

#[test]
fn player_query_each_narrows_to_player_context() {
    let cmds = PlayerQuery::players()
        .tag("ready")
        .nearest()
        .each(|player| vec![player.add_tag("chosen")]);

    assert_eq!(cmds.len(), 1);
    assert!(cmds[0].starts_with(
        "execute as @a[tag=ready,sort=nearest,limit=1] at @s run function __sand_local:sand/entity_query/"
    ));
}

#[test]
fn passengers_relation_is_many_cardinality_and_iterates_via_each() {
    let _guard = DYN_FN_REGISTRY_LOCK.lock().unwrap();
    let profile = latest();
    let cmds = EntityQuery::entities()
        .entity_type("minecraft:boat")
        .limit(1)
        .each(|boat| {
            boat.passengers()
                .each(&profile, |passenger| vec![passenger.add_tag("aboard")])
                .unwrap()
        });

    assert_eq!(cmds.len(), 1);
    let generated = sand_core::function::drain_dyn_fns();
    let outer_fn = generated
        .iter()
        .find(|(path, _)| cmds[0].ends_with(path))
        .expect("outer each() function should be registered");
    assert!(outer_fn.1[0].starts_with("execute on passengers run function __sand_local:"));
}
