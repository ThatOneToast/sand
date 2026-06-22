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
        .custom_name(Text::new("Dash Wand").aqua().bold(true))
        .lore_line(Text::new("Right click to dash").gray())
        .lore_line(Text::new("Consumes mana").dark_gray())
        .custom_data("dash_wand")
        .enchantment_glint_override(true)
        .max_stack_size(1)
}
