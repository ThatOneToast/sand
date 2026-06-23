# Storage And NBT

`StorageSchema<T>` and `StorageField<Schema, T>` produce static `data storage` commands and paths.

```rust
struct Config;
static CONFIG: StorageSchema<Config> = StorageSchema::new("arcane:data", "config");
static MAX: StorageField<Config, i32> = CONFIG.field("max_mana");
MAX.set(100);
assert_eq!(MAX.field_path(), "config.max_mana");
```

Use `SnbtValue` and `SnbtCompound` for values; `NbtPath` composes static paths; `RawSnbt` is the raw boundary. `data storage` is global. Per-player dynamic UUID/name paths must be designed explicitly at runtime—Sand does not fake a compile-time `for_player("@s")` path.
