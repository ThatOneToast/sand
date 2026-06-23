# Global Storage And Dynamic Paths

Minecraft `data storage` holds NBT under a namespaced global document. It is useful for structured config, lists, records, and function hand-off data. It is **not automatically per-player**.

## Minimal example

```rust
use sand_core::prelude::*;

struct Config;
static CONFIG: StorageSchema<Config> = StorageSchema::new("arcane:data", "config");
static MAX_MANA: StorageField<Config, i32> = CONFIG.field("max_mana");

#[component(Load)]
pub fn load_config() { MAX_MANA.set(100); }
```

This lowers to `data modify storage arcane:data config.max_mana set value 100`.

## Static paths and dynamic paths

`StorageSchema`, `StorageField`, and `field_path()` are typed static paths known while compiling. A player UUID is only known while Minecraft runs, so Sand intentionally does not pretend that `storage.for_player("@s")` is static. A manually keyed convention might look conceptually like `players.<uuid>.mana`, but obtaining/formatting the UUID and choosing safe keys is your runtime design.

## Good storage uses

- global config and world settings;
- lists/compound records used by one datapack system;
- static location/config records;
- intermediate NBT payloads for commands.

Use scoreboards for normal per-player progression. See [Player Data](../player-data.md) and [Locations](locations.md).

<div class="sand-limit"><strong>Vanilla limit.</strong> Storage has no native dynamic selector index. Runtime per-player NBT requires an explicit UUID/name-keying strategy and raw/data-command plumbing where static typed paths end.</div>
