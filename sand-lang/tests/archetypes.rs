mod support;

use sand::advanced::compiler::Compiler;
use support::*;

const SOURCE: &str = include_str!("fixtures/archetypes.sand");

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
