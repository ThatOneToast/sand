use sand_core::component::{try_export_components_json, try_export_components_json_for_version};
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
    let values = Nbt::storage("macro_test:runtime").path("greeting");
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

#[test]
fn registered_function_macro_is_rejected_before_minecraft_1_20_2() {
    let resolved = sand_core::version::resolve_export_caps("1.20.1").unwrap();
    let error = try_export_components_json_for_version(
        "macro_test",
        &resolved.caps,
        &resolved.version,
        resolved.is_fallback,
    )
    .expect_err("function macro lines require Minecraft 1.20.2+")
    .to_string();
    assert!(error.contains("macro_test:greet"), "{error}");
    assert!(error.contains("SAND-COMMAND-VERSION"), "{error}");
    assert!(error.contains("Minecraft 1.20.2+"), "{error}");
}
