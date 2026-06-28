// Canonical recipe: typed CustomItem with food/consumable properties used
// inside a #[function]. Demonstrates public item-building API and cmd::give.
use sand_core::prelude::*;
use sand_macros::function;

fn enchanted_carrot() -> CustomItem {
    CustomItem::new("minecraft:carrot")
        .custom_name(Text::new("Enchanted Carrot").gold())
        .rarity(ItemRarity::Rare)
        .food(FoodProperties::new(6, 8.0).can_always_eat(true))
        .consumable(
            ConsumableProperties::new(1.6)
                .animation(ConsumableAnimation::Eat)
                .sound("minecraft:entity.player.burp"),
        )
        .max_stack_size(16)
}

#[function]
pub fn give_starter_kit() {
    cmd::give(Selector::self_(), enchanted_carrot());
    cmd::tellraw(
        Selector::self_(),
        Text::new("Starter kit granted!").green(),
    );
}

fn main() {
    let item = enchanted_carrot();
    let snbt = item.to_string();
    assert!(
        snbt.contains("custom_name"),
        "custom_name missing: {snbt}"
    );
    assert!(snbt.contains("food"), "food component missing: {snbt}");
    assert!(
        snbt.contains("consumable"),
        "consumable component missing: {snbt}"
    );

    let kit_cmds = give_starter_kit();
    assert!(
        kit_cmds
            .iter()
            .any(|c| c.starts_with("give") && c.contains("carrot")),
        "expected give command containing carrot; got: {kit_cmds:?}"
    );
    assert!(
        kit_cmds.iter().any(|c| c.contains("tellraw")),
        "expected tellraw; got: {kit_cmds:?}"
    );
}
