# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed — structure copy sources can no longer escape the project root (#158)

- **Structure-template copy sources are now canonically contained (#158).**
  `validate_structure_source_file` joined the declared source onto the project
  root and accepted it if `std::fs::metadata` reported a file. Because
  `metadata` follows symlinks, a project-relative source such as
  `src/structures/leak.nbt -> ../../outside/leak.nbt` — or any source reached
  through a symlinked intermediate directory — passed every lexical check
  (absolute, `..`, prefix, null byte, `.nbt`) and was copied into
  `data/<namespace>/structure/…`. That left the datapack copy boundary weaker
  than the resource-pack copy boundary hardened in #154/#156. Both paths now
  share one helper, `validate_copy_source_within_project`, which canonicalizes
  the project root and the source and requires the canonical source to stay
  inside the canonical root. **Symlink policy:** a symlink is accepted only
  when its resolved target is still inside the project root, matching the
  existing resource-pack policy. Containment is component-wise, so
  `project-other` is not treated as inside `project`. Every structure record is
  still validated before the datapack output directory is created, and the
  missing, not-a-file, and unreadable diagnostics are unchanged. A symlink
  whose target is missing now says so instead of reporting a file the author
  can see in their tree as "not found". Two limits are documented rather than
  claimed away: hard links are invisible to canonicalization, and the check is
  a preflight, not a race-free copy (the copy re-joins the path afterwards, as
  the resource-pack path already did).

### Fixed — collision-safe schedule objectives, typed state holders, typed target filters (#124, #146, #200)

- **Generated schedule objectives can no longer silently collide (#124).**
  `#[schedule]` derived `__ss_<key>_t` / `__ss_<key>_p` from a bare 32-bit
  FNV-1a hash of the schedule path, so two distinct paths that hashed equal
  would share countdown/phase state while still producing a valid-looking
  datapack. Keys are now allocated for the whole schedule set up front by
  `allocate_schedule_objective_keys`, which walks paths in sorted order via a
  `BTreeSet` and claims the first unclaimed deterministically salted rehash.
  Sorted iteration makes the mapping depend only on the *set* of schedule
  paths — never on `inventory` registration order, thread scheduling, or
  hash-map iteration order — and probe attempt 0 is the original path hash, so
  non-colliding packs keep byte-identical output. Salted rehashing is used
  instead of the `_<n>` suffix that `armor::allocate_armor_tag_keys` uses
  because `__ss_` (5) + 8 hex + `_t` (2) already spends 15 of Minecraft's
  16-character objective budget. If all 64 probes collide, the export fails
  with a diagnostic naming both schedule paths, the generated key, and the
  affected objective names.
- **`Flag`, `Timer`, and `Cooldown` gained validated typed-holder paths
  (#146).** All three interpolated `selector: impl Display` straight into
  `scoreboard players …` / `execute if score …` text with no validation,
  independent of the `ScoreVar` typed path delivered by #285/#286. Every
  holder-bearing operation now has a `try_*` counterpart taking
  `impl Into<sand_commands::ScoreHolder>`. Operations where vanilla requires
  exactly one holder — every condition and guard — enforce
  `ScoreHolder::validate_single`, rejecting wildcard and multi-entity holders;
  plain mutations validate the holder but still permit legal multi-target
  forms. Each `try_*` delegates to its raw counterpart after validation, so
  typed and compatibility calls render byte-identically. The infallible
  methods remain as documented raw escape hatches (no `#[deprecated]`, matching
  the existing repository idiom).
- **`temp_score!` metadata is validated before export (#146).** Objective
  names now route through the canonical `ObjectiveName` rules, so a name over
  the 16-character limit is deterministically hashed the same way the rest of
  the state family hashes theirs, while empty, whitespace-bearing, and
  control-character names are rejected outright rather than silently hashed.
  Criterion tokens must be non-empty and use only `[A-Za-z0-9_.:-]`.
  **Generated-output change:** display names were previously interpolated as
  bare text into `scoreboard objectives add <name> <criterion> <display>`, but
  vanilla requires a JSON text component in that position — so any pack using
  the three-argument `temp_score!` form was emitting a malformed command.
  Display names are now rendered through the canonical `TextComponent`, e.g.
  `{"text":"Total Kills"}`, with quotes escaped and control characters
  rejected. Covered by exact regression tests.
- **`EntityTarget<A>` / `PlayerTarget<A>` expose the full typed filter
  vocabulary (#200).** The typed filters added to `Selector` by #286
  (`predicate_id`, `scores_typed`, `distance_typed`, `level_typed`, typed
  tags/teams, typed gamemode) were unreachable from the target wrappers
  without `.into_selector()`. They are now forwarded generically over the
  cardinality marker `A`, so a single target stays single and a many target
  stays many through every filter. Forwarding is capability-correct:
  `gamemode`/`level` are exposed only on `PlayerTarget` (player-only vanilla
  arguments) and `entity_type`/`not_type` only on `EntityTarget` (a player
  target is already `type=player`). Every wrapper method delegates to the
  underlying `Selector`, so validation and diagnostics are identical and
  rendering is byte-identical; compile-fail tests pin the incompatible
  combinations. Raw escape hatches (`scores_raw`, `nbt_raw`, `predicate_raw`)
  are forwarded and remain explicitly documented.

### Fixed — visual-component builder validation and registry coverage accuracy (#141, #193)

- **Asset-backed visual builders now validate before export.**
  `BannerPattern`, `PaintingVariant`, `TrimMaterial`, `TrimPattern`, and
  `WolfVariant` implement `DatapackComponent::validate`, so invalid
  builder states (empty/malformed `asset_id`s, texture paths, and item
  IDs; painting `width`/`height` outside `1..=16`, a real
  vanilla-documented bound; non-finite trim `item_model_index`; empty or
  malformed `WolfVariant` `biomes` shapes) are rejected with a structured
  diagnostic before any JSON is written, instead of silently serializing
  incomplete or malformed public state. No public API became panicking;
  invalid state is now caught earlier, at export time, via the existing
  `try_content`/`validate` hook. `TrimMaterial::item_model_index` does
  **not** enforce a `0.0..=1.0` range: it is a legacy pre-1.21.4 field
  with no vanilla-documented bound, and real third-party tooling uses
  out-of-range values (e.g. `-1.0`) intentionally — only finiteness is
  required.
- **`RegistryCoverage` status now matches actual typedness.**
  `minecraft:advancement` is downgraded from `FullyImplemented` to
  `PartiallyImplemented` (still exposes normal-path raw strings /
  `serde_json::Value` fields for display text, icon, rewards, and parent;
  tracked by #183). `minecraft:recipe` was considered for the same
  downgrade but its typed item/tag ID work (#178) turned out to already
  be closed and present on this branch's base, so it remains
  `FullyImplemented` with notes explaining why. A new
  `KNOWN_PARTIAL_REGISTRIES` fixture and
  `registry_coverage_status_matches_known_gaps` regression test now fail
  the build if a tracked-partial registry is promoted to
  `FullyImplemented` without updating the fixture, keeping the coverage
  table from silently overclaiming completeness again. The pre-existing
  `minecraft:loot_table` row's issue reference was also corrected from a
  closed, unrelated issue (#17) to the issue that actually tracks its gap
  (#185).

### Added — structured `sand run` diagnostics and vanilla registry façade

- **`sand run --server-log`.** Replaces the old raw-passthrough `--verbose`
  flag (kept as a deprecated hidden alias for `--server-log verbose`) with
  four modes: `classified` (default — Sand's filtered console with
  structured, phase-tagged datapack diagnostics), `verbose` (classified
  output plus the raw lines behind each event), `raw` (fully unfiltered
  passthrough), and `json` (structured diagnostics only, one JSON object
  per line, for editor/CI integrations).
- **Structured diagnostic model.** Every classified failure now carries an
  explicit `RunPhase` (`server_startup`, `datapack_discovery`, `reload`,
  `runtime`, `shutdown`, tracked from real process/command state) and a
  stable `DiagnosticCode` (command parse error, JSON/component error,
  missing reference, pack-format incompatibility, reload failure, startup
  failure, process exit, runtime command error, or unclassified).
  Repeated copies of the same root failure are folded into a trailing
  `repeated N times` note instead of reprinted every occurrence.
- **`sand::vanilla` module.** Discoverable, generated vanilla identifiers —
  `vanilla::Item`, `vanilla::Block`, `vanilla::EntityType`,
  `vanilla::SoundEvent` — reachable from the top-level `sand` façade and
  the prelude, converting into `ItemId`/`BlockId`/`EntityTypeId` via
  `.into()`.
- **Typed vanilla/custom entity types and items on normal paths.**
  `EntityTargets`/`Selector::entity_type`/`not_type`,
  `EntityQuery::entity_type`/`not_entity_type`, `cmd::summon` and friends,
  and `cmd::give` now accept generated vanilla identifiers
  (`vanilla::EntityType::Marker`, `vanilla::Item::Diamond`) and typed
  `EntityTypeId`/`ItemId` directly, in addition to the existing raw
  `&str`/`String` path (unchanged, still accepted — this is purely
  additive, no signature became stricter). See
  `sand_commands::selector::IntoEntityType` and
  `sand_core::cmd::IntoGiveItem`.
- **API-signature regression guard.** A new test
  (`sand/tests/api_signature_guard.rs`) scans the public façade's `pub fn`
  signatures for parameters that look like a typed identifier/target
  concept (item, block, entity type, resource refs) but still accept an
  untyped string, so new occurrences of the pattern this PR fixed don't
  reappear silently.

### Changed — API and compiler reorganization

- **Single public dependency.** Datapack projects now depend on one crate,
  `sand`, instead of `sand-core` + `sand-macros`. All supported attribute
  and function-like macros (`#[function]`, `#[component]`, `#[event]`,
  `#[item]`, `#[armor_event]`, `#[schedule]`, `run_fn!`, and the
  `resourcepack`-gated `hud_bar!`/`hud_element!`/`texture!`) are re-exported
  from `sand`.
- **New prelude and topic modules.** `use sand::prelude::*` covers ordinary
  authoring; less common APIs are organized under `sand::{event, item,
  state, command, component, entity, data, text, version, vfx}`.
  `sand::advanced` holds documented low-level export hooks and raw escape
  hatches.
- **CLI package renamed.** The CLI package is now `sand-cli`; the installed
  binary is still named `sand`. Author library builds no longer pull in
  CLI-only dependencies (clap, zip, handlebars, server management).
- **Removed deprecated APIs** (no replacement shims kept — Sand is pre-1.0):
  `InventorySlot`, `SlotPattern`, `Execute::if_items_pattern`,
  `Execute::unless_items_pattern` (use `ItemSlot` and its wildcard variants
  instead), `DamageTracker::{recently_damaged, damaged_at_least,
  delta_objective}` (use `damaged_this_tick`/`current_damage_at_least`),
  `StorageVar::set_raw`/`StorageField::set_raw` (use `set_raw_snbt`), the
  two-argument `CustomItem::raw_component(key, snbt)` form (use
  `with_raw_component`), and the `compat` module/`TypedEvent` alias.
- **Minecraft 26.2 is now the canonical target.** `sand_version::DEFAULT_CODEGEN_VERSION`
  and the primary correctness target for fixtures, examples, and the book
  moved from 1.21.11/1.21.4 to 26.2. Minecraft 1.21.4 is retained only as an
  explicit, clearly-labeled compatibility boundary in CI and version-profile
  tests.
- **Internal compiler reorganization.** `sand-core`'s former
  4,400-line `component.rs` export pipeline is now split into
  phase-scoped modules under `sand-core/src/compiler/export/`
  (collection, records, events, lifecycle, diagnostics, dialogs,
  predicates, tags, schedules, armor, functions). No generated output or
  public API changed.

See [`docs/architecture/adr-001-crate-boundaries.md`](docs/architecture/adr-001-crate-boundaries.md)
for the full design rationale and crate graph.

### Added

- A shared profile-aware fallible command validation/rendering boundary with
  pre-write function diagnostics, canonical fallible selector narrowing, and
  validated coordinate, slot, score-holder, objective, operation, and execute
  foundations. Typed values validate structurally; the final collected-string
  fallback always checks line integrity and only inspects confidently
  recognized top-level command argument positions, preserving literal
  JSON/SNBT plus unknown, macro, and modded syntax.

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
- **sand-macros**: `#[function("...")]` paths, `#[component(Tag = "...")]`
  tags, and `run_fn!("...")` paths are now validated at macro-expansion
  time (non-empty, lowercase `[a-z0-9_./-]`, single `:` namespace
  separator with a non-empty namespace and path) instead of failing later
  at export or runtime. Refs #122 — parameterized legacy `#[event(...)]`
  forms are not covered by this validation and remain open.
- **sand-components**: structured, export-time validation for enchantment,
  damage type, instrument, and jukebox-song components, replacing silent
  clamping and unchecked construction with diagnostics raised through the
  shared component-validation error path:
  - `Enchantment`: non-empty description and valid primary/exclusive
    resource-location syntax, non-empty `supported_items`, valid slots via
    the new typed `EnchantmentSlot` (re-exported from `sand-core`/
    `sand::prelude`), weight validated to `1..=1024`, max level validated to
    `1..=255`, and `effects` required to be a JSON object. Resolves #138.
  - `DamageType`: non-empty message ID with no whitespace-only value or
    control characters, and a finite, non-negative `exhaustion`. Resolves
    #139.
  - Instruments and jukebox songs: valid sound-event IDs and finite,
    positive duration/range/song length, with comparator output validated
    to `1..=15` instead of silently clamped. Resolves #140.

### Fixed

- Multiplayer score operations generated by `DamageTracker`, XP level-up
  events, timers, and cooldowns now execute per player with single-holder `@s`
  operands.
- User-declared function-tag entries are sorted by tag and function after
  Sand-owned setup entries, removing linker-order variation while preserving
  framework-before-user lifecycle semantics.

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
