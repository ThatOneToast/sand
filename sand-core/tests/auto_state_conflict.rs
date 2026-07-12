use sand_core::{StateDescriptor, StateLifecycle};

sand_core::inventory::submit! {
    StateDescriptor::new(StateLifecycle::score("same_state").default(10))
}

sand_core::inventory::submit! {
    StateDescriptor::new(StateLifecycle::score("same_state").default(20))
}

#[test]
fn conflicting_automatic_declarations_fail_the_fallible_export() {
    let error = sand_core::try_export_components_json("conflict_pack").unwrap_err();
    let message = error.to_string();
    assert!(message.contains("conflicting automatic state `same_state`"));
    assert!(message.contains("default Some(10)"));
    assert!(message.contains("default Some(20)"));
}
