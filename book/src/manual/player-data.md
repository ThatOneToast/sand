# Player Data

`PlayerDataSchema` groups per-player scoreboard fields under one named schema.
`PlayerSchema` remains as a compatibility alias. This is still a manual Phase 1
builder: call `define_all()` from load and `init_player(selector)` from a join
or first-join handler. Automatic lifecycle wiring is future work tracked by
#47/#68.

```rust
let schema = PlayerDataSchema::new("magic")
 .score(&MANA, 100)
 .flag(&HAS_WAND, false)
 .timer(&REGEN_TIMER)
 .cooldown(&CAST_CD)
 .storage(Config::SCHEMA);
schema.define_all(); schema.init_player("@s");
```

Use scoreboards for reliable per-player state: numbers, flags, timers, and
cooldowns. Objective names stay deterministic because they come from the
underlying `ScoreVar`, `Flag`, `Timer`, and `Cooldown` definitions; if a name is
longer than vanilla's 16-character limit, Sand hashes it to a stable objective
name just like the lower-level state APIs.

`.name`, `.scoreboard_field_count`, `.has_storage`, and `.storage_locations`
provide introspection; locations are `StorageDescriptor`s. The schema name is a
label, not an automatic namespace. Storage descriptors do not create dynamic
per-player NBT.

<div class="sand-danger"><strong>Global storage.</strong> Use scoreboards for per-player number/flag/timer data. Use storage for global structures or explicitly keyed player records.</div>
