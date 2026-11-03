//! No book_project reference: this executable has no gameplay registrations.
use sand::prelude::*;
use sand::systems::raycast::BlockRaycast;

#[test]
fn discarded_workflow_definitions_and_actions_do_not_register_runtime_work() {
    let before = sand::advanced::try_export_pack_json("unused", "26.2").unwrap();
    let ray = BlockRaycast::new(ResourceLocation::new("unused", "unregistered_anchor").unwrap())
        .first_collision()
        .pass_through(BlockPredicate::new().blocks(vec![BlockId::minecraft("air").unwrap()]))
        .max_distance(8.0)
        .step(0.25)
        .on_hit(FunctionId::custom(
            ResourceLocation::new("unused", "accepted").unwrap(),
        ));
    drop(ray.cast());
    drop(Target::players().each(|player| {
        player
            .living()
            .give_effect(EffectId::Speed)
            .duration(Ticks::seconds(1))
    }));
    let after = sand::advanced::try_export_pack_json("unused", "26.2").unwrap();
    assert_eq!(before, after);
    assert!(!after.contains("unregistered_anchor"));
    assert!(!after.contains("raycast"));
    assert!(!after.contains("Traversal"));
}
