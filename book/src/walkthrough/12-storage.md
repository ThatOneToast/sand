# 12. Storage And NBT

Use static paths for global structured data, not an implicit per-player database.

```rust
#[derive(SandStorage)]
#[sand(storage = "arcane:data", root = "config")]
struct Config { max_mana: i32 }
Config::max_mana().set(100);
```

See [Storage](../manual/storage.md) and [Player Data](../manual/player-data.md).
