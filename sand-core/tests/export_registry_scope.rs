//! End-to-end coverage that typed command registries are scoped to one
//! export, driven through the real export pipeline (#293).
//!
//! The per-family lifecycle properties are asserted against each family's
//! own storage in `sand-commands`' `export_registry::family_coverage`
//! harness. What that harness *cannot* show is the consequence: that a
//! stale registration from an earlier export used to change a later,
//! unrelated export's verdict. These tests show exactly that, using the
//! only lever that produces two different verdicts for one byte-identical
//! line — a version-gated typed node:
//!
//! `execute if items entity @s weapon.mainhand minecraft:diamond run say found`
//!
//! * Built through typed [`Execute::if_items`], it registers an
//!   `ExecuteItemCondition` capability requirement (Minecraft 1.20.5+), so
//!   exporting it under 1.20.4 **fails** with `SAND-COMMAND-VERSION`.
//! * Written by hand as a raw string, it registers nothing, is opaque, and
//!   exports fine under *any* version — raw lines are never validated for
//!   capabilities the user did not ask Sand to model.
//!
//! So "does an export of the raw form under 1.20.4 succeed?" is a direct
//! read-out of whether a previous export's typed registration leaked.
//!
//! The pack body is chosen per thread (`PHASE`), so tests in this file —
//! which the harness runs concurrently — never fight over one global
//! switch, and a concurrency test can genuinely have two threads exporting
//! *different* bodies under *different* profiles at the same time.

use std::cell::Cell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use sand_commands::{Execute, ItemSlot, Selector};
use sand_core::function::FunctionDescriptor;

/// The one command text both phases produce, byte for byte.
const SHARED_LINE: &str =
    "execute if items entity @s weapon.mainhand minecraft:diamond run say found";

/// Minecraft version at which `if items` (`ExecuteItemCondition`) exists.
const SUPPORTS_IF_ITEMS: &str = "1.20.5";
/// Minecraft version at which it does not.
const LACKS_IF_ITEMS: &str = "1.20.4";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// Emit `SHARED_LINE` via the typed builder, registering a
    /// version-gated node for it.
    Typed,
    /// Emit `SHARED_LINE` as an opaque raw string, registering nothing.
    Raw,
}

thread_local! {
    static PHASE: Cell<Phase> = const { Cell::new(Phase::Raw) };
}

fn set_phase(phase: Phase) {
    PHASE.with(|cell| cell.set(phase));
}

/// The pack's only function body. Runs on the exporting thread, so it sees
/// that thread's `PHASE`.
fn phase_body() -> Vec<String> {
    match PHASE.with(Cell::get) {
        Phase::Typed => {
            let line = Execute::new()
                .if_items(Selector::self_(), ItemSlot::MainHand, "minecraft:diamond")
                .run_raw("say found");
            assert_eq!(
                line, SHARED_LINE,
                "test setup: the typed and raw phases must render identical text"
            );
            vec![line]
        }
        Phase::Raw => vec![SHARED_LINE.to_string()],
    }
}

sand_core::inventory::submit! {
    FunctionDescriptor { path: "phase_line", make: phase_body }
}

fn export_at(version: &str) -> Result<String, String> {
    sand_core::advanced::try_export_components_json("regtest_scopepack", version)
        .map_err(|error| error.to_string())
}

fn hash_of(json: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    json.hash(&mut hasher);
    hasher.finish()
}

fn phase_line_content(json: &str) -> String {
    let records: Vec<serde_json::Value> = serde_json::from_str(json).unwrap();
    records
        .iter()
        .find(|record| record["path"] == "phase_line")
        .expect("missing generated record `regtest_scopepack:phase_line`")["content"]
        .as_str()
        .unwrap()
        .to_string()
}

/// The premise every other test here rests on: within *one* export, a typed
/// line really is re-validated against the active profile.
#[test]
fn a_typed_line_still_revalidates_against_the_active_profile() {
    set_phase(Phase::Typed);
    export_at(SUPPORTS_IF_ITEMS).expect("typed `if items` must export on 1.20.5");

    let error = export_at(LACKS_IF_ITEMS).expect_err(
        "typed `if items` must be rejected on 1.20.4, or nothing here is being checked",
    );
    assert!(error.contains("SAND-COMMAND-VERSION"), "{error}");
    assert!(error.contains("ExecuteItemCondition"), "{error}");
}

