# 12. Storage And NBT

## What you will build

Add static global Arcane configuration using typed NBT storage and decide when scores, storage records, or marker entities should represent data and locations.

## Concepts introduced

Minecraft `data storage`, NBT, `StorageSchema`, `StorageField`, `SandStorage`, static NBT paths, global records, and location representations.

## File changes

Add `sand-macros` to imports and place this near the top of `arcane/src/lib.rs`:

```rust
use sand_core::prelude::*;
use sand_macros::{component, function, SandStorage};

#[derive(SandStorage)]
#[sand(storage = "arcane:data", root = "config")]
pub struct ArcaneConfig {
    pub max_mana: i32,
    #[sand(path = "spawn")]
    pub spawn_record: String,
}

#[component(Load)]
pub fn load_config() {
    ArcaneConfig::max_mana().set(100);
    ArcaneConfig::spawn_record().set("overworld_spawn");
}
```

`ArcaneConfig::max_mana().field_path()` is the static path `config.max_mana`; the set lowers to `data modify storage arcane:data config.max_mana set value 100`.

## How it works

NBT is Minecraft's typed tree data format: compounds contain named fields, lists contain values, and paths select nodes. `data storage arcane:data` is one global document, not one document per player. The derive creates `SCHEMA` and static field accessors; Rust types document intended values but do not turn runtime NBT into a Rust struct.

Use scoreboards for per-player numbers/flags/cooldowns. Use storage for global config, global counters, lists, and records. A location can be three scores for math, a marker entity for `execute at`, or a storage `{x,y,z}` record for configuration. Runtime per-player records need an explicit UUID/name-keyed design; Sand cannot type a dynamic path that only exists while Minecraft runs.

## What Sand generates

The schema itself emits no JSON. Each setter emits a vanilla `data modify storage` command. Storage needs no `define` command.

## Try it in Minecraft

Build and `/reload`, then run `/data get storage arcane:data config`. You should see the two fields. Change a value via a small function and rerun the command.

## Common mistakes

- Treating `arcane:data` as automatically per-player.
- Storing a frequently updated mana value in NBT instead of a scoreboard.
- Assuming a storage location record can immediately be used as a teleport coordinate without extracting it.

## Deeper reading

[Global Storage](../manual/data-model/storage.md), [Locations](../manual/data-model/locations.md), [Storage](../manual/storage.md), and [Player Data](../manual/player-data.md).
