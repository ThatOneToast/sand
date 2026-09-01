use sand_core::entity::{StateFieldDescriptor, StateFieldKind};
use sand_core::{StateDescriptor, StateLifecycleDescriptor, StateScope};

const FIRST_FIELDS: &[StateLifecycleDescriptor] = &[StateLifecycleDescriptor::new(
    "same_state",
    StateFieldDescriptor::new("value", StateFieldKind::Score, 10, None),
)];
const SECOND_FIELDS: &[StateLifecycleDescriptor] = &[StateLifecycleDescriptor::new(
    "same_state",
    StateFieldDescriptor::new("value", StateFieldKind::Score, 20, None),
)];

sand_core::inventory::submit! {
    StateDescriptor::new(
        "test:same_state", 1, StateScope::Player, "same_present", "same_suppressed",
        FIRST_FIELDS, &[], &[],
    )
}

sand_core::inventory::submit! {
    StateDescriptor::new(
        "test:same_state", 1, StateScope::Player, "same_present", "same_suppressed",
        SECOND_FIELDS, &[], &[],
    )
}

#[test]
fn conflicting_automatic_declarations_fail_the_fallible_export() {
    let error = sand_core::try_export_components_json("conflict_pack").unwrap_err();
    let message = error.to_string();
    assert!(message.contains("conflicting State component declarations for `test:same_state`"));
}
