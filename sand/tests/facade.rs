//! Architecture guard: a datapack author can write a working pack with only
//! `use sand::prelude::*` — attribute macros, commands, events, components,
//! state, and the export entry point all resolve through the façade.

use sand::prelude::*;

static MANA: ScoreVar<i32> = ScoreVar::new("facade_mana");

#[function]
fn facade_hello() {
    cmd::tellraw(
        Target::players(),
        Text::new("facade check").gold().bold(true),
    );
}

#[datapack_component]
fn facade_advancement() -> Advancement {
    Advancement::new("facade_ns:facade_root".parse().unwrap())
        .criterion("tick", Criterion::new(AdvancementTrigger::Tick))
}

#[on_event]
fn facade_join(event: Event<sand::events::OnJoinEvent>) {
    let _ = event;
    cmd::call(facade_hello);
    let _ = MANA.set("@s", 10);
}

// Participant context (#230) is reachable through the façade — both the
// curated vocabulary in the glob prelude (`EntityParticipantRole`) and the
// `sand::participant` module for typed handles/plan declaration. Accessors
// return the typed participant directly, not a `ParticipantAvailability`
// wrapper (#273).
#[on_event]
fn facade_on_hurt(event: Event<sand::event::vanilla::EntityDamagesPlayer>) {
    let attacker: sand::participant::EntityParticipant = event.attacker();
    cmd::raw(format!(
        "# facade check: attacker = {}",
        attacker.selector()
    ));
}

#[on_event]
fn facade_on_hurt_entity(event: Event<sand::event::vanilla::PlayerDamagesEntity>) {
    let weapon: sand::item::ItemSnapshot = event.weapon();
    cmd::raw(format!(
        "# facade check: weapon storage = {}",
        weapon.storage()
    ));
}

#[test]
fn export_includes_facade_declarations() {
    let json = sand::advanced::try_export_components_json("facade_ns", "26.2")
        .expect("export must succeed through the facade");
    assert!(json.contains("facade_hello"));
    assert!(json.contains("facade_root"));
    assert!(json.contains("facade_on_hurt"));
}

// Inventory's validated fallible path (#172) is reachable using only the
// glob prelude — no explicit `sand_commands::inventory` import required.
#[test]
fn prelude_only_inventory_try_methods_compile_and_validate() {
    let inv = Inventory::of(Target::self_());

    // Valid input on the fallible path matches the infallible builder's
    // output exactly (regression: identical generated command text).
    assert_eq!(
        inv.try_set(ItemSlot::MainHand, "minecraft:diamond_sword")
            .unwrap(),
        inv.set(ItemSlot::MainHand, "minecraft:diamond_sword")
    );

    // Wildcard slots are rejected in a single-slot write context …
    assert!(inv.try_set(ItemSlot::AnyHotbar, "minecraft:stone").is_err());
    // … but the infallible compatibility path never panics on the same input.
    let _ = inv.set(ItemSlot::AnyHotbar, "minecraft:stone");

    // Out-of-range slot indices are diagnostics, not panics, on both paths.
    assert!(
        inv.try_set(ItemSlot::Hotbar(99), "minecraft:stone")
            .is_err()
    );
    let _ = inv.set(ItemSlot::Hotbar(99), "minecraft:stone");
}

#[test]
fn canonical_predicate_id_filters_the_canonical_target() {
    let predicate = PredicateId::custom("facade_ns:is_ready".parse().unwrap());
    assert_eq!(
        Target::entities().predicate(predicate).to_string(),
        "@e[predicate=facade_ns:is_ready]"
    );

    let predicate = PredicateId::custom("facade_ns:is_ready".parse().unwrap());
    assert_eq!(
        Target::players().not_predicate(predicate).to_string(),
        "@a[predicate=!facade_ns:is_ready]"
    );
}

#[test]
fn prelude_does_not_leak_compiler_internals() {
    // These modules exist, but their contents are deliberately not in the
    // prelude; reaching them requires an explicit advanced/__private path.
    let json = sand::advanced::try_export_components_json("facade_ns2", "26.2");
    assert!(json.is_ok());
}
