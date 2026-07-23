//! Architecture guard: a datapack author can write a working pack with only
//! `use sand::prelude::*` — attribute macros, commands, events, components,
//! state, and the export entry point all resolve through the façade.

use sand::prelude::*;

static MANA: ScoreVar<i32> = ScoreVar::new("facade_mana");

#[function]
fn facade_hello() {
    cmd::tellraw(
        Selector::all_players(),
        Text::new("facade check").gold().bold(true),
    );
}

#[component]
fn facade_advancement() -> Advancement {
    Advancement::new("facade_ns:facade_root".parse().unwrap())
        .criterion("tick", Criterion::new(AdvancementTrigger::Tick))
}

#[event]
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
#[event]
fn facade_on_hurt(event: Event<sand::event::vanilla::EntityDamagesPlayer>) {
    let attacker: sand::participant::EntityParticipant = event.attacker();
    cmd::raw(format!(
        "# facade check: attacker = {}",
        attacker.selector()
    ));
}

#[event]
fn facade_on_hurt_entity(event: Event<sand::event::vanilla::PlayerDamagesEntity>) {
    let weapon: sand::item::ItemSnapshot = event.weapon();
    cmd::raw(format!(
        "# facade check: weapon storage = {}",
        weapon.storage()
    ));
}

#[test]
fn export_includes_facade_declarations() {
    let json = sand::advanced::try_export_components_json("facade_ns")
        .expect("export must succeed through the facade");
    assert!(json.contains("facade_hello"));
    assert!(json.contains("facade_root"));
    assert!(json.contains("facade_on_hurt"));
}

#[test]
fn prelude_does_not_leak_compiler_internals() {
    // These modules exist, but their contents are deliberately not in the
    // prelude; reaching them requires an explicit advanced/__private path.
    let json = sand::advanced::try_export_components_json("facade_ns2");
    assert!(json.is_ok());
}
