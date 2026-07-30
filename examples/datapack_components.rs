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

#[component]
pub fn quartz_trim_material() -> TrimMaterial {
    TrimMaterial::new(ResourceLocation::new("example", "quartz").unwrap())
        .asset_name(TrimAssetName::new("quartz").unwrap())
        .ingredient(ItemId::minecraft("quartz").unwrap())
        .item_model_index(0.1)
        .description(TextComponent::translate(
            "trim_material.example.quartz",
        ))
}

#[component]
pub fn bolt_trim_pattern() -> TrimPattern {
    TrimPattern::new(ResourceLocation::new("example", "bolt").unwrap())
        .asset_id(ResourceLocation::new("example", "bolt").unwrap())
        .template_item(ItemId::minecraft("bolt_armor_trim_smithing_template").unwrap())
        .description(TextComponent::translate("trim_pattern.example.bolt"))
}
