---
id: custom-item
capabilities:
  - custom-items
  - item-events
minecraft:
  minimum: "1.20.5"
  maximum_verified: "26.2.0"
cargo_features: []
verification:
  compiles: true
  golden_output: false
  vanilla_reload: false
---

# Custom item

## Intent

A custom-data-identified item built on a vanilla base item, given to
players via a typed function, with a use-triggered advancement calling back
into a typed function handler.

## Required crates and features

`sand-core` (`custom_item_ext::CustomItemExt` for `.on_use_fn(...)`),
`sand-macros`.

**Do not use `examples/custom_items.rs`** as a reference — it is stale and
does not compile against current source (two-argument `CustomItem::new`,
raw-string `.custom_name()`, `#[component]` on a type that isn't a
`DatapackComponent`). See `ai/known-limitations.md` (`LIM-DOC-005`). The
code below was verified to compile against current source.

## Code

```rust
use sand_core::custom_item_ext::CustomItemExt;
use sand_core::prelude::*;
use sand_macros::function;

fn inferno_blade() -> CustomItem {
    CustomItem::new("minecraft:diamond_sword")
        .custom_data("inferno_blade")
        .custom_name(TextComponent::literal("Inferno Blade").color(ChatColor::Red))
        .lore_line(TextComponent::literal("A weapon of pure flame").color(ChatColor::DarkRed))
        .enchantment("minecraft:fire_aspect", 2)
        .max_stack_size(1)
        .rarity(ItemRarity::Rare)
}

#[function]
pub fn give_inferno() {
    cmd::give(Selector::all_players(), inferno_blade());
}

pub fn on_use_advancement() -> Advancement {
    inferno_blade().on_use_fn(
        ResourceLocation::new("my_pack", "items/inferno_blade/on_use").unwrap(),
        give_inferno,
    )
}
```

Register `on_use_advancement()`'s return value the same way as any other
`Advancement` component (via `#[component]` on a function returning
`Advancement`, or your export pipeline's advancement list) — `CustomItem`
itself is never registered as a datapack component; it only ever appears
inline as an item-component string passed to commands like `cmd::give`.

## Expected generated resources

- `data/<namespace>/function/give_inferno.mcfunction` — a `give` command
  with the full item-component string, e.g. `give @a
  minecraft:diamond_sword[custom_data={inferno_blade:1b},custom_name={...},
  lore=[...],enchantments={...},max_stack_size=1,rarity="rare"]`.
- `data/my_pack/advancement/items/inferno_blade/on_use.json` — a
  `minecraft:using_item` (or equivalent, per `on_use_fn`'s underlying
  trigger) advancement whose reward function is `give_inferno`.

## Sand limitations

Identity is via `custom_data`, not name/lore — names and lore are cosmetic
and can be duplicated across distinct items; don't use them to distinguish
items in predicates or execute checks. Use
`item.item_check_in(...)`/`item_check_mainhand()` (from `CustomItemExt`) for
typed presence checks that key off `custom_data`, not string-matching the
display name.

## Vanilla limitations

Data components (the whole `minecraft:diamond_sword[...]` model) require
Minecraft 1.20.5+; full ergonomics assume 1.21+. Item identity by
`custom_data` is a datapack convention Sand builds on top of — vanilla has
no first-class "custom item ID" concept.

## Validation steps

1. `cargo build`.
2. `cargo run -p sand -- build`; read the generated `give_inferno.mcfunction` and confirm the item-component string matches what was authored.
3. Not vanilla-reload-verified in this review.

## Common incorrect approaches

- Calling `CustomItem::new(location, base_item)` with two arguments — the
  real signature is `CustomItem::new(base: impl Display)` (one argument);
  set identity separately with `.custom_data("key")`.
- Passing raw JSON strings to `.custom_name(...)`/`.lore_line(...)` — both
  take `TextComponent`, not `&str`/`String`.
- Wrapping a `CustomItem`-returning function in `#[component]` — `CustomItem`
  has no resource location and is not a `DatapackComponent`; only the
  `Advancement` that references it (via `on_use_fn`, etc.) is registered as
  a component.
