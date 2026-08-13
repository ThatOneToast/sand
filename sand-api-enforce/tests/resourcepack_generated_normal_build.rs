#[path = "common/consumer_fixture.rs"]
mod consumer_fixture;

#[test]
fn resourcepack_hud_handles_require_exact_consumer_contracts() {
    consumer_fixture::assert_fixture_passes(
        "resourcepack-generated-missing",
        Some("complete-provider"),
    );
    consumer_fixture::assert_fixture_fails_with(
        "resourcepack-generated-missing",
        "enforced API scope `sand` has missing contracts: sand::STATUS_ICON",
    );
}
