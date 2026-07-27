//! An invalid `temp_score!` declaration must fail the export **before** any
//! generated output is produced (#146).
//!
//! This lives in its own test binary because `inventory` registration is
//! process-global: a deliberately invalid declaration would otherwise poison
//! every other export test in the same binary.

use sand_core::TempScoreboard;

// A criterion containing whitespace would silently shift the rest of the
// command into the wrong argument position if it were emitted unchecked.
sand_core::inventory::submit! {
    TempScoreboard {
        name: "broken_temp_score",
        criteria: "dummy extra",
        display_name: None,
    }
}

#[test]
fn invalid_temp_score_fails_the_export() {
    let error = sand_core::try_export_components("failpack")
        .expect_err("an invalid temp score must fail the export");
    let rendered = error.to_string();

    assert!(
        rendered.contains("broken_temp_score"),
        "diagnostic must name the offending declaration; got: {rendered}"
    );
    assert!(
        rendered.contains("temp_score"),
        "diagnostic must identify the temp score phase; got: {rendered}"
    );
    assert!(
        rendered.contains("invalid character"),
        "diagnostic must explain the violated rule; got: {rendered}"
    );
}

#[test]
fn failed_export_produces_no_records_at_all() {
    assert!(
        sand_core::try_export_components("failpack").is_err(),
        "export must fail"
    );
    // The fallible export API returns records only on success, so a failed
    // export can never hand a partial record set to the writer. `sand-cli`
    // performs every filesystem write from that returned set, which is what
    // makes this diagnostic strictly pre-write.
    assert!(
        sand_core::try_export_components_json("failpack").is_err(),
        "the JSON entry point must fail identically"
    );
}
