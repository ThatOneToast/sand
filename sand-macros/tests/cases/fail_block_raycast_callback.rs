use sand::prelude::*;
use sand::systems::raycast::BlockRaycast;
fn main() {
    BlockRaycast::new(ResourceLocation::new("demo", "ray").unwrap())
        .on_hit(ItemId::minecraft("stone").unwrap());
}
