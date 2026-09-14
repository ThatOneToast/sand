use sand::datapack_component;
use sand_core::{McFunction, ResourceLocation};

#[datapack_component]
fn direct_definition() -> McFunction {
    McFunction::new(ResourceLocation::new("nested_objective", "direct").unwrap())
        .command("scoreboard objectives add nested_conflict dummy")
}

#[datapack_component]
fn execute_wrapped_definition() -> McFunction {
    McFunction::new(ResourceLocation::new("nested_objective", "wrapped").unwrap())
        .command("execute as @a run scoreboard objectives add nested_conflict trigger")
}

#[test]
fn execute_wrapped_objective_definitions_participate_in_conflict_validation() {
    let error = sand_core::try_export_components("nested_objective").unwrap_err();
    let message = error.to_string();
    assert!(message.contains("nested_conflict"), "{message}");
    assert!(
        message.contains("nested_objective:function/direct"),
        "{message}"
    );
    assert!(
        message.contains("nested_objective:function/wrapped"),
        "{message}"
    );
}
