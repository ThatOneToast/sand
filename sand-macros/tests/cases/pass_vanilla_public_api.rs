//! Compile test proving normal ergonomics for vanilla, custom, and
//! external identifiers through the public `sand` façade (issue #277):
//! generated vanilla IDs, typed custom IDs, and the raw/external escape
//! hatch all work directly through `sand::prelude::*` and `sand::vanilla`.

use sand::prelude::*;
use sand::vanilla;

fn main() {
    // Vanilla item, straight from the generated registry.
    let give_vanilla = cmd::give(Selector::self_(), vanilla::Item::Diamond);
    assert_eq!(give_vanilla, "give @s minecraft:diamond");

    // Vanilla item via the typed custom-ID wrapper (equivalent resource
    // location, constructed the "custom ID" way).
    let diamond_id = ItemId::minecraft("diamond").unwrap();
    let give_typed = cmd::give(Selector::self_(), diamond_id);
    assert_eq!(give_typed, "give @s minecraft:diamond");

    // External/modded item ID, the explicit escape hatch.
    let external: ItemId = "other_mod:machine_core".parse().unwrap();
    let give_external = cmd::give(Selector::self_(), external);
    assert_eq!(give_external, "give @s other_mod:machine_core");

    // Raw string remains accepted (existing normal-path compatibility).
    let give_raw = cmd::give(Selector::self_(), "minecraft:diamond_sword");
    assert_eq!(give_raw, "give @s minecraft:diamond_sword");

    // Vanilla entity type in a selector filter.
    let marker_selector = EntityTargets::all().entity_type(vanilla::EntityType::Marker);
    assert_eq!(marker_selector.to_string(), "@e[type=minecraft:marker]");

    // Vanilla entity type in a typed query, narrowed to a single result.
    let marker_query = EntityQuery::entities()
        .entity_type(vanilla::EntityType::Marker)
        .nearest();
    assert!(
        marker_query
            .selector()
            .to_string()
            .contains("type=minecraft:marker")
    );

    // Custom/typed entity type ID also works on the same normal path.
    let custom_type = EntityTypeId::custom(ResourceLocation::new("mymod", "boss").unwrap());
    let custom_selector = EntityTargets::all().entity_type(custom_type);
    assert_eq!(custom_selector.to_string(), "@e[type=mymod:boss]");

    // Vanilla block, converted into the typed BlockId wrapper.
    let wool: BlockId = vanilla::Block::WhiteWool.into();
    assert_eq!(wool.to_string(), "minecraft:white_wool");
}
