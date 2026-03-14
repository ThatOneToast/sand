# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-14

Initial public release.

### Added

- **sand** CLI with `new`, `init`, `build`, `run`, `clean`, and `version` commands.
- **sand-core** library with core types and traits:
  - `ResourceLocation`, `PackNamespace`, `McVersion` validated types.
  - `DatapackComponent` and `IntoDatapack` traits.
  - `mcfunction!` macro for building command lists.
  - Full command builder system: `Execute`, `Selector`, `SetBlock`, `Fill`, `CloneBlocks`, `Bossbar`, `Title`, `Actionbar`, `Sound`, `Cooldown`, `Storage`, and more.
  - All major datapack component types: `Advancement`, `ShapedRecipe`, `ShapelessRecipe`, `CookingRecipe`, `StonecuttingRecipe`, `SmithingTransformRecipe`, `SmithingTrimRecipe`, `LootTable`, `Predicate`, `ItemModifier`, `Tag`, `McFunction`.
  - 1.21+ `CustomItem` with full item component support (food, equipment, tools, durability, enchantments, etc.).
  - Escape hatches (`Custom` variants) on all component types for raw JSON and mod compatibility.
- **sand-macros** procedural macros:
  - `#[function]` — register a Rust function as a `.mcfunction` file.
  - `#[component]` — register a function that returns a `DatapackComponent`.
  - `#[component(Tick)]` / `#[component(Load)]` — auto-register functions in `minecraft:tick` or `minecraft:load` tags.
  - `#[component(Tag = "ns:name")]` — register in a custom function tag.
  - `run_fn!` — define and call an inline function in one expression.
- **sand-build** data pipeline:
  - Automatic Minecraft server jar download with SHA1 verification and caching.
  - Data generator integration (requires Java 21+).
  - Rust codegen for registry enums (`Item`, `Block`, `EntityType`, `Biome`, `Enchantment`, `SoundEvent`).
  - Rust codegen for block state property types.
  - Rust codegen for typed command builders from `commands.json`.
  - Mojang version manifest fetching with offline fallback.
- **sand run** command: builds the datapack, downloads the server jar, and launches a local Minecraft server for testing.
- Project scaffolding with `sand new` and `sand init`.
- Zip packaging with `sand build --release`.

### Supported Minecraft versions

- Targeting **1.21.x** (pack_format 48-61).
- Registry codegen works with any version that supports `--reports`.
- Pack format mapping covers 1.18 through 1.21.11+.

[0.1.0]: https://github.com/ThatOneToast/sand/releases/tag/v0.1.0