/// The core regression: a successful export registers a typed node; the
/// next export emits byte-identical text that was never built from types
/// and must not inherit the earlier export's verdict.
#[test]
fn a_previous_exports_typed_node_does_not_validate_a_later_identical_raw_line() {
    set_phase(Phase::Typed);
    export_at(SUPPORTS_IF_ITEMS).expect("test setup: the typed export must succeed on 1.20.5");

    set_phase(Phase::Raw);
    let json = export_at(LACKS_IF_ITEMS).unwrap_or_else(|error| {
        panic!(
            "a raw, hand-authored command line must not be re-validated against a \
             byte-identical typed node registered by an *earlier* export: {error}"
        )
    });
    assert_eq!(
        phase_line_content(&json),
        SHARED_LINE,
        "raw lines must reach the output opaque and unmodified"
    );
}

/// The case a manual reset-at-the-end-of-the-happy-path gets wrong: the
/// leaking export never reaches its own cleanup.
#[test]
fn a_failed_export_does_not_leak_state_into_the_next_one() {
    set_phase(Phase::Typed);
    let error = export_at(LACKS_IF_ITEMS).expect_err("test setup: this export must fail");
    assert!(error.contains("SAND-COMMAND-VERSION"), "{error}");

    set_phase(Phase::Raw);
    export_at(LACKS_IF_ITEMS).unwrap_or_else(|error| {
        panic!("an export that failed partway must not leave registrations behind: {error}")
    });
}

/// Raw command text is never validated as a side effect of *any* family
/// having modelled something that renders the same way.
#[test]
fn raw_lines_pass_through_every_profile_unchanged() {
    set_phase(Phase::Raw);
    for version in ["1.18.0", LACKS_IF_ITEMS, SUPPORTS_IF_ITEMS, "1.21.4"] {
        let json = export_at(version)
            .unwrap_or_else(|error| panic!("raw line must export on {version}: {error}"));
        assert_eq!(phase_line_content(&json), SHARED_LINE);
    }
}

#[test]
fn repeated_exports_in_one_process_are_byte_identical() {
    set_phase(Phase::Raw);
    let first = export_at(SUPPORTS_IF_ITEMS).unwrap();
    let second = export_at(SUPPORTS_IF_ITEMS).unwrap();
    let third = export_at(SUPPORTS_IF_ITEMS).unwrap();
    assert_eq!(first, second);
    assert_eq!(second, third);
    assert_eq!(hash_of(&first), hash_of(&third));
}

/// Two threads exporting *different* pack bodies under *different*
/// profiles, repeatedly and at the same time. The typed thread must always
/// succeed (its node is supported on its profile) and the raw thread must
/// always succeed (its line is opaque) — either failing means one thread's
/// registrations were visible to the other.
#[test]
fn concurrent_exports_on_different_threads_do_not_contaminate_each_other() {
    const ROUNDS: usize = 12;

    let typed = std::thread::spawn(|| {
        set_phase(Phase::Typed);
        for round in 0..ROUNDS {
            export_at(SUPPORTS_IF_ITEMS)
                .unwrap_or_else(|error| panic!("typed export round {round} failed: {error}"));
        }
    });
    let raw = std::thread::spawn(|| {
        set_phase(Phase::Raw);
        let mut outputs = Vec::new();
        for round in 0..ROUNDS {
            outputs.push(export_at(LACKS_IF_ITEMS).unwrap_or_else(|error| {
                panic!(
                    "a concurrent export's typed `if items` registration leaked into this \
                         thread's raw export (round {round}): {error}"
                )
            }));
        }
        outputs
    });

    typed.join().unwrap();
    let outputs = raw.join().unwrap();
    assert!(
        outputs.windows(2).all(|pair| pair[0] == pair[1]),
        "concurrent exports must still be deterministic"
    );
}

/// Nested/reentrant exports are explicitly unsupported and diagnosed rather
/// than silently sharing a scope (or, as before this guard existed,
/// deadlocking on the export pipeline's non-reentrant dialog lock).
#[test]
fn a_nested_export_is_diagnosed_rather_than_sharing_a_scope() {
    set_phase(Phase::Raw);
    let outer =
        sand_commands::ExportRegistryGuard::enter().expect("no scope active on this thread");

    let error = export_at(SUPPORTS_IF_ITEMS)
        .expect_err("an export started inside an open export scope must be rejected");
    assert!(
        error.contains(sand_commands::NestedExportError::CODE),
        "nested export must be diagnosed with its own code: {error}"
    );

    drop(outer);
    export_at(SUPPORTS_IF_ITEMS).expect("a normal export works again once the outer scope closes");
}
