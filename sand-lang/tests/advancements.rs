mod support;

use support::*;

#[test]
fn consumption_and_arrival_use_native_filtered_advancement_triggers() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "travel", minecraft = "26.2")]
dimension Camp { noise_settings: "minecraft:overworld", biome: Biome::Plains }
advancement Journey {
    criteria: {
        meal: { consume_item: Item::Bread },
        arrival: { arrive_dimension: Camp },
    },
}
"#,
    );
    let pack = compile(&analysis);
    let json: serde_json::Value =
        serde_json::from_slice(&pack.resources["data/travel/advancement/journey.json"]).unwrap();
    assert_eq!(
        json["criteria"]["meal"]["trigger"],
        "minecraft:consume_item"
    );
    assert_eq!(
        json["criteria"]["meal"]["conditions"]["item"]["items"],
        serde_json::json!(["minecraft:bread"])
    );
    assert_eq!(
        json["criteria"]["arrival"]["trigger"],
        "minecraft:changed_dimension"
    );
    assert_eq!(
        json["criteria"]["arrival"]["conditions"]["to"],
        "travel:camp"
    );
}

const MILESTONE: &str = r#"
advancement Scout {
    display: {
        icon: Item::Spyglass,
        title: "{gold}Scout",
        description: Text::translate("item.minecraft.spyglass"),
        background: "minecraft:gui/advancements/backgrounds/adventure",
    },
    criteria: {
        find_paper: { inventory_item: Item::Paper },
        place_stone: { place_block: Block::Stone },
        hunt_pig: { kill_entity: Entity::Pig },
    },
    require_any: true,
    experience: 5,
}
"#;

#[test]
fn advancements_export_native_display_requirements_and_typed_triggers() {
    for target in ["26.2", "26.3"] {
        let source =
            format!(r#"#![pack(namespace = "milestone", minecraft = "{target}")]{MILESTONE}"#);
        let pack = compile(&analyze_one(&source));
        let resource: serde_json::Value =
            serde_json::from_slice(&pack.resources["data/milestone/advancement/scout.json"])
                .unwrap();
        assert_eq!(resource["display"]["title"]["extra"][0]["text"], "Scout");
        assert_eq!(
            resource["display"]["description"]["extra"][0]["translate"],
            "item.minecraft.spyglass"
        );
        assert_eq!(
            resource["criteria"]["find_paper"]["trigger"],
            "minecraft:inventory_changed"
        );
        assert_eq!(
            resource["criteria"]["place_stone"]["trigger"],
            "minecraft:placed_block"
        );
        assert_eq!(
            resource["criteria"]["hunt_pig"]["trigger"],
            "minecraft:player_killed_entity"
        );
        assert_eq!(
            resource["requirements"],
            serde_json::json!([["find_paper", "hunt_pig", "place_stone"]])
        );
        assert_eq!(resource["rewards"]["experience"], 5);
    }
}

#[test]
fn advancement_requires_each_named_criterion_by_default() {
    let source = format!(
        r#"#![pack(namespace = "milestone", minecraft = "26.2")]{}"#,
        MILESTONE.replace("    require_any: true,\n", "")
    );
    let pack = compile(&analyze_one(&source));
    let resource: serde_json::Value =
        serde_json::from_slice(&pack.resources["data/milestone/advancement/scout.json"]).unwrap();
    assert_eq!(
        resource["requirements"],
        serde_json::json!([["find_paper"], ["hunt_pig"], ["place_stone"]])
    );
}

#[test]
fn advancement_declarations_validate_trigger_names_types_and_display() {
    for (source, want) in [
        (
            "advancement Empty { criteria: {} }",
            "needs at least one criterion",
        ),
        (
            "advancement Invalid { criteria: { a: { place_block: Item::Paper } } }",
            "expected a `Block`",
        ),
        (
            "advancement Invalid { criteria: { a: { inventory_item: Item::Paper, kill_entity: Entity::Pig } } }",
            "exactly one trigger",
        ),
        (
            "advancement Invalid { criteria: { a: { inventory_item: Item::Paper }, a: { kill_entity: Entity::Pig } } }",
            "duplicate criterion",
        ),
        (
            r#"advancement Invalid { criteria: { a: { inventory_item: Item::Paper } }, display: { icon: Item::Paper, title: "{player}", description: "Desc" } }"#,
            "resource text cannot read runtime values",
        ),
    ] {
        let errors = errors(&analyze_one(&format!(
            r#"#![pack(namespace = "milestone")] {source}"#
        )));
        assert!(
            errors.iter().any(|error| error.contains(want)),
            "{errors:#?}"
        );
    }
}

#[test]
fn advancement_progress_compiles_to_the_current_players_selector() {
    let source = format!(
        r#"#![pack(namespace = "milestone", minecraft = "26.2")]
{MILESTONE}
fn inspect() {{
    if advancement_done(Scout, criterion = "find_paper") && !advancement_done(Scout) {{
        chat!("Paper collected");
    }}
}}
"#
    );
    let pack = compile(&analyze_one(&source));
    let generated = pack
        .resources
        .values()
        .map(|bytes| String::from_utf8_lossy(bytes))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(generated.contains("@s[advancements={milestone:scout={find_paper=true}}]"));
    assert!(generated.contains("@s[advancements={milestone:scout=true}]"));
}

#[test]
fn advancement_progress_rejects_unknown_criteria_and_server_hooks() {
    let source = format!(
        r#"#![pack(namespace = "milestone")]
{MILESTONE}
fn inspect() {{ if advancement_done(Scout, criterion = "bad") {{ chat!("invalid"); }} }}
"#
    );
    let Err(errors) = analyze_one(&source).compile() else {
        panic!("accepted a criterion absent from the declared advancement");
    };
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("has no criterion `bad`"))
    );

    let source = format!(
        r#"#![pack(namespace = "milestone")]
{MILESTONE}
#[tick]
fn from_server() {{ if advancement_done(Scout) {{ broadcast!("invalid"); }} }}
"#
    );
    assert!(analyze_one(&source).has_errors());
}

