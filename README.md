# Sand

**Write Minecraft datapacks in Rust.**

Sand is a toolkit that lets you define Minecraft datapack functions, advancements, recipes, loot tables, and more using type-safe Rust code. It compiles your project into a standard datapack that works with vanilla Minecraft (1.18+, targeting 1.21.x).

## Features

- **Type-safe commands** — generated enums for every `Item`, `Block`, `EntityType`, `Biome`, `Enchantment`, and `SoundEvent` in vanilla Minecraft, plus typed command builders for `execute`, `give`, `kill`, selectors, and more.
- **Proc macros** — `#[function]` turns a Rust function into a `.mcfunction` file. `#[component]` registers advancements, recipes, loot tables, and other datapack elements. `#[component(Tick)]` and `#[component(Load)]` hook into the game loop.
- **Auto-codegen from Minecraft data** — Sand downloads the official server jar and runs Minecraft's data generator to produce Rust types that exactly match the version you're targeting. No stale hand-maintained tables.
- **CLI tooling** — `sand new`, `sand build`, `sand run` (launches a local test server), and `sand clean`.
- **Full datapack component library** — advancements with triggers/criteria/rewards, shaped/shapeless/cooking/stonecutting/smithing recipes, loot tables with pools/conditions/functions, predicates, item modifiers, tags, and custom items (1.21+ item components).
- **Escape hatches** — every component type supports raw JSON for mod compatibility and edge cases not yet covered by typed APIs.

## Quick start

### Prerequisites

- **Rust** 1.85+ (edition 2024)
- **Java** 21+ (for Minecraft's data generator — runs automatically during `cargo build`)

### Install the CLI

```sh
cargo install sand
```

### Create a project

```sh
sand new my_pack
cd my_pack
```

This scaffolds a new Rust project with `sand.toml`, `build.rs`, and a starter `src/lib.rs`.

### Write your datapack

```rust
use sand_core::mcfunction;
use sand_macros::{component, function};

/// A simple function — compiled to `data/my_pack/function/greet.mcfunction`
#[function]
pub fn greet() {
    mcfunction! {
        r#"tellraw @a {"text":"Hello from Sand!","color":"gold"}"#;
    }
}

/// Runs every tick — automatically added to `minecraft:tick` function tag
#[component(Tick)]
pub fn tick_counter() {
    mcfunction! {
        "scoreboard players add @a tick_count 1";
    }
}

/// Runs once on load — automatically added to `minecraft:load` function tag
#[component(Load)]
pub fn on_load() {
    mcfunction! {
        "scoreboard objectives add tick_count dummy";
    }
}
```

### Build

```sh
sand build           # writes to dist/my_pack/
sand build --release # also produces dist/my_pack.zip
```

The output is a standard Minecraft datapack. Copy the folder (or zip) into your world's `datapacks/` directory.

### Test locally

```sh
sand run             # builds, downloads server jar, starts a local server
sand run --offline   # sets online-mode=false for easier testing
```

## Architecture

Sand is a five-crate workspace:

| Crate | Role |
|---|---|
| **`sand`** | CLI binary — `new`, `build`, `run`, `clean`, `version` |
| **`sand-core`** | Core types, traits, command builders, and all datapack component structs |
| **`sand-macros`** | Proc macros — `#[function]`, `#[component]`, `run_fn!` |
| **`sand-build`** | Build pipeline — downloads server jars, runs data generator, produces Rust codegen |
| **`sand-example`** | Integration tests and reference examples |

### How it works

1. **`build.rs`** calls `sand_build::generate()` which downloads the Minecraft server jar, runs its data generator, and writes `registries.rs`, `block_states.rs`, and `commands.rs` to `$OUT_DIR`.
2. **`sand-core`** `include!`s those generated files, giving you typed enums and builders.
3. **`sand-macros`** provides `#[function]` and `#[component]` which expand into `inventory::submit!()` registrations — no manual wiring needed.
4. **`sand build`** compiles your crate, runs the `sand_export` binary, collects all registered components as JSON, and writes the datapack directory structure.

## Macros

### `#[function]`

Turns a Rust function into a `.mcfunction` file. The function name becomes the resource path.

```rust
#[function]
pub fn give_diamonds() {
    mcfunction! {
        "give @a minecraft:diamond 64";
    }
}
// Produces: data/<namespace>/function/give_diamonds.mcfunction
```

### `#[component]`

Registers a datapack component (advancement, recipe, loot table, etc.).

```rust
#[component]
pub fn my_advancement() -> sand_core::Advancement {
    use sand_core::*;
    Advancement::new("my_pack:my_advancement".parse().unwrap())
        .criterion("has_diamond", Criterion::new(
            AdvancementTrigger::inventory_changed(vec![Item::Diamond])
        ))
        .display(AdvancementDisplay::new(
            AdvancementIcon::new(Item::Diamond),
            "Diamond Collector",
            "Obtain a diamond",
        ))
}
```

### `#[component(Tick)]` / `#[component(Load)]`

Shorthand for registering a function that runs every tick or once on load:

```rust
#[component(Tick)]
pub fn my_tick() {
    mcfunction! { "scoreboard players add @a timer 1"; }
}
```

### `#[component(Tag = "ns:name")]`

Hook into any function tag — useful for inter-datapack APIs:

```rust
#[component(Tag = "my_lib:on_player_death")]
pub fn handle_death() {
    mcfunction! { "say A player died!"; }
}
```

### `run_fn!`

Define and call an inline function in one expression:

```rust
#[function]
pub fn main_fn() {
    sand_core::cmd::Execute::new()
        .as_(sand_core::cmd::Selector::all_players())
        .run(run_fn!("my_pack:helper" {
            "say helper function";
        }));
}
```

## Components

Sand provides typed Rust structs for all major datapack element types:

- **`McFunction`** — list of command strings
- **`Advancement`** — triggers, criteria, display, rewards, telemetry
- **`ShapedRecipe` / `ShapelessRecipe`** — crafting recipes
- **`CookingRecipe`** — smelting, blasting, smoking, campfire cooking
- **`StonecuttingRecipe` / `SmithingTransformRecipe` / `SmithingTrimRecipe`**
- **`LootTable`** — pools, entries, conditions, functions
- **`Predicate`** — condition wrappers
- **`ItemModifier`** — loot modification functions
- **`Tag`** — block, item, entity, and function tags
- **`CustomItem`** — 1.21+ item component system (food, equipment, tools, etc.)

## Configuration

Your project is configured via `sand.toml`:

```toml
[pack]
namespace   = "my_pack"
description = "A cool datapack"
mc_version  = "1.21.4"
# pack_format is derived automatically; uncomment to override:
# pack_format = 61
```

## CLI reference

| Command | Description |
|---|---|
| `sand new <name>` | Create a new project (with `--mc_version`, `--description`) |
| `sand init` | Initialize in the current directory |
| `sand build` | Compile to datapack in `dist/` |
| `sand build --release` | Compile + zip for distribution |
| `sand run` | Build + start a local Minecraft server |
| `sand run --offline` | Same, with `online-mode=false` |
| `sand run --ram 8G` | Set JVM heap size |
| `sand clean` | Remove `dist/` |
| `sand clean --cargo` | Remove `dist/` and `target/` |
| `sand version` | Print version |

## Requirements

- **Rust** 1.85+ with edition 2024 support
- **Java** 21+ (used at build time to run Minecraft's data generator)
- **Internet access** on first build (to download the Minecraft server jar; cached afterwards in `~/.sand/cache/`)

## License

MIT
