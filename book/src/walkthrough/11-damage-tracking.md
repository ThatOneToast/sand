# 11. Damage Tracking

Enable `systems-damage`, define objectives once, and tick them before checking damage.

```rust
DamageTracker::define();
DamageTracker::tick_players();
DamageTracker::current_damage_at_least("@s", DamageThreshold::hearts(1.0));
```

Damage stats are approximate and use different units from `/damage`. See [Damage Tracking](../manual/damage-tracking.md).
