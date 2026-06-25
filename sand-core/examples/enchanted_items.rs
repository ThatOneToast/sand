//! Parser-oriented custom item example.
//!
//! The generated command uses current item components: enchantment components
//! are direct maps, not legacy `{levels:{...}}` compounds. Parsing an
//! enchantment on an item does not guarantee that Minecraft gives it useful
//! gameplay behavior (for example, Infinity on a crossbow).

use sand_commands::give;
use sand_core::prelude::*;

fn charged_crossbow() -> CustomItem {
    CustomItem::new("minecraft:crossbow")
        .typed_enchantment(EnchantmentId::minecraft("quick_charge").unwrap(), 10)
        .typed_enchantment(EnchantmentId::minecraft("infinity").unwrap(), 1)
}

fn inferno_sword() -> CustomItem {
    CustomItem::new("minecraft:diamond_sword")
        .id("arcane:inferno_blade")
        .custom_name(Text::new("Inferno Blade").red())
        .lore_line(Text::new("A parser-safe component example").gray())
        .max_damage(3000)
        .damage(0)
        .typed_enchantment(EnchantmentId::minecraft("sharpness").unwrap(), 5)
        .enchantment_glint_override(true)
}

fn sharp_book() -> CustomItem {
    CustomItem::new("minecraft:enchanted_book")
        .typed_stored_enchantment(EnchantmentId::minecraft("sharpness").unwrap(), 5)
}

fn give_crossbow() -> String {
    give(Selector::self_(), charged_crossbow())
}

fn give_sword() -> String {
    give(Selector::self_(), inferno_sword())
}

fn give_book() -> String {
    give(Selector::self_(), sharp_book())
}

fn main() {
    println!("{}", give_crossbow());
    println!("{}", give_sword());
    println!("{}", give_book());
}
