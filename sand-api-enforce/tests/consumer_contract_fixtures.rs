#[path = "common/consumer_fixture.rs"]
mod consumer_fixture;

fn assert_failure(fixture: &str, expected: &[&str]) {
    let output = consumer_fixture::check_fixture(fixture, None);
    assert!(!output.status.success(), "{fixture} unexpectedly compiled");
    let diagnostic = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    for text in expected {
        assert!(
            diagnostic.contains(text),
            "{fixture} did not report {text:?}:\n{diagnostic}"
        );
    }
}

#[test]
fn consumer_contract_boundaries_are_enforced_with_one_shared_build_cache() {
    for (fixture, diagnostic) in [
        (
            "custom-item-generated-missing",
            "enforced API scope `sand` has missing contracts: `sand::ShardBlade::DAMAGE`",
        ),
        (
            "derive-generated-missing",
            "enforced API scope `sand` has missing contracts: `sand::PlayerMagic::mana`",
        ),
        (
            "state-generated-missing",
            "enforced API scope `sand` has missing contracts: `sand::PlayerState::mana`",
        ),
        (
            "shape-preserving-consumer",
            "shape-preserving-consumer/src/lib.rs:16: `sand::generated_schedule`",
        ),
    ] {
        consumer_fixture::assert_fixture_passes(fixture, Some("complete-provider"));
        consumer_fixture::assert_fixture_fails_with(fixture, diagnostic);
    }

    assert_failure(
        "missing-contract",
        &["public API `sand::fixture::forgotten_api` (function) is missing #[api]"],
    );
    assert_failure(
        "reachable-enforced-missing",
        &[
            "sand::uncontracted_root_hook",
            "sand::predicate::Builder::uncontracted_method",
            "sand::version::VersionProfile::uncontracted_capability",
        ],
    );
    assert_failure(
        "reachable-include-unbound",
        &[
            "reachable module `sand::generated` contains include!",
            "neither a literal source include nor bound to a named generated API provider",
        ],
    );
}
