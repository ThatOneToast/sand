//! Portable gameplay vocabulary: score operations, relations, text, sounds,
//! effects, items, schedules, and player events decoded from strict JSON.
use sand::advanced::compiler::{Compiler, Program};
use std::collections::BTreeMap;

fn decode(document: serde_json::Value) -> Program {
    Compiler::decode(document.to_string().as_bytes(), &BTreeMap::new())
        .unwrap_or_else(|errors| panic!("decode failed: {errors:#?}"))
}

fn program(functions: serde_json::Value, events: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "format": "sand.program", "format_version": 1, "target": {"minecraft": "26.2"},
        "pack": {"namespace": "play", "description": "Portable gameplay"},
        "modules": [{
            "id": "play:module",
            "states": [
                {"id": "play:stats", "scope": "player", "revision": 1, "fields": [
                    {"name": "gold", "bounds": [0, 100]}, {"name": "best"}]},
                {"id": "play:world", "scope": "global", "revision": 1, "fields": [{"name": "round"}]}
            ],
            "functions": functions,
            "events": events
        }]
    })
}

fn field(state: &str, name: &str) -> serde_json::Value {
    serde_json::json!({"state": format!("play:{state}"), "field": name})
}

fn function_text(pack: &sand::advanced::compiler::CompiledPack, path: &str) -> String {
    String::from_utf8(pack.resources[&format!("data/play/function/{path}.mcfunction")].clone())
        .unwrap()
}

