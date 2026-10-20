use sand::command::Anchor;
use sand::prelude::*;
use sand::systems::raycast::BlockRaycast;

#[datapack_component]
fn mining_ray() -> BlockRaycast {
    BlockRaycast::new(ResourceLocation::new("demo", "mining").unwrap())
        .origin(Anchor::Eyes)
        .scan_for(BlockPredicate::new().tag(TagId::minecraft("logs").unwrap()))
        .max_distance(5.0)
        .step(0.25)
        .on_hit(found_log)
}
#[function]
fn found_log() {
    WorldContext::current().block(BlockPos::here()).replace(vanilla::Block::Air);
}
#[function]
fn use_tool() {
    mining_ray().cast();
}
fn main() {}
