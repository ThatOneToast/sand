#[path = "common/consumer_fixture.rs"]
mod consumer_fixture;

#[test]
fn shape_preserving_macros_keep_the_author_function_as_the_only_public_api() {
    consumer_fixture::assert_fixture_passes("shape-preserving-consumer", Some("complete-provider"));
    consumer_fixture::assert_fixture_fails_with(
        "shape-preserving-consumer",
        "shape-preserving-consumer/src/lib.rs:16: `sand::generated_schedule`",
    );
}
