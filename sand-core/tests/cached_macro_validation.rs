//! Macro prefixes must preserve typed validation through cached export bodies.
use sand::component::IntoCommands;
use sand::prelude::*;
use std::sync::OnceLock;

static BODY: OnceLock<Actions> = OnceLock::new();
fn body() -> Actions {
    BODY.get_or_init(|| {
        let invalid = sand_commands::Inventory::of(sand_commands::Target::self_())
            .set(sand_commands::ItemSlot::AnyHotbar, "cached:marker");
        vec![format!("$execute as @a run return run {invalid}")].into_commands()
    })
    .clone()
}
sand_core::inventory::submit! {
    sand_core::FunctionDescriptor { owner: module_path!(),
        context: sand::advanced::compiler::ExecutionContext::Server,
        path: "cached_invalid_macro",
        make: body,
    }
}
#[test]
fn cached_macro_strings_keep_validation_in_serial_and_parallel_exports() {
    let _cached = body();
    let check = || {
        let error = sand_core::try_export_components_json("cached")
            .unwrap_err()
            .to_string();
        assert!(error.contains("SAND-INVENTORY-WILDCARD-WRITE"), "{error}");
        assert!(error.contains("cached_invalid_macro"), "{error}");
    };
    for _ in 0..3 {
        check();
    }
    let threads: Vec<_> = (0..4).map(|_| std::thread::spawn(check)).collect();
    for thread in threads {
        thread.join().unwrap();
    }
}
