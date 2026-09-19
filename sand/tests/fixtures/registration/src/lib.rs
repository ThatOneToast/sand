//! External authoring fixture: its only dependency is the supported façade.
use sand::component::McFunction;
use sand::datapack_component as component;
use sand::prelude::{FunctionId, ResourceLocation};
use sand::registration::{
    DatapackRegistration, FunctionTagContribution, IntoDatapack, LifecycleContribution,
};

struct ExampleBundle;

impl IntoDatapack for ExampleBundle {
    fn into_datapack(self) -> DatapackRegistration {
        DatapackRegistration::new()
            .components([
                McFunction::new("consumer:alpha".parse().unwrap())
                    .command(sand::cmd::say("alpha").to_string()),
                McFunction::new("consumer:zeta".parse().unwrap())
                    .command(sand::cmd::say("zeta").to_string()),
            ])
            .lifecycle(LifecycleContribution::load(
                sand::cmd::say("bundle load").to_string(),
            ))
            .lifecycle(LifecycleContribution::tick(
                sand::cmd::say("bundle tick").to_string(),
            ))
            .function_tag(FunctionTagContribution::new(
                ResourceLocation::new("consumer", "entries").unwrap(),
                FunctionId::custom("consumer:alpha".parse().unwrap()),
            ))
    }
}

#[component]
fn example_bundle() -> ExampleBundle {
    ExampleBundle
}

#[component]
fn ordinary_component() -> McFunction {
    McFunction::new("consumer:ordinary".parse().unwrap())
        .command(sand::cmd::say("ordinary").to_string())
}

#[test]
fn facade_only_bundle_and_ordinary_component_export() {
    // This is Sand's supported build hook, not an exporter-internal API.
    let json = sand::advanced::try_export_components_json("consumer", "26.2").unwrap();
    for expected in [
        "say alpha",
        "say zeta",
        "say ordinary",
        "say bundle load",
        "say bundle tick",
        "consumer:alpha",
        "consumer:__sand_lifecycle_load",
        "consumer:__sand_lifecycle_tick",
    ] {
        assert!(json.contains(expected), "missing {expected}: {json}");
    }
}
