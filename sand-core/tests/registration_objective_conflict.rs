use sand::datapack_component;
use sand_core::{DatapackRegistration, LifecycleContribution};

#[datapack_component]
fn dummy_objective() -> DatapackRegistration {
    DatapackRegistration::new().lifecycle(LifecycleContribution::load(
        "scoreboard objectives add shared_objective dummy",
    ))
}

#[datapack_component]
fn trigger_objective() -> DatapackRegistration {
    DatapackRegistration::new().lifecycle(LifecycleContribution::load(
        "scoreboard objectives add shared_objective trigger",
    ))
}

#[test]
fn conflicting_registration_objectives_name_both_owners() {
    let error = sand_core::try_export_components("objective_conflict").unwrap_err();
    let message = error.to_string();
    assert!(message.contains("shared_objective"), "{message}");
    assert!(message.contains("dummy_objective"), "{message}");
    assert!(message.contains("trigger_objective"), "{message}");
}
