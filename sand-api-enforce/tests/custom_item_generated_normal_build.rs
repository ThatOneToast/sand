#[path = "common/consumer_fixture.rs"]
mod consumer_fixture;

#[test]
fn custom_item_generated_public_apis_require_exact_consumer_contracts() {
    consumer_fixture::assert_fixture_passes(
        "custom-item-generated-missing",
        Some("complete-provider"),
    );
    consumer_fixture::assert_fixture_fails_with(
        "custom-item-generated-missing",
        "enforced API scope `sand` has missing contracts: sand::ShardBlade::DAMAGE",
    );
}
