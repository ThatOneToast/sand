use sand_core::component::try_export_components_json;
use sand_core::prelude::*;
use sand_macros::function;

#[function("greet")]
fn greet() {
    let args = FunctionMacroArgs::new(["player", "count"]).unwrap();
    let player = args.variable("player").unwrap();
    let count = args.variable("count").unwrap();
    args.line(format!("say Hello, {player}!")).unwrap();
    args.line(format!("give {player} minecraft:diamond {count}"))
        .unwrap();
}

#[function("run_greeting")]
fn run_greeting() {
    let args = FunctionMacroArgs::new(["player", "count"]).unwrap();
    let values =
        Nbt::storage(ResourceLocation::new("macro_test", "runtime").unwrap()).path("greeting");
    args.call_with(greet, &values).unwrap();
}

#[test]
fn registered_function_macro_exports_typed_placeholders_and_call() {
    let records: Vec<serde_json::Value> =
        serde_json::from_str(&try_export_components_json("macro_test").unwrap()).unwrap();

    let greet_record = records
        .iter()
        .find(|record| record["path"] == "greet")
        .expect("missing greet function");
    assert_eq!(
        greet_record["content"],
        "$say Hello, $(player)!\n$give $(player) minecraft:diamond $(count)"
    );

    let caller_record = records
        .iter()
        .find(|record| record["path"] == "run_greeting")
        .expect("missing caller function");
    assert_eq!(
        caller_record["content"],
        "function macro_test:greet with storage macro_test:runtime greeting"
    );
}

#[function("scoped_greeting")]
fn scoped_greeting() {
    let player = sand_core::entity::EntityContext::<sand_core::entity::PlayerKind>::default();
    sand_core::entity::EntityScope::bind(&player, |_| {
        let args = FunctionMacroArgs::new(["player"]).unwrap();
        args.line("say scoped $(player)").unwrap()
    });
}

#[test]
fn scoped_macro_lines_keep_the_argument_bearing_function_and_cleanup() {
    let first = try_export_components_json("macro_test").unwrap();
    assert_eq!(first, try_export_components_json("macro_test").unwrap());
    let records: Vec<serde_json::Value> = serde_json::from_str(&first).unwrap();
    let body = records
        .iter()
        .find(|record| record["path"] == "scoped_greeting")
        .unwrap()["content"]
        .as_str()
        .unwrap();
    let lines: Vec<_> = body.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("tag @s add __sand_scope_"));
    assert_eq!(lines[1], "$say scoped $(player)");
    assert!(lines[2].starts_with("tag @e[tag=__sand_scope_"));
    assert!(lines[2].contains(" remove __sand_scope_"));
}
