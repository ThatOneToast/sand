//! Export/build-level regression coverage for #172's inventory validation
//! boundary: an invalid inventory command produced by the infallible
//! (non-panicking) `Inventory` builder path must still surface as a Sand
//! diagnostic and fail the export, rather than being written to a datapack.
//!
//! `Inventory`'s ordinary methods (`give`, `set`, `clear_slot`, `copy_from`,
//! `modify`, …) never panic on invalid input (see #172), but the rendered
//! line's typed node is retained in a pre-write registry so
//! `validate_collected_line` — invoked from every real export via
//! `validate_function_records` — re-validates it before any record is
//! accepted. This is a separate test binary from
//! `inventory_registry_export.rs` because `sand_core::inventory::submit!`
//! registrations are process-global for the whole test binary (not scoped
//! by the `namespace` argument to `try_export_components_json`): keeping
//! this deliberately-invalid fixture in its own file/process keeps it from
//! being included in every export in that other file's tests.

use sand_commands::{Inventory, ItemSlot, Selector};
use sand_core::component::try_export_components_json as export_components_json;
use sand_core::function::FunctionDescriptor;

fn wildcard_write_pack_body() -> Vec<String> {
    // The infallible, non-panicking `Inventory::set` path: it must not
    // panic on a wildcard-slot write, but the rendered line is registered
    // so export-time validation still catches it (see #172's acceptance
    // criterion: "route inventory output through exporter validation").
    vec![Inventory::of(Selector::self_()).set(ItemSlot::AnyHotbar, "regtest:direct_wildcard")]
}

sand_core::inventory::submit! {
    FunctionDescriptor { path: "wildcard_write", make: wildcard_write_pack_body }
}

#[test]
fn export_rejects_invalid_inventory_command_from_infallible_builder_before_writing_a_datapack() {
    let error = export_components_json("regtest_wildcardpack")
        .expect_err(
            "a wildcard-slot write from the infallible `Inventory::set` path must fail export",
        )
        .to_string();
    assert!(
        error.contains("SAND-INVENTORY-WILDCARD-WRITE"),
        "expected the inventory wildcard-write diagnostic code, got: {error}"
    );
    assert!(
        error.contains("regtest_wildcardpack:wildcard_write"),
        "diagnostic must identify the offending function, got: {error}"
    );
}
