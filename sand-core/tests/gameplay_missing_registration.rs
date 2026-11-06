//! A typed action must not export a pack with a missing private runtime helper.
use sand::prelude::*;
use sand::systems::{Lifetime, Scene};

#[function]
fn unregistered_scene() {
    Scene::new(Lifetime::<PlayerKind>::new(
        "missing_scene:celebration".parse().unwrap(),
        Ticks::seconds(2),
    ))
    .start();
}

#[test]
fn scene_start_without_registration_reports_the_missing_definition() {
    let error = sand::advanced::try_export_pack_json("missing_scene", "26.3")
        .expect_err("unregistered Scene must fail export");
    let message = error.to_string();
    assert!(
        message.contains("unregistered gameplay function"),
        "{message}"
    );
    assert!(
        message.contains("missing_scene:__sand_system/lifetime/celebration/start"),
        "{message}"
    );
    assert!(message.contains("#[datapack_component]"), "{message}");
}
