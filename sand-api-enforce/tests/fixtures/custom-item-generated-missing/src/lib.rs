use sand::custom_item;
use sand::prelude::{CustomItem, ItemSlot};

#[custom_item(name = "ShardBlade", data = [DAMAGE: i32 = 7])]
pub fn shard_blade() -> CustomItem {
    CustomItem::new("minecraft:diamond_sword").custom_data("shard_blade")
}

const _: &'static str = ShardBlade::BASE;
const _: fn(ItemSlot, String) -> String = ShardBlade::if_wearing;

#[doc(hidden)]
pub mod __private {
    include!(concat!(env!("OUT_DIR"), "/api_enforcement.rs"));
}
