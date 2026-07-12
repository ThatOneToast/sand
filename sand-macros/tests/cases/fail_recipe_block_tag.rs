use sand_core::{BlockId, Ingredient, TagId};

fn main() {
    let block_tag: TagId<BlockId> = TagId::minecraft("logs").unwrap();
    let _ = Ingredient::item_tag(block_tag);
}
