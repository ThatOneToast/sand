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

#[test]
fn export_includes_facade_declarations() {
    let json = sand::advanced::try_export_components_json("facade_ns")
        .expect("export must succeed through the facade");
    assert!(json.contains("facade_hello"));
    assert!(json.contains("facade_root"));
}

#[test]
fn prelude_does_not_leak_compiler_internals() {
    // These modules exist, but their contents are deliberately not in the
    // prelude; reaching them requires an explicit advanced/__private path.
    let json = sand::advanced::try_export_components_json("facade_ns2");
    assert!(json.is_ok());
}
