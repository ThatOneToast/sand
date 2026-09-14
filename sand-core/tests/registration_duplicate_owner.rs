use sand_core::{ComponentFactory, DatapackRegistration};

fn must_not_run() -> DatapackRegistration {
    panic!("duplicate owners must be rejected before factory execution")
}

inventory::submit!(ComponentFactory {
    owner: "duplicate_owner",
    make: must_not_run,
});

inventory::submit!(ComponentFactory {
    owner: "duplicate_owner",
    make: must_not_run,
});

#[test]
fn duplicate_factory_owners_fail_before_factories_are_invoked() {
    let error = sand_core::try_export_components("duplicate_owner").unwrap_err();
    let message = error.to_string();
    assert!(
        message.contains("duplicate component factory owner"),
        "{message}"
    );
    assert!(message.contains("duplicate_owner"), "{message}");
}
