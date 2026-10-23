use sand::component::McFunction;
use sand::prelude::*;
use sand_core::ir::Cmd;
use std::sync::OnceLock;

#[function]
fn generic_callback() {
    cmd::say("generic");
}
#[function("player_callback", context = player)]
fn player_callback() {
    cmd::say("player");
}

fn cached() -> Actions {
    static ACTIONS: OnceLock<Actions> = OnceLock::new();
    ACTIONS
        .get_or_init(|| {
            let mut body = Actions::default();
            body.extend([
                Cmd::RegisteredFunction(generic_callback.function_id()),
                Cmd::ReturnRun(Box::new(Cmd::RegisteredFunction(
                    player_callback.function_id(),
                ))),
            ]);
            body
        })
        .clone()
}
#[datapack_component]
fn cached_calls() -> McFunction {
    McFunction::new(ResourceLocation::new("__sand_local", "cached_calls").unwrap())
        .commands(cached())
}
thread_local! {
    static EXPLICIT_NAMESPACE: std::cell::Cell<&'static str> = const { std::cell::Cell::new("registered") };
}
#[datapack_component]
fn explicit_calls() -> McFunction {
    let namespace = EXPLICIT_NAMESPACE.get();
    let mut actions = Actions::default();
    actions.extend([
        Cmd::RegisteredFunction(format!("{namespace}:generic_callback").parse().unwrap()),
        Cmd::RegisteredFunction(format!("{namespace}:player_callback").parse().unwrap()),
    ]);
    McFunction::new(ResourceLocation::new("__sand_local", "explicit_calls").unwrap())
        .commands(actions)
}
#[test]
fn cached_calls_resolve_each_export_and_guard_canonical_player_context() {
    let error = Cmd::RegisteredFunction(generic_callback.function_id())
        .try_render()
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("registered function calls require function export lowering"),
        "{error}"
    );
    // Construct before any active export; cached actions retain typed IDs.
    let _ = cached();
    let export = |namespace| {
        EXPLICIT_NAMESPACE.set(namespace);
        let records = sand_core::try_export_components(namespace).unwrap();
        let body = &records
            .iter()
            .find(|r| r.path == "cached_calls")
            .unwrap()
            .content;
        assert_eq!(
            body,
            &format!(
                "function {namespace}:generic_callback\nreturn run execute if entity @s[type=minecraft:player] run function {namespace}:player_callback"
            )
        );
        let explicit = &records
            .iter()
            .find(|r| r.path == "explicit_calls")
            .unwrap()
            .content;
        assert_eq!(
            explicit,
            &format!(
                "function {namespace}:generic_callback\nexecute if entity @s[type=minecraft:player] run function {namespace}:player_callback"
            )
        );
        records
    };
    let first = export("registered");
    assert_eq!(first, export("registered"));
    std::thread::scope(|scope| {
        let same = scope.spawn(|| export("registered"));
        let other = scope.spawn(|| export("other"));
        assert_eq!(first, same.join().unwrap());
        other.join().unwrap();
    });
}