#[test]
fn gameplay_actions_lower_to_typed_commands() {
    let gold = field("stats", "gold");
    let best = field("stats", "best");
    let round = field("world", "round");
    let document = program(
        serde_json::json!([
            {"id": "play:reward", "context": "player", "body": [
                {"action": {"op": "score_operation", "score": gold, "operator": "mul", "operand": 3}},
                {"action": {"op": "score_operation", "score": best, "operator": "max", "operand": gold}},
                {"action": {"op": "score_random", "score": best, "min": 1, "max": 6}},
                {"action": {"op": "branch",
                    "condition": {"left": gold, "relation": "gt", "right": best},
                    "then": [{"action": {"op": "message", "audience": "player", "channel": "title",
                        "text": [{"kind": "text", "text": "Gold: ", "style": {"color": "gold", "bold": true}},
                                 {"kind": "score", "score": gold},
                                 {"kind": "player_name"}]}}]}},
                {"action": {"op": "sound", "sound": "minecraft:entity.player.levelup", "pitch": 1.5}},
                {"action": {"op": "particle", "particle": "minecraft:heart", "count": 3}},
                {"action": {"op": "effect", "effect": "minecraft:speed", "seconds": 5, "amplifier": 1}},
                {"action": {"op": "clear_effect"}},
                {"action": {"op": "give_item", "item": "minecraft:diamond", "count": 2}},
                {"action": {"op": "clear_item", "item": "minecraft:dirt"}}
            ]},
            {"id": "play:tick", "context": "server", "body": [
                {"action": {"op": "message", "audience": "everyone", "channel": "chat",
                    "text": [{"kind": "text", "text": "Round "}, {"kind": "score", "score": round}]}},
                {"action": {"op": "schedule", "function": {"kind": "internal", "id": "play:tick"}, "ticks": 40}}
            ]}
        ]),
        serde_json::json!([
            {"event": {"kind": "death"}, "function": {"kind": "internal", "id": "play:reward"}},
            {"event": {"kind": "use_item", "item": "minecraft:carrot_on_a_stick"},
             "function": {"kind": "internal", "id": "play:reward"}}
        ]),
    );
    let pack = Compiler::compile(&decode(document)).unwrap();
    let reward = function_text(&pack, "reward");
    for expected in [
        "scoreboard players operation @s ",
        " *= #c3 ",
        " > @s ",
        "run random value 1..6",
        "playsound minecraft:entity.player.levelup master @s ~ ~ ~ 1 1.5",
        "particle minecraft:heart ~ ~1 ~ 0 0 0 0 3",
        "effect give @s minecraft:speed 5 1",
        "effect clear @s",
        "give @s minecraft:diamond 2",
        "clear @s minecraft:dirt",
    ] {
        assert!(
            reward.contains(expected),
            "missing {expected:?} in\n{reward}"
        );
    }
    // Bounded fields are clamped after operations, like additions.
    assert!(reward.contains("matches 101.. run scoreboard players set @s"));
    let tick = function_text(&pack, "tick");
    assert!(tick.starts_with("tellraw @a "), "{tick}");
    assert!(
        tick.contains("schedule function play:tick 40t replace"),
        "{tick}"
    );
    let branch = pack
        .resources
        .iter()
        .find(|(path, bytes)| {
            path.ends_with("/then.mcfunction") && String::from_utf8_lossy(bytes).contains("title")
        })
        .map(|(_, bytes)| String::from_utf8_lossy(bytes).into_owned())
        .expect("title branch");
    assert!(
        branch.contains(r#"{"bold":true,"color":"gold","text":"Gold: "}"#),
        "{branch}"
    );

    let load = function_text(&pack, "__sand_lifecycle_load");
    assert!(load.contains(" deathCount"), "{load}");
    assert!(load.contains("scoreboard players set #c3 "), "{load}");
    let tick = function_text(&pack, "__sand_lifecycle_tick");
    assert!(
        tick.contains(
            "run function play:__sand_program/event/use_item/minecraft/carrot_on_a_stick"
        ),
        "{tick}"
    );
}

#[test]
fn gameplay_actions_validate_context_ranges_and_identities() {
    let gold = field("stats", "gold");
    let document = program(
        serde_json::json!([
            {"id": "play:server", "context": "server", "body": [
                {"action": {"op": "sound", "sound": "minecraft:block.bell.use"}},
                {"action": {"op": "give_item", "item": "diamond", "count": 0}},
                {"action": {"op": "score_operation", "score": gold, "operator": "div", "operand": 0}},
                {"action": {"op": "schedule", "function": {"kind": "internal", "id": "play:player"}, "ticks": 0}}
            ]},
            {"id": "play:player", "context": "player", "body": []}
        ]),
        serde_json::json!([]),
    );
    let errors = Compiler::check(&decode(document)).unwrap_err();
    let messages: Vec<_> = errors
        .iter()
        .map(|e| (e.pointer.as_str(), e.message.as_str()))
        .collect();
    for (pointer, message) in [
        (
            "/functions/0/body/0",
            "a sound for the executing player requires player execution context",
        ),
        (
            "/functions/0/body/1/action/item",
            "`diamond` is not a namespaced item identity such as `minecraft:example`",
        ),
        (
            "/functions/0/body/1/action/count",
            "item count must be 1..=6400",
        ),
        (
            "/functions/0/body/2/action/operand",
            "division or modulo by the constant zero has no effect",
        ),
        (
            "/functions/0/body/3/action/function",
            "callee requires a player executor",
        ),
        (
            "/functions/0/body/3/action/ticks",
            "schedule delay must be at least one tick",
        ),
    ] {
        assert!(
            messages.contains(&(pointer, message)),
            "missing {pointer} {message:?} in {messages:#?}"
        );
    }
}

#[test]
fn existing_range_branches_still_decode() {
    let gold = field("stats", "gold");
    let document = program(
        serde_json::json!([{"id": "play:f", "context": "player", "body": [
            {"action": {"op": "branch", "condition": {"score": gold, "min": 1, "max": null},
             "then": [{"action": {"op": "score_add", "score": gold, "value": 1}}]}}
        ]}]),
        serde_json::json!([]),
    );
    Compiler::compile(&decode(document)).unwrap();
}

#[test]
fn return_while_and_attributes_lower_to_function_results_and_macros() {
    let gold = field("stats", "gold");
    let best = field("stats", "best");
    let document = program(
        serde_json::json!([{"id": "play:f", "context": "player", "body": [
            {"action": {"op": "branch", "condition": {"score": gold, "min": null, "max": 0},
             "then": [{"action": {"op": "return"}}]}},
            {"action": {"op": "while", "condition": {"score": best, "min": 1, "max": null},
             "body": [{"action": {"op": "score_add", "score": best, "value": -1}}]}},
            {"action": {"op": "attribute_get", "score": best, "attribute": "minecraft:max_health"}},
            {"action": {"op": "attribute_set", "attribute": "minecraft:max_health", "value": 24.0}},
            {"action": {"op": "attribute_set", "attribute": "minecraft:movement_speed",
             "value": {"score": best, "scale": 1000.0}}}
        ]}]),
        serde_json::json!([]),
    );
    let pack = Compiler::compile(&decode(document)).unwrap();
    let f = function_text(&pack, "f");
    let lines: Vec<_> = f.lines().collect();
    assert!(
        lines[1].starts_with("execute if function play:__sand_program/")
            && lines[1].ends_with(" run return 1"),
        "{f}"
    );
    assert!(
        lines[2].contains("matches 1..2147483647 run function play:__sand_program/"),
        "{f}"
    );
    assert!(
        f.contains("run attribute @s minecraft:max_health base get 1"),
        "{f}"
    );
    assert!(
        f.contains("attribute @s minecraft:max_health base set 24"),
        "{f}"
    );
    assert!(
        f.contains("store result storage play:sand.args value double 0.001"),
        "{f}"
    );
    let find = |suffix: &str| {
        pack.resources
            .iter()
            .find(|(path, _)| path.ends_with(suffix))
            .map(|(_, bytes)| String::from_utf8_lossy(bytes).into_owned())
            .unwrap()
    };
    let branch = find("/branch.mcfunction");
    assert_eq!(
        branch
            .lines()
            .filter(|l| l.contains("return run function"))
            .count(),
        2,
        "{branch}"
    );
    assert_eq!(find("/then.mcfunction"), "return 1");
    assert!(find("/loop.mcfunction").contains("run return run function"));
    assert_eq!(
        find("/attribute.mcfunction"),
        "$attribute @s minecraft:movement_speed base set $(value)"
    );
}

#[test]
fn custom_items_recipes_and_tags_lower_through_typed_components() {
    let mut document = program(
        serde_json::json!([{"id": "play:f", "context": "player", "body": [
            {"action": {"op": "give_item", "item": "play:ruby_sword", "count": 1}},
            {"action": {"op": "clear_item", "item": "play:ruby_sword"}}
        ]}]),
        serde_json::json!([]),
    );
    let module = &mut document["modules"][0];
    module["items"] = serde_json::json!([{
        "id": "play:ruby_sword", "base": "minecraft:diamond_sword",
        "name": [{"kind": "text", "text": "Ruby Sword", "style": {"color": "red"}}],
        "unbreakable": true, "rarity": "epic"
    }]);
    module["recipes"] = serde_json::json!([
        {"kind": "shaped", "id": "play:ruby_sword", "pattern": [" R ", " R ", " S "],
         "key": {"R": "minecraft:redstone_block", "S": "#minecraft:planks"},
         "result": {"item": "play:ruby_sword"}},
        {"kind": "cooking", "id": "play:leather", "method": "smelting",
         "ingredient": "minecraft:rotten_flesh", "result": {"item": "minecraft:leather"}, "experience": 0.1}
    ]);
    module["resource_tags"] = serde_json::json!([
        {"kind": "item", "id": "play:ores", "values": ["minecraft:iron_ore", "#minecraft:coal_ores"]}
    ]);
    let pack = Compiler::compile(&decode(document)).unwrap();
    let f = function_text(&pack, "f");
    assert!(
        f.contains("give @s minecraft:diamond_sword[")
            && f.contains("custom_data={\"play:ruby_sword\":1b}"),
        "{f}"
    );
    assert!(
        f.contains(
            "clear @s minecraft:diamond_sword[minecraft:custom_data~{\"play:ruby_sword\":1b}]"
        ),
        "{f}"
    );
    let json = |path: &str| -> serde_json::Value {
        serde_json::from_slice(&pack.resources[path]).unwrap_or_else(|_| panic!("{path}"))
    };
    let shaped = json("data/play/recipe/ruby_sword.json");
    assert_eq!(shaped["type"], "minecraft:crafting_shaped");
    assert_eq!(shaped["result"]["id"], "minecraft:diamond_sword");
    assert!(
        shaped["result"]["components"]
            .to_string()
            .contains("play:ruby_sword"),
        "{shaped}"
    );
    assert_eq!(
        json("data/play/recipe/leather.json")["type"],
        "minecraft:smelting"
    );
    let tag = json("data/play/tags/item/ores.json");
    assert_eq!(
        tag["values"],
        serde_json::json!(["minecraft:iron_ore", "#minecraft:coal_ores"])
    );

    let mut bad = program(serde_json::json!([]), serde_json::json!([]));
    bad["modules"][0]["items"] =
        serde_json::json!([{"id": "play:gem", "base": "minecraft:emerald"}]);
    bad["modules"][0]["recipes"] = serde_json::json!([
        {"kind": "shaped", "id": "play:x", "pattern": ["AB"], "key": {"A": "play:gem", "C": "minecraft:stick"},
         "result": {"item": "minecraft:stick"}}
    ]);
    let errors = Compiler::check(&decode(bad)).unwrap_err();
    let messages: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
    for expected in [
        "pattern character `B` has no key",
        "key `C` is not used in the pattern",
        "custom item `play:gem` cannot be a recipe ingredient: vanilla recipes match only by item id or tag",
    ] {
        assert!(
            messages.contains(&expected),
            "missing {expected:?} in {messages:#?}"
        );
    }
}
