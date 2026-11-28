mod support;

use sand::advanced::compiler::Compiler;
use support::*;

const SOURCE: &str = r#"
#![pack(namespace = "archetype_probe", minecraft = "26.2")]
entity state Memory { value: number = 4, name: string = "sentinel" }
global state Activity { initialized: number, released: number }

archetype Sentinel {
    entity: Entity::ArmorStand,
    components: [Memory],
    initialize: initialize,
    cleanup: cleanup,
}
archetype Observer {
    entity: Entity::ArmorStand,
    components: [Memory],
}
archetype Piglet {
    entity: Entity::Pig,
    components: [Memory],
    adopt_every: 4,
}
fn initialize() { Memory.value += 1; Activity.initialized += 1; }
fn cleanup() {
    if Memory.value == 5 { Activity.released += 1; }
}
pub fn create() { spawn_archetype(Sentinel, at = pos(0, 110, 0)); }
pub fn add_observer() { attach_archetype(Observer); }
pub fn release_sentinel() { release_archetype(Sentinel); }
pub fn release_observer() { release_archetype(Observer); }
pub fn check_current() {
    if has_archetype(Sentinel) && has_state(Memory) && Memory.name == "sentinel" {
        Activity.initialized += 1;
    }
}
"#;

#[test]
fn archetypes_spawn_and_bind_canonical_component_lifecycles() {
    let analysis = analyze_one(SOURCE);
    let pack = compile(&analysis);
    assert!(
        function(&pack, "archetype_probe/function/create").contains(
            "positioned 0 110 0 summon minecraft:armor_stand run function archetype_probe:"
        )
    );
    let commands = pack
        .resources
        .iter()
        .filter(|(path, _)| path.ends_with(".mcfunction"))
        .map(|(_, text)| std::str::from_utf8(text).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    for required in [
        "function archetype_probe:initialize",
        "function archetype_probe:cleanup",
        "type=minecraft:pig",
        "scoreboard players reset @s",
        "unless entity @s[tag=",
    ] {
        assert!(commands.contains(required), "{required}");
    }
    let encoded = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded = Compiler::decode(&encoded, &Default::default()).unwrap();
    assert_eq!(
        pack.resources,
        Compiler::compile(&decoded).unwrap().resources
    );
}

#[test]
fn archetype_components_require_distinct_entity_state() {
    for (old, invalid, expected) in [
        (
            "entity state Memory",
            "player state Memory",
            "archetype components require entity State",
        ),
        (
            "components: [Memory]",
            "components: [Memory, Memory]",
            "duplicate archetype component",
        ),
        (
            "adopt_every: 4",
            "adopt_every: 0",
            "expected a value from 1",
        ),
    ] {
        let diagnostics = errors(&analyze_one(&SOURCE.replacen(old, invalid, 1)));
        assert!(
            diagnostics.iter().any(|error| error.contains(expected)),
            "{diagnostics:?}"
        );
    }
}

#[test]
fn archetype_private_objectives_cannot_alias_state_fields() {
    let mut program = analyze_one(SOURCE).program.unwrap();
    let module = &mut program.modules[0];
    let mut state = module
        .states
        .iter()
        .find(|state| state.scope == "entity")
        .unwrap()
        .clone();
    state.id = module.archetypes[0].id.clone();
    state.fields[0].name = "archetype_version".into();
    module.states.push(state);
    let diagnostics = Compiler::compile(&program).unwrap_err();
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("collides with an automatic state declaration")),
        "{diagnostics:?}"
    );
}
