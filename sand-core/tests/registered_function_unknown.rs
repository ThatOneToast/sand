use sand::component::{IntoCommands, McFunction};
use sand::prelude::*;
use sand_core::ir::Cmd;
use std::sync::OnceLock;

#[datapack_component]
fn missing_callback() -> McFunction {
    static ACTIONS: OnceLock<Actions> = OnceLock::new();
    McFunction::new(ResourceLocation::new("__sand_local", "callback_owner").unwrap()).commands(
        ACTIONS
            .get_or_init(|| Cmd::RegisteredFunction(FunctionId::local("missing")).into_commands())
            .clone(),
    )
}
#[test]
fn unknown_cached_function_id_is_rejected_on_every_export_with_owner_and_callback() {
    let export = |namespace| {
        let error = sand_core::try_export_components(namespace)
            .unwrap_err()
            .to_string();
        assert!(error.contains("callback_owner"), "{error}");
        assert!(
            error.contains(&format!("callback `{namespace}:missing`")),
            "{error}"
        );
        assert!(error.contains("not a registered #[function]"), "{error}");
    };
    for namespace in ["first", "second", "first"] {
        export(namespace);
    }
    std::thread::scope(|scope| {
        let first = scope.spawn(|| export("first"));
        let second = scope.spawn(|| export("second"));
        first.join().unwrap();
        second.join().unwrap();
    });
}
