# Shockwave Shield

Enable `systems-inventory` and `systems-movement`. Give a shield a stable marker and guard a shockwave with both offhand presence and cooldown.

```rust
static SHIELD: CustomItemId = CustomItemId::new("minecraft:shield", "arcane_shockwave");
static CD: Cooldown = Cooldown::new("arcane_shield", Ticks::seconds(5));

#[function]
pub fn shockwave() {
    SHIELD.has_in_offhand().run(when(CD.ready("@s")).then_all([
        PushAway::new().source(Selector::self_())
          .targets(EntityTargets::nearby(6.0).excluding_players()).strength(1.5).lift(0.25)
          .build().join("\n"),
        CD.start(Selector::self_()),
    ]));
}
```

Build the item with `CustomItem::new("minecraft:shield").custom_data("arcane_shockwave")` and supported max-damage/durability components; add typed particles and sound to the response. Exact successful shield-block detection and axe-disable state are not exposed as a dedicated vanilla event, so this reacts to an item-use/held pattern rather than claiming a true block signal.
