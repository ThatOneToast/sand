//! Gameplay vocabulary: arithmetic, text, builtins, events, and scheduling,
//! checked through emitted commands, diagnostics, and editor explanations.

mod support;

use sand_lang::FileId;
use sand_lang::ide;
use support::*;

const PACK: &str = "#![pack(namespace = \"demo\", minecraft = \"26.2\")]\n";

fn pack(body: &str) -> String {
    format!("{PACK}{body}")
}

fn at(text: &str, needle: &str) -> usize {
    text.find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not found"))
}

#[test]
fn arithmetic_chains_fold_constants_and_read_fields() {
    let analysis = analyze_one(&pack(
        r#"
player state S { a: number, b: number, c: number }
fn f() {
    S.a = S.b * (2 + 3) - S.c;
    S.a *= 4;
    S.a %= S.b;
    S.a = -S.b;
}
"#,
    ));
    let program = analysis.program.as_ref().expect("lowered");
    let body = serde_json::to_value(&program.modules[0].functions[0].body).unwrap();
    let ops: Vec<String> = body
        .as_array()
        .unwrap()
        .iter()
        .map(|op| {
            let action = &op["action"];
            match action["op"].as_str().unwrap() {
                "score_operation" => format!(
                    "{} {}",
                    action["operator"].as_str().unwrap(),
                    action["operand"]
                        .get("field")
                        .map(|f| f.as_str().unwrap().to_string())
                        .unwrap_or_else(|| action["operand"].to_string())
                ),
                other => format!("{other} {}", action["value"]),
            }
        })
        .collect();
    assert_eq!(
        ops,
        vec![
            "set b", "mul 5", "sub c", "mul 4", "mod b", "set b", "mul -1"
        ]
    );
    let pack = compile(&analysis);
    let f = function(&pack, "demo/function/f");
    assert!(f.contains("scoreboard players operation @s"), "{f}");
}

#[test]
fn reading_the_target_later_in_the_expression_uses_a_temporary() {
    let analysis = analyze_one(&pack(
        "player state S { a: number, b: number }\nfn f() { S.a = S.b - S.a; }\n",
    ));
    let program = analysis.program.as_ref().expect("compiles");
    let body = serde_json::to_value(&program.modules[0].functions[0].body).unwrap();
    // b - a is computed in a temporary, then copied into a in one write.
    let last = &body.as_array().unwrap().last().unwrap()["action"];
    assert_eq!(last["operator"], "set");
    assert_eq!(last["score"]["field"], "a");
    assert_eq!(last["operand"]["state"], "demo:sand.locals");
}

#[test]
fn random_and_field_comparisons() {
    let analysis = analyze_one(&pack(
        r#"
player state Dice { roll: number, best: number }
fn roll() {
    Dice.roll = random(1..=6);
    if Dice.roll > Dice.best { Dice.best = Dice.roll; }
}
"#,
    ));
    let pack = compile(&analysis);
    let roll = function(&pack, "demo/function/roll");
    assert!(roll.contains("run random value 1..6"), "{roll}");
    let branch = pack
        .resources
        .iter()
        .find(|(path, _)| path.ends_with("branch.mcfunction"))
        .map(|(_, bytes)| String::from_utf8_lossy(bytes).into_owned())
        .unwrap();
    assert!(
        branch.contains("if score @s") && branch.contains(" > @s "),
        "{branch}"
    );
}