#[test]
fn completion_handlers_share_one_native_reward_and_keep_declaration_order() {
    let source = format!(
        r#"#![pack(namespace = "milestone", minecraft = "26.2")]
{MILESTONE}
#[on(advancement = Scout)]
fn give_reward() {{ give_item(Item::Bread); }}
#[on(advancement = Scout)]
fn mark_complete() {{ chat!("Scouting complete"); }}
"#
    );
    let analysis = analyze_one(&source);
    let pack = compile(&analysis);
    let id = "milestone:__sand_program/event/advancement/milestone/scout";
    let advancement: serde_json::Value =
        serde_json::from_slice(&pack.resources["data/milestone/advancement/scout.json"]).unwrap();
    assert_eq!(advancement["rewards"]["function"], id);
    assert_eq!(advancement["rewards"]["experience"], 5);

    let dispatch = function(
        &pack,
        "milestone/function/__sand_program/event/advancement/milestone/scout",
    );
    assert!(dispatch.lines().rev().take(2).eq([
        "function milestone:mark_complete",
        "function milestone:give_reward",
    ]));
    assert!(!pack.resources.iter().any(|(path, bytes)| {
        path.ends_with(".mcfunction")
            && path != "data/milestone/function/__sand_program/event/advancement/milestone/scout.mcfunction"
            && String::from_utf8_lossy(bytes).contains(id)
    }));

    let mut missing = analysis.program.unwrap();
    missing.modules[0].advancements.clear();
    let errors = sand::advanced::compiler::Compiler::compile(&missing).unwrap_err();
    assert!(errors.iter().any(|error| {
        error.pointer == "/events/0/event/advancement"
            && error.message.contains("requires an authored advancement")
    }));
}

#[test]
fn completion_handlers_require_a_visible_advancement_declaration() {
    for attribute in [
        "advancement",
        "advancement = \"minecraft:story/root\"",
        "advancement = Unknown",
    ] {
        let source = format!(
            "#![pack(namespace = \"milestone\")]\n#[on({attribute})]\nfn completion() {{}}"
        );
        let analysis = analyze_one(&source);
        assert!(analysis.has_errors(), "{attribute}");
    }
}

#[test]
fn native_interaction_taming_and_totem_criteria_keep_their_filters() {
    let source = r#"
#![pack(namespace = "milestone", minecraft = "26.2")]
advancement Companion {
    criteria: {
        touch: { interact_entity: { entity: Entity::Wolf, item: Item::Bone } },
        tame: { tame_entity: Entity::Wolf },
        survived: { use_totem: Item::TotemOfUndying },
    },
}
"#;
    let analysis = analyze_one(source);
    let pack = compile(&analysis);
    let json: serde_json::Value =
        serde_json::from_slice(&pack.resources["data/milestone/advancement/companion.json"])
            .unwrap();
    let criteria = &json["criteria"];
    assert_eq!(
        criteria["touch"]["trigger"],
        "minecraft:player_interacted_with_entity"
    );
    assert_eq!(
        criteria["touch"]["conditions"]["entity"][0]["predicate"]["minecraft:entity_type"],
        "minecraft:wolf"
    );
    assert_eq!(
        criteria["touch"]["conditions"]["item"]["items"],
        serde_json::json!(["minecraft:bone"])
    );
    assert_eq!(criteria["tame"]["trigger"], "minecraft:tame_animal");
    assert_eq!(
        criteria["tame"]["conditions"]["entity"][0]["predicate"]["minecraft:entity_type"],
        "minecraft:wolf"
    );
    assert_eq!(criteria["survived"]["trigger"], "minecraft:used_totem");
    assert_eq!(
        criteria["survived"]["conditions"]["item"]["items"],
        serde_json::json!(["minecraft:totem_of_undying"])
    );

    let invalid = source.replace("item: Item::Bone", "item: Entity::Pig");
    assert!(analyze_one(&invalid).has_errors());
    let encoded = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded =
        sand::advanced::compiler::Compiler::decode(&encoded, &Default::default()).unwrap();
    assert_eq!(
        sand::advanced::compiler::Compiler::compile(&decoded)
            .unwrap()
            .resources,
        pack.resources
    );
}
