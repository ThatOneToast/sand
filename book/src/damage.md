# Damage Tracking

Direct `/damage` is single-target in vanilla, so Sand models that in Rust:

```rust
cmd::damage(SingleEntity::self_(), 4.0).damageType("minecraft:generic");
```

For many targets, use the high-level builder:

```rust
Damage::new()
    .to(EntityTargets::nearby(5.0).excluding_players().excluding_self())
    .amount(DamageAmount::fixed(4.0))
    .damage_type(DamageKind::Generic)
    .run();
```

Sand lowers that to:

```mcfunction
execute as @e[distance=0.1..5,type=!minecraft:player] run damage @s 4 minecraft:generic
```

Damage events use `DamageEvent<T>`:

```rust
#[event]
pub fn on_damaged(event: DamageEvent<MyDamageEvent>) {
    event
        .reflect_damage()
        .to(EntityTargets::nearby(5.0).excluding_players().excluding_self())
        .amount(DamageAmount::fixed(4.0))
        .damage_type(DamageKind::Generic)
        .run();
}
```

Exact same-as-event damage is not available from vanilla advancement reward
functions. Use fixed damage unless a real tracker is added.

Enable `systems-damage` for `DamageTracker`. It creates cumulative damage stat, previous stat, current delta, last non-zero delta, and ticks-since-hurt objectives. Call `DamageTracker::define()` in load and `DamageTracker::tick_players()` every tick before reading a condition.

```rust
let hit = DamageTracker::damaged_this_tick("@s");
let large_hit = DamageTracker::current_damage_at_least("@s", DamageThreshold::hearts(1.0));
let recent = DamageTracker::hurt_within("@s", Ticks::seconds(2));
```

`not_damaged_this_tick`, `last_damage_at_least`, `current_damage_at_least`, and `hurt_within` are the normal query helpers. `DamageThreshold::hearts(1.0)` becomes 10 raw stat units; `raw_stat` is for advanced scoreboard work. Threshold queries require a positive finite heart value that rounds to at least 1 raw stat unit, or a positive raw stat value. Minecraft's damage-taken statistic is **not** `/damage` HP: one stat unit is 0.1 heart, so fractional heart thresholds are rounded to the nearest 0.1 heart. In contrast, `DamageAmount::hearts(1.0)` emits two `/damage` HP.

<div class="sand-warning"><strong>Approximation.</strong> Multiple hits in one tick are summed; invulnerability frames can report no delta; source/type/attacker are not available from this statistic. Combine an advancement damage event for source-aware reactions. <code>recently_damaged</code>, <code>damaged_at_least</code>, and <code>delta_objective</code> are deprecated compatibility APIs.</div>
