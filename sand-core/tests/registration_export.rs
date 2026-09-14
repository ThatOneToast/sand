use sand::datapack_component;
use sand_core::{
    DatapackRegistration, FunctionId, FunctionTagContribution, IntoDatapack, LifecycleContribution,
    McFunction, ResourceLocation,
};

#[datapack_component]
fn ordinary_component() -> McFunction {
    McFunction::new(ResourceLocation::new("registration_export", "ordinary").unwrap())
        .command("say ordinary")
}

#[datapack_component(Tag = "registration_export:feature_entrypoints")]
fn descriptor_tag_member() {
    sand_core::cmd::say("descriptor tag member");
}

struct FutureFeature;

impl IntoDatapack for FutureFeature {
    fn into_datapack(self) -> DatapackRegistration {
        DatapackRegistration::new()
            .components([
                McFunction::new(
                    ResourceLocation::new("registration_export", "bundle/zeta").unwrap(),
                )
                .command("say zeta"),
                McFunction::new(
                    ResourceLocation::new("registration_export", "bundle/alpha").unwrap(),
                )
                .command("say alpha"),
            ])
            .lifecycle(LifecycleContribution::load("say registration load first"))
            .lifecycle(LifecycleContribution::load("say registration load second"))
            .lifecycle(LifecycleContribution::tick("say registration tick first"))
            .lifecycle(LifecycleContribution::tick("say registration tick second"))
            .function_tag(FunctionTagContribution::new(
                ResourceLocation::new("registration_export", "feature_entrypoints").unwrap(),
                FunctionId::custom(
                    ResourceLocation::new("registration_export", "bundle/alpha").unwrap(),
                ),
            ))
    }
}

#[datapack_component]
fn future_feature() -> FutureFeature {
    FutureFeature
}

fn record<'a>(
    records: &'a [sand_core::ComponentRecord],
    namespace: &str,
    dir: &str,
    path: &str,
) -> &'a sand_core::ComponentRecord {
    records
        .iter()
        .find(|record| record.namespace == namespace && record.dir == dir && record.path == path)
        .unwrap_or_else(|| panic!("missing {namespace}:{dir}/{path}"))
}

#[test]
fn generalized_registration_uses_canonical_component_lifecycle_and_tag_output() {
    let records = sand_core::try_export_components("registration_export").unwrap();

    assert_eq!(
        record(&records, "registration_export", "function", "ordinary").content,
        "say ordinary"
    );
    assert_eq!(
        record(&records, "registration_export", "function", "bundle/alpha").content,
        "say alpha"
    );
    assert_eq!(
        record(&records, "registration_export", "function", "bundle/zeta").content,
        "say zeta"
    );

    let load = record(
        &records,
        "registration_export",
        "function",
        "__sand_lifecycle_load",
    );
    assert_eq!(
        load.content,
        "say registration load first\nsay registration load second"
    );
    let tick = record(
        &records,
        "registration_export",
        "function",
        "__sand_lifecycle_tick",
    );
    assert_eq!(
        tick.content,
        "say registration tick first\nsay registration tick second"
    );

    let custom_tag = record(
        &records,
        "registration_export",
        "tags/function",
        "feature_entrypoints",
    );
    let json: serde_json::Value = serde_json::from_str(&custom_tag.content).unwrap();
    assert_eq!(
        json["values"],
        serde_json::json!([
            "registration_export:bundle/alpha",
            "registration_export:descriptor_tag_member"
        ])
    );

    let load_tag = record(&records, "minecraft", "tags/function", "load");
    let load_json: serde_json::Value = serde_json::from_str(&load_tag.content).unwrap();
    assert_eq!(
        load_json["values"],
        serde_json::json!(["registration_export:__sand_lifecycle_load"])
    );
    let tick_tag = record(&records, "minecraft", "tags/function", "tick");
    let tick_json: serde_json::Value = serde_json::from_str(&tick_tag.content).unwrap();
    assert_eq!(
        tick_json["values"],
        serde_json::json!(["registration_export:__sand_lifecycle_tick"])
    );
}

#[test]
fn repeated_and_concurrent_exports_are_byte_identical_and_isolated() {
    let expected = sand_core::try_export_components_json("registration_export").unwrap();
    assert_eq!(
        sand_core::try_export_components_json("registration_export").unwrap(),
        expected
    );

    let exports: Vec<_> = (0..4)
        .map(|_| {
            std::thread::spawn(|| {
                sand_core::try_export_components_json("registration_export").unwrap()
            })
        })
        .map(|thread| thread.join().unwrap())
        .collect();
    assert!(exports.iter().all(|export| export == &expected));
}
