//! Regression coverage for the `sand_commands::inventory` pre-write
//! re-validation registry (#172) staying scoped to a single export.
//!
//! `sand_commands::inventory`'s registry (mirroring `blocks`/`nbt`/etc's
//! established pattern) retains a rendered command line's typed node so
//! `validate_collected_line` can re-validate it structurally at export time
//! even when the line came from an infallible (non-panicking, non-fallible)
//! `Inventory` method. That registry is process-global
//! (`OnceLock<Mutex<BTreeMap<String, _>>>`), so without an explicit
//! per-export reset, an earlier export's registration for some rendered
//! line text persists into a later export in the same process: if that
//! later export (or a raw/hand-authored command, or an unrelated pack)
//! happens to render a *different* command that is byte-identical to the
//! earlier, now-stale entry, it gets incorrectly re-validated against that
//! stale typed node instead of being treated as its own, unrelated line.
//!
//! Since #293 this registry is no longer process-global at all: it lives in
//! `sand_commands::export_registry`'s thread-local, export-scoped layer,
//! created and destroyed by the `ExportRegistryGuard` that
//! `try_export_components_impl` takes as its first act. This test's
//! scenario — polluting the registry by calling `Inventory` directly
//! *outside* any export, then exporting a byte-identical raw line — now
//! passes because the export scope shadows the ambient layer the direct
//! call wrote to, rather than because a reset ran.

use sand_commands::{Inventory, ItemSlot, Target};
use sand_core::component::try_export_components_json as export_components_json;
use sand_core::function::FunctionDescriptor;
use std::sync::Mutex;

/// The real export path serializes every export through a process-wide
/// lock (crate-private to `sand-core`), so two real exports never
/// interleave. That lock isn't reachable from this integration-test crate,
/// and tests in one file run on parallel threads of one process by default
/// — so tests here that touch the inventory registry directly (via public
/// `Inventory` methods, outside of any export) serialize on this
/// test-local lock instead, to avoid one test's direct pollution leaking
/// into another concurrently-running test's export.
static TEST_LOCK: Mutex<()> = Mutex::new(());

fn contaminating_raw_line() -> String {
    // Deliberately not built via `Inventory` at all — this is what a
    // hand-authored raw `.mcfunction` command line (or an unrelated pack's
    // command) rendering the exact same text would look like. It must be
    // treated purely as unrecognized/raw syntax and pass through export.
    "item replace entity @s hotbar.* with regtest:contaminate_marker".to_string()
}

fn raw_line_pack_body() -> Vec<String> {
    vec![contaminating_raw_line()]
}

sand_core::inventory::submit! {
    FunctionDescriptor { path: "raw_line", make: raw_line_pack_body }
}

#[test]
fn stale_registry_entry_from_direct_inventory_use_does_not_leak_into_a_later_export() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    // Directly call the infallible, wildcard-slot-accepting `Inventory::set`
    // outside of any export. This never panics, but it registers the exact
    // same line text as `raw_line_pack_body` renders above, tagged as an
    // *invalid* wildcard-slot write.
    let contaminating_line =
        Inventory::of(Target::self_()).set(ItemSlot::AnyHotbar, "regtest:contaminate_marker");
    assert_eq!(
        contaminating_line,
        contaminating_raw_line(),
        "test setup bug: the polluting line must exactly match the pack's raw line"
    );

    // If the registry reset at export start didn't happen, this export
    // would incorrectly fail: `validate_collected_line` would find the
    // stale wildcard-write registration above and reject `regtest:raw_line`
    // even though it never went through `Inventory` this export.
    let result = export_components_json("regtest_invpack");
    assert!(
        result.is_ok(),
        "an unrelated prior direct `Inventory::set` call must not contaminate a later export's \
         validation of a byte-identical raw command line: {:?}",
        result.err()
    );

    let records: Vec<serde_json::Value> = serde_json::from_str(&result.unwrap()).unwrap();
    let raw_record = records
        .iter()
        .find(|record| record["path"] == "raw_line")
        .expect("missing generated record `regtest:raw_line`");
    assert_eq!(raw_record["content"], contaminating_raw_line());
}

#[test]
fn two_back_to_back_exports_in_the_same_process_are_byte_identical() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let first = export_components_json("regtest_invpack");
    let second = export_components_json("regtest_invpack");
    assert_eq!(
        first.is_ok(),
        second.is_ok(),
        "repeated exports in the same process must succeed/fail identically"
    );
    assert_eq!(
        first.unwrap(),
        second.unwrap(),
        "repeated exports in the same process must be byte-identical"
    );
}