#[test]
fn text_macros_interpolate_scores_names_and_styles() {
    let analysis = analyze_one(&pack(
        r#"
global state World { round: number }
fn announce() {
    broadcast!("{gold}{bold}Round {World.round}{reset} begins! {{braces}}");
}
fn greet() {
    title!("Hi {player}");
}
"#,
    ));
    let pack = compile(&analysis);
    let announce = function(&pack, "demo/function/announce");
    assert!(announce.starts_with("tellraw @a "), "{announce}");
    assert!(
        announce.contains(r#""bold":true,"color":"gold","text":"Round ""#),
        "{announce}"
    );
    assert!(announce.contains(r#""score":{"name":"#), "{announce}");
    assert!(
        announce.contains(r#""text":" begins! {braces}""#),
        "{announce}"
    );
    let greet = function(&pack, "demo/function/greet");
    assert!(greet.contains(r#"title @s title"#) && greet.contains(r#""selector":"@s""#));

    // `{player}` and player-facing macros make the function player-context.
    let def = analysis.defs.iter().find(|d| d.name == "greet").unwrap();
    let sand_lang::DefInfo::Function {
        context,
        player_reason,
        ..
    } = &def.info
    else {
        panic!()
    };
    assert_eq!(*context, sand_lang::Context::Player);
    assert!(player_reason.as_deref().unwrap().contains("title!"));
}

#[test]
fn text_errors_point_inside_the_string() {
    let text = pack(
        "player state S { v: number }\nfn f() {\n    chat!(\"a {S.nope} b {wat} c } d {open\");\n}\n",
    );
    let analysis = analyze_one(&text);
    assert_eq!(
        errors(&analysis),
        vec![
            "state `S` has no field `nope`",
            "unknown `{wat}` in text",
            "unmatched `}` in text",
            "unclosed `{` in text",
        ]
    );
    assert_eq!(spanned(&analysis, "no field"), "nope");
    assert_eq!(spanned(&analysis, "unknown `{wat}`"), "wat");
}

#[test]
fn builtins_lower_to_typed_commands() {
    let analysis = analyze_one(&pack(
        r#"
fn reward() {
    give_item(Item::Diamond, 3);
    add_effect(Effect::Speed, seconds = 10, level = 2, hide_particles = true);
    add_effect(Effect::NightVision);
    clear_effect(Effect::Poison);
    play_sound(Sound::EntityPlayerLevelup, volume = 0.5);
    spawn_particle(Particle::Heart, count = 5, spread = 0.5);
    clear_item(Item::Dirt, 2);
    clear_inventory();
}
"#,
    ));
    let pack = compile(&analysis);
    let reward = function(&pack, "demo/function/reward");
    for expected in [
        "give @s minecraft:diamond 3",
        "effect give @s minecraft:speed 10 1 true",
        "effect give @s minecraft:night_vision infinite",
        "effect clear @s minecraft:poison",
        "playsound minecraft:entity.player.levelup master @s",
        "particle minecraft:heart ~ ~1 ~ 0.5 0.5 0.5 0 5",
        "clear @s minecraft:dirt 2",
        "clear @s",
    ] {
        assert!(
            reward.contains(expected),
            "missing {expected:?} in\n{reward}"
        );
    }
}

#[test]
fn builtin_arguments_are_checked() {
    let analysis = analyze_one(&pack(
        r#"
fn f() {
    give_item("diamond");
    give_item(Item::Diamond, count = 0);
    add_effect(Effect::Speed, level = 300, colour = 2);
    play_sound();
    schedule(f);
    nope!("x");
}
"#,
    ));
    let errors = errors(&analysis);
    for expected in [
        "`diamond` is not a valid item identity",
        "expected a value from 1 to 6400, found 0",
        "`add_effect` has no argument named `colour`; expected `effect`, `seconds`, `level`, `hide_particles`",
        "`play_sound` needs `sound`: Sound event, such as `Sound::EntityPlayerLevelup`.",
        "`schedule` needs exactly one of `ticks` or `seconds`",
        "unknown macro `nope!`",
    ] {
        assert!(
            errors.iter().any(|e| e == expected),
            "missing {expected:?} in {errors:#?}"
        );
    }
}

#[test]
fn player_events_dispatch_through_namespace_owned_observers() {
    let analysis = analyze_one(&pack(
        r#"
player state S { deaths: number }
#[on(death)]
fn died() { S.deaths += 1; }
#[on(first_join)]
fn hello() { chat!("Welcome!"); }
#[on(use_item = Item::CarrotOnAStick)]
fn zap() { play_sound(Sound::EntityLightningBoltThunder); }
"#,
    ));
    let pack = compile(&analysis);
    let load = function(&pack, "demo/function/__sand_lifecycle_load");
    assert!(load.contains(" deathCount"), "{load}");
    assert!(
        load.contains(" minecraft.used:minecraft.carrot_on_a_stick"),
        "{load}"
    );
    let tick = function(&pack, "demo/function/__sand_lifecycle_tick");
    assert!(
        tick.contains("run function demo:__sand_program/event/death"),
        "{tick}"
    );
    assert!(tick.contains("execute as @a[tag=!sand."), "{tick}");
    let death = function(&pack, "demo/function/__sand_program/event/death");
    let lines: Vec<_> = death.lines().collect();
    assert!(
        lines[1].starts_with("scoreboard players reset @s"),
        "{death}"
    );
    assert_eq!(lines.last(), Some(&"function demo:died"));

    let bad = analyze_one(&format!("{PACK}#[on(teleport)]\nfn f() {{}}\n"));
    assert_eq!(errors(&bad), vec!["unknown event `teleport`"]);
}

#[test]
fn scheduled_functions_must_run_on_the_server() {
    let analysis = analyze_one(&pack(
        r#"
player state S { v: number }
fn later() { S.v += 1; }
fn ok() { for player in players() { S.v += 1; } }
#[tick]
fn tick() {
    schedule(later, ticks = 20);
    schedule(ok, seconds = 1);
}
"#,
    ));
    assert_eq!(
        errors(&analysis),
        vec![
            "`later` runs later on the server without a player or entity, but it writes player state `S.v`"
        ]
    );
    assert_eq!(spanned(&analysis, "runs later"), "later");
}

#[test]
fn hover_explains_builtins_attributes_events_styles_and_keywords() {
    let text = pack(
        r#"
player state S { v: number }
#[on(death)]
fn f() {
    chat!("{gold}hi");
    play_sound(Sound::BlockNoteBlockPling);
}
flow F on S.v { 0 -> 1; }
"#,
    );
    let analysis = analyze_one(&text);
    assert!(!analysis.has_errors(), "{:#?}", errors(&analysis));
    let hover = |needle: &str, delta: usize| {
        ide::hover(&analysis, FileId(0), at(&text, needle) + delta)
            .unwrap_or_else(|| panic!("no hover at {needle:?}"))
            .markdown
    };
    let sound = hover("play_sound(", 1);
    assert!(
        sound.contains("Play a sound to the executing player"),
        "{sound}"
    );
    assert!(sound.contains("Minecraft: `playsound"), "{sound}");
    assert!(sound.contains("`pitch` (optional)"), "{sound}");
    assert!(hover("chat!", 1).contains("`{State.field}` shows a live score"));
    assert!(hover("death", 1).contains("The player died"));
    assert!(hover("on(", 0).contains("whenever a player event happens"));
    assert!(hover("gold}", 1).contains("Colours the following text gold"));
    assert!(hover("flow F", 1).contains("state machine"));
}

#[test]
fn completion_offers_builtins_events_and_interpolations() {
    let text = pack("player state S { v: number }\n#[on(\nfn f() {\n    \n    chat!(\"{\");\n}\n");
    let analysis = analyze_one(&text);
    let labels = |offset: usize| -> Vec<String> {
        ide::completions(&analysis, FileId(0), offset)
            .into_iter()
            .map(|c| c.label)
            .collect()
    };
    let events = labels(at(&text, "#[on(") + 5);
    assert!(events.contains(&"death".to_string()) && events.contains(&"use_item".to_string()));
    let body = labels(at(&text, "    \n") + 4);
    for expected in [
        "chat!",
        "give_item",
        "add_effect",
        "schedule",
        "let",
        "while",
    ] {
        assert!(
            body.iter().any(|l| l == expected),
            "missing {expected} in {body:?}"
        );
    }
    assert!(
        !body.iter().any(|l| l == "random"),
        "values are not statements"
    );
    let inside = labels(at(&text, "{\");") + 1);
    for expected in ["player", "gold", "bold", "S.v"] {
        assert!(
            inside.iter().any(|l| l == expected),
            "missing {expected} in {inside:?}"
        );
    }
}

#[test]
fn statistic_events_observe_typed_filters() {
    let analysis = analyze_one(&pack(
        r#"
player state S { n: number }
#[on(jump)]
fn hop() { S.n += 1; }
#[on(mine_block = Block::DiamondOre)]
fn mined() { S.n += 10; }
#[on(kill = Entity::Zombie)]
fn slayer() { S.n += 1; }
#[on(killed_by = Entity::Creeper)]
fn boom() { S.n -= 1; }
#[on(craft_item = Item::Bread)]
fn baker() { S.n += 1; }
#[on(take_damage)]
fn ouch() { S.n -= 1; }
"#,
    ));
    let pack = compile(&analysis);
    let load = function(&pack, "demo/function/__sand_lifecycle_load");
    for criterion in [
        " minecraft.custom:minecraft.jump",
        " minecraft.mined:minecraft.diamond_ore",
        " minecraft.killed:minecraft.zombie",
        " minecraft.killed_by:minecraft.creeper",
        " minecraft.crafted:minecraft.bread",
        " minecraft.custom:minecraft.damage_taken",
    ] {
        assert!(load.contains(criterion), "missing {criterion:?} in\n{load}");
    }
    let tick = function(&pack, "demo/function/__sand_lifecycle_tick");
    assert!(
        tick.contains("run function demo:__sand_program/event/mine_block/minecraft/diamond_ore"),
        "{tick}"
    );

    let bad = analyze_one(&pack_src(
        "#[on(mine_block)]\nfn a() {}\n#[on(mine_block = Item::Diamond)]\nfn b() {}\n#[on(swim)]\nfn c() {}\n",
    ));
    let errors = errors(&bad);
    for expected in [
        "`mine_block` needs a `Block`",
        "expected a `Block`, found an `Item`",
        "unknown event `swim`",
    ] {
        assert!(
            errors.iter().any(|e| e == expected),
            "missing {expected:?} in {errors:#?}"
        );
    }
}

fn pack_src(body: &str) -> String {
    format!("{PACK}{body}")
}

#[test]
fn event_completion_lists_events_and_their_filter_values() {
    let text = pack_src("#[on()]\nfn f() {}\n#[on(mine_block = )]\nfn g() {}\n");
    let analysis = analyze_one(&text);
    let names: Vec<String> = ide::completions(&analysis, FileId(0), text.find("on()").unwrap() + 3)
        .into_iter()
        .map(|c| c.label)
        .collect();
    for expected in ["jump", "mine_block", "kill", "killed_by", "take_damage"] {
        assert!(
            names.contains(&expected.to_string()),
            "missing {expected} in {names:?}"
        );
    }
    let values: Vec<String> = ide::completions(
        &analysis,
        FileId(0),
        text.find("mine_block = )").unwrap() + 13,
    )
    .into_iter()
    .map(|c| c.label)
    .collect();
    assert!(values.contains(&"Block::DiamondOre".to_string()));
    assert!(
        !values.iter().any(|v| v.starts_with("Item::")),
        "only blocks"
    );
}
