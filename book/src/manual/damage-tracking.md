# Damage Tracking

Enable `systems-damage`. Define once and tick every player before querying conditions.

```rust
DamageTracker::define(); DamageTracker::tick_players();
DamageTracker::damaged_this_tick("@s");
DamageTracker::last_damage_at_least("@s", DamageThreshold::hearts(1.0));
DamageTracker::hurt_within("@s", Ticks::seconds(2));
```

Tracker objectives hold total stat, previous total, delta, last non-zero delta, and hurt age. One heart is 10 raw damage-stat units; threshold queries require a positive finite heart value that rounds to at least 1 raw stat unit, or a positive raw stat value. `/damage` uses HP, so `DamageAmount::hearts(1.0)` is two HP. Multiple hits may be combined and invulnerability frames may be absent. Deprecated `recently_damaged`, `damaged_at_least`, and `delta_objective` remain only for compatibility.
