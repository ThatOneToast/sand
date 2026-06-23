# Damage Reactive Ability

Enable `systems-damage` and tick it before checking thresholds. Pair it with a cooldown to avoid a response every tick.

```rust
#[component(Load)] pub fn load_damage() { DamageTracker::define(); CD.define(); }
#[component(Tick)] pub fn tick_damage() { DamageTracker::tick_players(); CD.tick(Selector::all_players()); }
#[function] pub fn react() {
 when(all![DamageTracker::current_damage_at_least("@s", DamageThreshold::hearts(1.0)), CD.ready("@s")])
 .then_all([cmd::effect_give(Selector::self_(), EffectId::Regeneration).seconds(3), CD.start(Selector::self_())]);
}
```

Use `hurt_within` for an aftershock window. The tracker sees cumulative stat deltas: it can combine hits and cannot identify attacker/type; use a damage advancement criterion when source filtering matters.
