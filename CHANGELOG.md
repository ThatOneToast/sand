# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **sand-components**: `item::stack::ItemStack`, `item::matcher::ItemMatcher`, and
  `item::definition::CustomItemDefinition` — a shared foundation for item identity
  (phase 1 of #229). `ItemStack` is a typed, component-bearing concrete stack
  (typed `ItemId`, validated `1..=99` count, `CustomItem`'s existing component
  model, and an explicit `RawComponent` escape hatch). `ItemMatcher` is a
  separate detection type distinguishing exact (`custom_data_exact`,
  `raw_components_exact`) from partial (`custom_data_partial`,
  `raw_predicates_partial`) matching, plus typed `enchantment`/`damage_range`
  helpers, and converts to `predicates::ItemPredicate` through a single
  consumer- and `VersionCaps`-aware seam (`ItemMatcher::try_render_for`,
  `ItemMatcherConsumer`) that rejects rather than weakens unsupported
  component constraints — generalizing the `AdvancementItemConsumer` seam PR
  #237 introduced as this work's documented precursor.
  `CustomItemDefinition` ties one base item ID and `custom_data` marker to
  consistent `.stack()`/`.matcher()`/`.try_item_predicate()`/`.try_recipe_result()`
  output, so a custom item's identity never has to be repeated per API.
  New `TryIntoIngredient`/`TryIntoRecipeResult` traits integrate `ItemMatcher`/
  `ItemStack` with the existing `Ingredient`/`RecipeResult` recipe types
  without changing their public API, and `AdvancementTrigger::render_for`'s
  legacy-item-filter rejection now delegates to the same shared diagnostic
  function instead of duplicating it. `ItemLocation` (typed entity/block
  accessors) and `ItemSnapshot` (event-time capture) remain follow-up work —
  see `docs/typedness-audit.md`. Refs #229.
- **sand-core**: `entity` module — cardinality-aware `EntityQuery`/`PlayerQuery`
  builders on top of the existing typed selector arity wrappers, an
  execution-scoped `EntityContext<K>` (`@s` handle) passed into `.each(...)`,
  typed traversal of vanilla `execute on <relation>` relationships (owner,
  leasher, target, vehicle, controller, attacker, origin, passengers) gated
  against `VersionProfile`, and `EntityScope::bind` for preserving a working
  reference to an entity across relationship traversal via a collision-safe
  temporary tag. Framework infrastructure for #228–#230; resolves #227.

### Fixed

- **sand-components**: `AdvancementTrigger::render_for` now maps the target
  profile to an explicit `AdvancementSchemaFamily` in one place instead of an
  inline capability check, and rejects an item filter on
  `AdvancementSchemaFamily::Legacy` (pre-1.20.5 profiles) with an actionable
  diagnostic instead of emitting a modern `components`/`predicates` shape
  those profiles don't recognize. Item-predicate conversion is now tagged
  with an `AdvancementItemConsumer` for richer diagnostics. `sand-vanilla-audit`
  gained `placed_block`/`item_used_on_block` fixtures exercising the fixed
  `conditions.location`/`minecraft:location_check`/`minecraft:match_tool`
  shape, verified against a real Minecraft 26.2 server (load + reload).
  `trigger_coverage::TriggerCoverage` now tracks `vanilla_load_tested_profiles`
  and `semantic_runtime_tested_profiles` separately from `golden_json_tested`.
  Refs #231, #232 — real client-driven semantic (fires-only-for-matches)
  verification remains unautomated; see `docs/vanilla-reload-validation.md`.

### Documentation

- Documented the typed gameplay state API surface (`GameState<S>`,
  `TypedGameState`, lifecycle registry, transitions, enter/exit hooks,
  per-state tick, tick-cost guidance, transition backend table). Added a
  runnable, compile-tested `examples/gameplay_state.rs` example pack
  mirrored under `sand-example/src/gameplay_state_example.rs`. Added a
  concise agent-facing guide at `docs/agents/state-guide.md`. Resolves
  issue #61.

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
