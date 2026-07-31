//! Typed datapack components.

use sand_core::prelude::*;
use sand_macros::component;

#[component]
pub fn starter_dialog() -> Dialog {
    Dialog::notice_local("starter")
        .title(Text::new("Starter Kit").gold())
        .body(DialogBody::text(Text::new(
            "Your datapack JSON came from typed Rust.",
        )))
}

#[component]
pub fn starter_item() -> CustomItem {
    CustomItem::new("minecraft:stick")
        .id("example:dash_wand")
        .component(ItemComponent::custom_name(
            Text::new("Dash Wand").aqua().bold(true),
        ))
        .component(ItemComponent::lore(vec![
            Text::new("Right click to dash").gray(),
            Text::new("Consumes mana").dark_gray(),
        ]))
        .component(ItemComponent::EnchantmentGlintOverride(true))
        .component(ItemComponent::max_stack_size(1))
}

/// A complete vanilla-like custom dimension type with typed registry references.
#[component]
pub fn bright_overworld() -> DimensionType {
    DimensionType::overworld_like(
        ResourceLocation::new("example", "bright_overworld").expect("static ID is valid"),
    )
    .ambient_light(0.25)
    .monster_spawn_light_level(MonsterSpawnLightLevel::Constant(7))
}
