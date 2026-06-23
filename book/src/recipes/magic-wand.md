# Magic Wand

Create a custom-data stick, bind an item-use advancement to a typed function, and require mana/cooldown in the handler.

```rust
let wand = CustomItem::new("minecraft:stick").custom_data("arcane_wand");
let use_wand = wand.on_use_fn(ResourceLocation::new("arcane", "wand/use").unwrap(), cast);

#[function]
pub fn cast() {
    when(all![MANA.of("@s").gte(20), CAST_CD.ready("@s")]).then_all([
        MANA.remove("@s", 20), CAST_CD.start(Selector::self_()), cmd::say("spell cast"),
    ]);
}
```

Use a generated command/effect when available. If a projectile needs an unmodeled NBT field, use typed `summon_at_with_nbt` and keep only that NBT raw.
