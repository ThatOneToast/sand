use sand::datapack_component;
use sand_core::{McFunction, ResourceLocation};

fn duplicate() -> McFunction {
    McFunction::new(ResourceLocation::new("duplicate_across", "same").unwrap())
}

#[datapack_component]
fn first_owner() -> McFunction {
    duplicate()
}

#[datapack_component]
fn second_owner() -> McFunction {
    duplicate()
}

#[test]
fn duplicate_identities_from_separate_registrations_fail_with_both_owners() {
    let error = sand_core::try_export_components("duplicate_across").unwrap_err();
    let message = error.to_string();
    assert!(
        message.contains("duplicate_across:function/same.mcfunction"),
        "{message}"
    );
    assert!(message.contains("first_owner"), "{message}");
    assert!(message.contains("second_owner"), "{message}");
}
