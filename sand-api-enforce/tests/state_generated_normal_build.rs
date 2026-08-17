#[path = "common/consumer_fixture.rs"]
mod consumer_fixture;

#[test]
fn state_derive_generated_public_apis_require_exact_consumer_contracts() {
    consumer_fixture::assert_fixture_passes("state-generated-missing", Some("complete-provider"));
    consumer_fixture::assert_fixture_fails_with(
        "state-generated-missing",
        "enforced API scope `sand` has missing contracts: sand::PlayerState::mana",
    );
}
