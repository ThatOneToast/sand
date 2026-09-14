use sand::datapack_component;
use sand_core::{McFunction, ResourceLocation};

#[datapack_component]
fn real_definition() -> McFunction {
    McFunction::new(ResourceLocation::new("quoted_run", "definition").unwrap())
        .command("scoreboard objectives add phantom dummy")
}

#[datapack_component]
fn quoted_run_text() -> McFunction {
    McFunction::new(ResourceLocation::new("quoted_run", "quoted").unwrap()).command(
        "execute as @e[name=\" run scoreboard objectives add phantom trigger\"] run say hi",
    )
}

#[test]
fn quoted_run_text_is_not_mistaken_for_the_execute_subcommand_boundary() {
    let records = sand_core::try_export_components("quoted_run").unwrap();
    let quoted = records
        .iter()
        .find(|record| record.namespace == "quoted_run" && record.path == "quoted")
        .expect("quoted execute function must be exported");
    assert_eq!(
        quoted.content,
        "execute as @e[name=\" run scoreboard objectives add phantom trigger\"] run say hi"
    );
}
