use sand::datapack_component;
use sand_core::McFunction;

#[datapack_component]
fn panicking_factory() -> McFunction {
    panic!("intentional registration panic")
}

#[test]
fn factory_panic_becomes_a_structured_export_diagnostic() {
    let error = sand_core::try_export_components("panic_registration").unwrap_err();
    let message = error.to_string();
    assert!(message.contains("sand:registration"), "{message}");
    assert!(message.contains("registration"), "{message}");
    assert!(message.contains("panicking_factory"), "{message}");
    assert!(
        message.contains("intentional registration panic"),
        "{message}"
    );
}
