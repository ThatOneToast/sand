use sand::component::McFunction;
use sand::prelude::*;
use sand::registration::{DatapackRegistration, IntoDatapack};
use sand::systems::raycast::BlockRaycast;
use std::cell::Cell;

thread_local! { static CASE: Cell<u8> = const { Cell::new(0) }; }
#[function]
fn callback() {
    cmd::say("callback");
}
#[datapack_component]
fn test_definition() -> DatapackRegistration {
    let id = ResourceLocation::new("__sand_local", "test").unwrap();
    let mut ray = BlockRaycast::new(id)
        .scan_for(BlockPredicate::new().blocks(vec![vanilla::Block::Stone.into()]))
        .on_hit(callback)
        .on_miss(callback);
    let case = CASE.get();
    match case {
        1 => ray = ray.step(f64::NAN),
        2 => ray = ray.max_distance(0.0),
        3 => ray = ray.step(0.00001),
        4 => ray = ray.first_collision(),
        5 => ray = ray.on_hit(FunctionId::local("absent")),
        8 => ray = ray.on_miss(FunctionId::local("absent_miss")),
        9 => {
            ray = BlockRaycast::new(ResourceLocation::new("validation", "no_hit").unwrap())
                .scan_for(BlockPredicate::new())
        }
        _ => (),
    }
    let mut registration = ray.into_datapack();
    if case == 6 {
        registration = registration.component(
            McFunction::new(
                ResourceLocation::new("__sand_local", "__sand_system/raycast/test/start").unwrap(),
            )
            .command(cmd::say("collision")),
        );
    }
    if case == 7 {
        // Same-criterion objective declarations must not alias generated storage.
        let objective =
            sand::command::ObjectiveName::logical("sand raycast validation:test remaining");
        registration = registration.component(
            McFunction::new(ResourceLocation::new("validation", "foreign_owner").unwrap())
                .command(Objective::dynamic(objective.to_string()).create("dummy")),
        );
    }
    registration
}
#[test]
fn invalid_definitions_and_resource_or_counter_collisions_fail_before_output() {
    for (case, expected) in [
        (1, "finite and positive"),
        (2, "finite and positive"),
        (3, "between 1 and 256"),
        (4, "pass_through"),
        (5, "callback `validation:absent`"),
        (6, "collides"),
        (7, "collides with generated numeric storage"),
        (8, "callback `validation:absent_miss`"),
        (9, "an on_hit callback is required"),
    ] {
        CASE.set(case);
        let error = sand::advanced::try_export_pack_json("validation", "26.2")
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected), "case {case}: {error}");
        CASE.set(0);
        sand::advanced::try_export_pack_json("validation", "26.2").unwrap();
    }
}
