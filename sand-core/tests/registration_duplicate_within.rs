use sand::datapack_component;
use sand_core::{DatapackRegistration, McFunction, ResourceLocation};

#[datapack_component]
fn duplicate_bundle() -> DatapackRegistration {
    DatapackRegistration::new().components([
        McFunction::new(ResourceLocation::new("duplicate_within", "same").unwrap()),
        McFunction::new(ResourceLocation::new("duplicate_within", "same").unwrap()),
    ])
}

#[test]
fn duplicate_identities_within_one_registration_fail_at_the_same_boundary() {
    let error = sand_core::try_export_components("duplicate_within").unwrap_err();
    let message = error.to_string();
    assert!(
        message.contains("duplicate_within:function/same.mcfunction"),
        "{message}"
    );
    assert!(message.contains("duplicate_bundle"), "{message}");
}
