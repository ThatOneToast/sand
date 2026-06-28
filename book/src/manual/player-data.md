# Player Data

`PlayerSchema` groups scoreboard initialization and describes static storage schemas.
It is a manual builder today: call `define_all()` from load and
`init_player(selector)` from a join or first-join handler. Automatic lifecycle
wiring is future work tracked by #47/#68.

```rust
let schema = PlayerSchema::new("magic").score(&MANA, 100).flag(&HAS_WAND, false)
 .cooldown(&CAST_CD).storage(Config::SCHEMA);
schema.define_all(); schema.init_player("@s");
```

`.name`, `.scoreboard_field_count`, `.has_storage`, and `.storage_locations` provide introspection; locations are `StorageDescriptor`s. The schema name does not namespace objectives—authors own names. Storage descriptors do not create dynamic per-player NBT.

<div class="sand-danger"><strong>Global storage.</strong> Use scoreboards for per-player number/flag/timer data. Use storage for global structures or explicitly keyed player records.</div>
