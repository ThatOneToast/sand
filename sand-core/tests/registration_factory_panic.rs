use sand::datapack_component;
use sand_core::McFunction;

#[datapack_component]
fn panicking_factory() -> McFunction {
    panic!("intentional registration panic")
}

#[test]
fn factory_panic_becomes_a_structured_export_diagnostic() {
    // Restoring a wrapper instead of the original hook grows a permanent
    // chain on every export. Verify the original allocation survives intact.
    let hook = std::panic::take_hook();
    let original = (&*hook as *const _) as *const ();
    std::panic::set_hook(hook);
    for _ in 0..32 {
        assert!(sand_core::try_export_components("panic_registration").is_err());
    }
    let hook = std::panic::take_hook();
    let restored = (&*hook as *const _) as *const ();
    std::panic::set_hook(hook);
    assert_eq!(original, restored, "export must restore the original hook");

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
