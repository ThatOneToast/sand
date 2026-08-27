#[path = "common/consumer_fixture.rs"]
mod consumer_fixture;

#[test]
fn real_derive_expansion_is_provider_connected_and_missing_member_contract_fails_check() {
    consumer_fixture::assert_fixture_passes("derive-generated-missing", Some("complete-provider"));
    consumer_fixture::assert_fixture_fails_with(
        "derive-generated-missing",
        "enforced API scope `sand` has missing contracts: `sand::PlayerMagic::mana`",
    );
}
