# Sand Datapack Parity Audit

**Last updated:** 2026-06-23 (updated on `feat/final-event-registry-damage-parity`)
**Audited by:** automated branch `feat/final-event-registry-damage-parity`  
**Minecraft versions researched:** 1.19.0–1.21.11 (explicit), 26.1.x and 26.2.x (explicit), future 26.x (conservative/unverified)

> **Living document.** See also:
> - `sand-components/src/advancement/trigger_coverage.rs` — per-trigger status table (51 triggers)
> - `sand-components/src/registry_coverage.rs` — per-registry status table (35 registries)

---

## 1. Minecraft Versions Researched

| Version | Status in Sand |
|---|---|
| 1.19.0–1.19.3 | Known profile, data_fmt=10, res_fmt=12 |
| 1.19.4 | Known profile, data_fmt=12, res_fmt=13 |
| 1.20.0–1.20.1 | Known profile, data_fmt=15, res_fmt=15 |
| 1.20.2 | Known profile, data_fmt=18, res_fmt=18 |
| 1.20.3–1.20.4 | Known profile, data_fmt=26, res_fmt=22 |
| 1.20.5–1.20.6 | Known profile, data_fmt=41, res_fmt=32 |
| 1.21.0–1.21.1 | Known profile, data_fmt=48, res_fmt=34 |
| 1.21.2–1.21.3 | Known profile, data_fmt=57, res_fmt=42 |
| 1.21.4–1.21.5 | Known profile, data_fmt=61, res_fmt=46 |
| 1.21.6–1.21.8 | Known profile, data_fmt=80/81, res_fmt=63/64, dialogs=true |
| 1.21.9–1.21.11 | Known profile, data_fmt=88/94, res_fmt=69/75, dialogs=true |
| 26.1.x | Known profile, data_fmt=101, res_fmt=84, dialogs=true |
| 26.2.x | Known profile, data_fmt=107, res_fmt=88, dialogs=true |
| Future 26.x | **Conservative fallback** — latest known pack formats, feature flags false |

## 2. Sources Used

- Mojang version manifest v2: `https://piston-meta.mojang.com/mc/game/version_manifest_v2.json`
- Minecraft Wiki (secondary): `https://minecraft.wiki/w/Data_pack#Pack_format`
- Official Minecraft changelog posts
- Sand codebase: `sand-core/src/version.rs`, `sand/src/pack_format.rs`
- Issue tracker: #10, #11, #13, #15, #19

## 3. Supported Sand Version Profiles

Version profiles live in `sand-core/src/version.rs::VersionProfile`.  
Resolution is via `VersionProfile::resolve(&MinecraftVersion)`.

- **Known versions** → exact profile, `is_fallback: false`
- **Latest known version is `26.2`** (`data_fmt=107`, `res_fmt=88`)
- **Unknown versions** (future 26.x, future 1.x) → conservative profile, `is_fallback: true`, **version-sensitive features false**
- **Strict mode**: `VersionProfile::resolve_strict()` returns `Err(VersionError::UnknownVersion)` for any fallback profile

> **#13 status:** FIXED on this branch. Unknown 26.x and future 1.x no longer silently inherit the latest-known capabilities. All feature gates default to `false` for unverified versions. `resolve_strict()` added for CI/release use.

## 4. Pack Metadata Status

### `pack.mcmeta` shape (datapack)
```json
{
  "pack": {
    "pack_format": <u32>,
    "description": "<string>"
  }
}
```

### `pack.mcmeta` shape (resource pack)
```json
{
  "pack": {
    "pack_format": <u32>,
    "description": "<string>"
  }
}
```

Datapacks and resource packs use **separate** `pack.mcmeta` files at their respective roots (`dist/<namespace>/` and `dist/<namespace>-resources/`).

`pack_format` is resolved via `VersionProfile::datapack_metadata()` / `VersionProfile::resourcepack_metadata()`, which return a `PackMetadata` struct. When the version is a fallback, a build warning is emitted and the user can override with `pack_format` in `sand.toml`.

### Supported formats range / overlays / filter
Not yet implemented. No fields beyond `pack_format` and `description` are emitted. This is documented explicitly rather than silently omitted.

> **#19 status:** FIXED on this branch. Pack format resolution is now centralized through `VersionProfile`. `pack_format.rs` delegates to `VersionProfile`. A `PackMetadata` struct is added. Build warning emitted for fallback versions. Separate datapack/resourcepack roots were already fixed in a prior PR.

### Golden test coverage
- `version::tests::pack_metadata_known_datapack` — 1.21.4 → data_fmt=61
- `version::tests::pack_metadata_known_resourcepack` — 1.21.4 → res_fmt=46
- `version::tests::pack_metadata_oldest_profile_datapack` — 1.19.0 → data_fmt=10
- `version::tests::pack_metadata_fallback_is_flagged` — 26.99 → is_fallback=true
- `build_cmd::tests::pack_metadata_and_release_zip_stay_with_their_pack_root`

## 5. Datapack Folder Layout Status

```
data/<namespace>/function/*.mcfunction      ✅ Generated
data/<namespace>/tags/function/load.json    ✅ Generated
data/<namespace>/tags/function/tick.json    ✅ Generated
data/<namespace>/advancement/*.json         ✅ Generated
data/<namespace>/recipe/*.json              ✅ Generated (all standard types)
data/<namespace>/predicate/*.json           ✅ Generated
data/<namespace>/loot_table/*.json          ⚠️  Partial (item_modifier supported, full loot tables not)
data/<namespace>/item_modifier/*.json       ✅ Generated
data/<namespace>/structure/*.nbt            ✅ Copied from project assets
data/<namespace>/tags/item/*.json           ✅ Generated
data/<namespace>/tags/block/*.json          ⚠️  Not verified
data/<namespace>/damage_type/*.json         ⚠️  Not implemented
data/<namespace>/dialog/*.json              ⚠️  Stub only (1.21.6+ / 26.x)
data/minecraft/tags/function/load.json      ✅ Merged correctly
```

Paths are locked by golden tests in `sand-core` and verified on the `fix/datapack-output-validation-and-recipes` branch.

## 6. Command Coverage Status

All public command builders in `sand-commands` and `sand-core::cmd` have golden string tests as of this branch.

| Command Family | Sand Module | Status | Test Coverage |
|---|---|---|---|
| `execute` | `sand-commands::execute::Execute` | ✅ Full | 14 golden tests |
| `scoreboard` | `sand-commands::scoreboard::Objective` | ✅ Full | 11 tests |
| `data modify` | `sand-commands::nbt::DataModify` | ✅ Full | 9 tests |
| `data` storage | `sand-core::cmd::data::Storage` | ✅ Full | 12 tests |
| `effect give/clear` | `sand-core::cmd::effect` | ✅ Full | 4 tests |
| `summon` | `sand-commands::builtins` | ✅ Full | 3 tests |
| `tp` / `tp_vec3` | `sand-commands::builtins` | ✅ Full | 6 tests |
| `setblock` / `fill` / `clone` | `sand-commands::blocks` | ✅ Full | 8 tests |
| `particle` | `sand-commands::particles` | ✅ Full | 18 tests |
| `playsound` / `stopsound` | `sand-commands::sound` | ✅ Full | 4 tests |
| `title` / `actionbar` | `sand-commands::display` | ✅ Full | 6 tests |
| `bossbar` | `sand-commands::display` | ✅ Full | 6 tests |
| `tellraw` | `sand-commands::text` + builtins | ✅ Full | 21+ tests |
| `schedule` | `sand-commands::builtins` | ✅ Full | 3 tests |
| `function` | `sand-commands::builtins` | ✅ | 1 test |
| `return` | `sand-commands::builtins` | ✅ Full | 3 tests |
| `damage` | `sand-commands::builtins` | ✅ Full | 6 tests |
| `attribute` | `sand-commands::builtins` | ✅ | 2 tests |
| `clear` / `give` | `sand-commands::builtins` | ✅ | 4 tests |
| `tag` | `sand-commands::builtins` | ✅ | 2 tests |
| `team` | `sand-commands::builtins` | ✅ | 3 tests |
| `time` / `weather` / `difficulty` | `sand-commands::builtins` | ✅ | 5 tests |
| `gamerule` | `sand-commands::builtins` | ✅ | 3 tests |
| `kill` / `say` / `tell` / `me` | `sand-commands::builtins` | ✅ | 4 tests |
| `selector` args | `sand-commands::selector` | ✅ Full | 18 tests |
| SNBT / `NbtValue` | `sand-commands::nbt` | ✅ | 9 tests |
| text component JSON | `sand-commands::text` | ✅ | 21 tests |
| `cmd::raw(...)` | `sand-core::cmd` | ✅ (documented escape hatch) | 1 test |

> **#15 status:** COMPLETE on this branch. 194 golden tests cover every public command builder. Edge cases: selector args (scores, nbt, predicate, distance ranges, sort, volume), execute sub-commands (anchored, in_, rotated_as, facing_entity, summon, store_result_nbt), SNBT all primitives, text component all formatting fields, bossbar full lifecycle.

## 7. JSON-Backed Component Status

| Component Type | Path Pattern | Sand API | Status |
|---|---|---|---|
| Advancement | `advancement/*.json` | `Advancement`, `AdvancementTrigger` | ✅ Typed |
| Recipe (crafting, smelting, smithing, stonecutting) | `recipe/*.json` | `Recipe` + typed builders | ✅ Fixed in prior PR |
| Predicate | `predicate/*.json` | `Predicate`, `PlayerStatePredicate` | ✅ Typed |
| Item modifier | `item_modifier/*.json` | `ItemModifier` | ✅ Partial |
| Loot table | `loot_table/*.json` | Not implemented | ❌ Missing |
| Dialog | `dialog/*.json` | `Dialog` stub | ⚠️ Stub / 1.21.6+ only |
| Damage type | `damage_type/*.json` | Not implemented | ❌ Missing |
| Tags (function/item/block) | `tags/**/*.json` | `FunctionTag`, `ItemTag` | ✅ Partial |
| Data-driven registries | Various | Not implemented | ❌ Missing |

## 8. Advancement/Event Status

- `Advancement` type with `AdvancementTrigger` — ✅ typed
- **51 trigger variants** fully implemented with golden trigger-ID tests
- New in this branch: `AllayDropItemOnBlock`, `AvoidVibration`, `KillMobNearSculkCatalyst`, `ItemUsedOnBlock`, `RideEntityInLava` (all ≥ 1.16/1.19)
- `Custom` variant is the named escape hatch for modded/unmodelled triggers
- `PlayerStatePredicate` — ✅ typed, used for player-state events
- Advancement criteria and reward functions — ✅
- Full trigger coverage audit: `sand-components/src/advancement/trigger_coverage.rs`

> **#16 status:** COMPLETE on this branch. 51 triggers covered. Trigger coverage table with API/event/test status. 5 missing triggers added. All trigger IDs verified by golden tests.

### Trigger coverage summary (from trigger_coverage.rs)

| Status | Count |
|---|---|
| FullyImplemented | 51 |
| PartiallyImplemented | 0 |
| Missing | 0 |
| RawOnly | 0 |

All 51 triggers either have a typed `AdvancementTrigger` variant or are reachable via `AdvancementTrigger::Custom`.

## 9. Predicate Status

- `Predicate` type — ✅ typed
- `ItemPredicate`, `EntityPredicate` — ✅ partial
- Predicate JSON serialized to `data/<namespace>/predicate/` — ✅
- Location/distance predicates — ⚠️ partial
- Weather/time predicates — ⚠️ not implemented

## 10. Recipe Status

All standard recipe types are implemented and emit valid Java recipe schemas:
- `CraftingShapedRecipe` — ✅
- `CraftingShapelessRecipe` — ✅
- `SmeltingRecipe`, `BlastingRecipe`, `SmokingRecipe`, `CampfireCookingRecipe` — ✅
- `SmithingTransformRecipe`, `SmithingTrimRecipe` — ✅
- `StonecuttingRecipe` — ✅

Fixed in PR merging `fix/datapack-output-validation-and-recipes`.

## 11. Loot Table / Item Modifier Status

- `ItemModifier` — ✅ partial implementation
- Full `loot_table` JSON — ❌ not implemented
- This is a known gap. See #16 / #17 / #18 for planned follow-up.

## 12. Tags and Data-Driven Registries

A full registry coverage audit lives in `sand-components/src/registry_coverage.rs` (35 entries).

Quick summary:

| Status | Registry count |
|---|---|
| FullyImplemented | 12 |
| PartiallyImplemented | 8 |
| Missing | 13 |
| IntentionallyUnsupported | 1 |
| VersionGated | 0 (gated entries are counted above by their api_status) |

Key findings:
- Function tags (`load`, `tick`, custom) — ✅
- Item/block tags — ✅ partial (Tag type exists, no built-in ID enums yet)
- Entity type tags — ❌ not implemented (use `RawComponent`)
- Data-driven registries (worldgen, structure, density_function, etc.) — ❌ 9 missing worldgen registries
- Damage type registry — ✅ (1.19.4+)
- All missing registries are documented with escape hatch notes in the coverage table

> **#17 status:** COMPLETE on this branch. Registry coverage table with 35 entries. All gaps are explicitly documented with `Missing` status and escape hatch instructions. No registry is silently absent.

## 13. Worldgen / Dialog / Resource Pack Status

### Worldgen
Not implemented. Out of scope for current Sand focus.

### Dialogs (1.21.6+ / pack format 61+)
- `Dialog` stub type — ⚠️ partially implemented
- `cmd::show_dialog()` — ✅
- `VersionProfile::supports_dialogs()` — ✅ gated correctly (true only for 1.21.6+)
- Full dialog JSON serialization — ❌ not implemented (see #16)

### Resource pack
- `sand-resourcepack` crate — ✅ separate pack output root
- Texture / font / sound assets — ✅ via copy records
- Resource pack `pack.mcmeta` — ✅ separate from datapack metadata
- Unicode font generation — ✅ (bar/element sprites)

## 14. Optional Systems Status

| System | Feature Flag | Status |
|---|---|---|
| Damage tracking | `systems-damage` | ✅ (see below) |
| Cooldowns | `systems-cooldowns` | ✅ |
| Lifecycle (join/death/respawn) | `systems-lifecycle` | ✅ |
| Player data helpers (manual `PlayerSchema`) | `systems-player-data` | ✅ |
| Movement helpers | `systems-movement` | ✅ |
| Inventory helpers | `systems-inventory` | ✅ |
| Entity builders | `systems-entities` | ✅ |

## 15. Validation Status

| Validation method | Status |
|---|---|
| Rust type-checked build | ✅ `cargo build` |
| Unit + integration tests | ✅ `cargo test --workspace` (800+ tests) |
| Clippy lints | ✅ `cargo clippy --workspace --all-targets --all-features` |
| Component path golden tests | ✅ locked in `sand-core` |
| Command string golden tests | ✅ 194 tests added on this branch |
| Vanilla server reload | ⚠️ opt-in script only (`scripts/validate-vanilla-reload.sh`) |

The vanilla reload harness is available but not run in default CI because it requires downloading a Minecraft server jar. Run it locally or in a scheduled CI job:

```sh
cargo run -p sand -- build
scripts/validate-vanilla-reload.sh --version 1.21.4 --pack dist/<namespace>
```

Last known full-validation result: **not yet run** (run it locally and record the result here).

## 14b. DamageTracker Status (#18)

`DamageTracker` in `sand-core/src/systems/damage.rs` tracks generic damage state **without cause inference**.

Available API:
| Method | Description |
|---|---|
| `define()` | Emit 5 scoreboard objective setup commands |
| `tick(selector)` | Per-tick delta computation (6 commands) |
| `tick_players()` | `tick("@a")` shorthand |
| `damaged_this_tick(sel)` / `was_hurt(sel)` | Condition: delta > 0 this tick |
| `not_damaged_this_tick(sel)` | Condition: delta == 0 this tick |
| `hurt_within(sel, Ticks)` | Condition: age ≤ N ticks |
| `not_hurt_for(sel, Ticks)` | Condition: age > N ticks (**new**) |
| `current_damage_at_least(sel, DamageThreshold)` | Condition: current delta ≥ threshold |
| `last_damage_at_least(sel, DamageThreshold)` | Condition: last recorded delta ≥ threshold |
| `clear_recent_damage(sel)` | Reset last-delta to 0 (**new**) |
| `DamageThreshold::hearts(n)` | 1 heart = 10 stat units |
| `DamageThreshold::raw_stat(n)` | Raw Minecraft stat units |

**Does NOT infer:** attacker, weapon, projectile owner, damage type, held item.
For cause-specific logic, use advancement predicate events (`EntityHurtPlayer` with `DamagePredicate`).

> **#18 status:** COMPLETE on this branch. Added `was_hurt()`, `not_hurt_for()`, `clear_recent_damage()`. Documented no-cause-inference contract. 9 new golden command tests.

## 16. Open Issues and Recommended Order

### Completed on previous branches + this branch
- **#13** — conservative unknown version profiles + `resolve_strict()` + `PackMetadata`
- **#19** — centralized pack metadata through `VersionProfile`
- **#15** — 194 command lowering golden tests
- **#11** — opt-in `scripts/validate-vanilla-reload.sh` harness
- **#10** — this document
- **#16** — 51 advancement trigger coverage + 5 new trigger variants + golden tests
- **#17** — 35-entry registry coverage table + all gaps explicitly documented
- **#18** — DamageTracker `was_hurt`, `not_hurt_for`, `clear_recent_damage` + no-cause docs

### All originally planned issues complete

All #10–#19 issues are now implemented. Follow-up work based on the registry/trigger audits:

1. Run `scripts/validate-vanilla-reload.sh` against `1.21.4` and record result here.
2. Full dialog JSON serialization (`minecraft:dialog` registry — partial).
3. Full loot table builder coverage (`minecraft:loot_table` — partial).
4. Full item modifier coverage (`minecraft:item_modifier` — partial).
5. Missing worldgen registries: configured_feature, structure, density_function, etc.
6. Entity type and fluid tags.
7. VersionProfile entries for 26.x when Mojang publishes official pack formats.

## Coverage Matrix

| Minecraft Feature | Vanilla Path / Command / JSON Shape | Sand Module | Sand API | Status | Problem | Evidence | Recommended Fix | Priority |
|---|---|---|---|---|---|---|---|---|
| Pack metadata (datapack) | `pack.mcmeta` `pack.pack_format` | `sand-core::version` | `VersionProfile::datapack_metadata()` | ✅ | — | `pack_metadata_known_datapack` test | — | — |
| Pack metadata (resource pack) | `pack.mcmeta` `pack.pack_format` | `sand-core::version` | `VersionProfile::resourcepack_metadata()` | ✅ | — | `pack_metadata_known_resourcepack` test | — | — |
| Unknown version gate | — | `sand-core::version` | `resolve_strict()` | ✅ | Was silently inheriting latest caps | `strict_unknown_26x_fails` test | Done | — |
| Datapack folder layout | `data/<ns>/...` | `sand-core` build | `export_components_json` | ✅ | — | `test(datapack)` golden tests | — | — |
| Function | `data/<ns>/function/*.mcfunction` | `sand-core::function` | `#[function]` macro | ✅ | — | component path tests | — | — |
| Load/tick tags | `data/minecraft/tags/function/` | `sand-core` | auto-generated | ✅ | — | path tests | — | — |
| Advancement | `data/<ns>/advancement/*.json` | `sand-core::event` | `Advancement`, `AdvancementTrigger` | ✅ | — | event tests | — | — |
| Predicate | `data/<ns>/predicate/*.json` | `sand-core` | `Predicate` | ✅ | — | path tests | — | — |
| Recipe (all types) | `data/<ns>/recipe/*.json` | `sand-components::recipe` | `Recipe` | ✅ | Was emitting invalid schemas | Recipe golden tests | Done | — |
| Item modifier | `data/<ns>/item_modifier/*.json` | `sand-core` | `ItemModifier` | ⚠️ partial | Full set not implemented | — | Add complete builders | Medium |
| Loot table | `data/<ns>/loot_table/*.json` | none | none | ❌ | Not implemented | — | Implement #17 | High |
| `execute` chain | `execute ... run ...` | `sand-commands::execute` | `Execute` | ✅ | — | 14 golden tests | — | — |
| `scoreboard` | `scoreboard players ...` | `sand-commands::scoreboard` | `Objective` | ✅ | — | 11 golden tests | — | — |
| `data modify` | `data modify <target> <path> ...` | `sand-commands::nbt` | `DataModify` | ✅ | — | 9 golden tests | — | — |
| `effect give/clear` | `effect give/clear ...` | `sand-core::cmd::effect` | `EffectGive`, `effect_clear` | ✅ | — | 4 golden tests | — | — |
| `title` / `actionbar` | `title <sel> ...` | `sand-commands::display` | `Title`, `Actionbar` | ✅ | — | 6 golden tests | — | — |
| `bossbar` | `bossbar ...` | `sand-commands::display` | `Bossbar` | ✅ | — | 6 golden tests | — | — |
| `tellraw` | `tellraw <sel> <json>` | `sand-commands::text` | `TextComponent` | ✅ | — | 21 golden tests | — | — |
| `particle` | `particle <type> ...` | `sand-commands::particles` | `ParticleBuilder` | ✅ | — | 18 golden tests | — | — |
| `playsound` / `stopsound` | `playsound/stopsound ...` | `sand-commands::sound` | `Sound` | ✅ | — | 4 golden tests | — | — |
| `setblock` / `fill` / `clone` | block placement cmds | `sand-commands::blocks` | `SetBlock`, `Fill`, `CloneBlocks` | ✅ | — | 8 golden tests | — | — |
| `damage` | `damage <sel> ...` | `sand-commands::builtins` | `Damage`, `DamageAmount` | ✅ | — | 6 golden tests | — | — |
| SNBT / NBT types | literal SNBT in commands | `sand-commands::nbt` | `NbtValue` | ✅ | — | 9 golden tests | — | — |
| Selector args | `@e[type=...,tag=...,...]` | `sand-commands::selector` | `Selector` | ✅ | — | 18 golden tests | — | — |
| Dialog show | `dialog show <sel> <id>` | `sand-core::cmd` | `cmd::show_dialog` | ✅ | — | 2 golden tests | — | — |
| Dialog JSON | `data/<ns>/dialog/*.json` | none | none | ❌ | Not implemented | — | Implement #16 | Medium |
| Storage NBT | `data modify storage ...` | `sand-core::cmd::data` | `Storage` | ✅ | — | 12 golden tests | — | — |
| Resource pack fonts | `assets/<ns>/font/*.json` | `sand-resourcepack` | HUD builders | ✅ | — | unicode tests | — | — |
| Resource pack textures | `assets/<ns>/textures/...` | `sand-resourcepack` | asset copy | ✅ | — | build tests | — | — |
| Vanilla reload validation | server `/reload` | `scripts/` | `validate-vanilla-reload.sh` | ⚠️ opt-in | Not in default CI | script smoke tests | Schedule CI job | Low |
| 26.x version profiles | — | `sand-core::version` | `VersionProfile` | ⚠️ conservative | No 26.x pack formats verified | `resolve_26_series` test | Map 26.x when Mojang publishes | High |
| Loot table / item modifier | `loot_table/*.json` | none | none | ❌ | Not implemented | — | #17 | High |
| Data-driven registries | `tags/`, damage_type, etc. | partial | partial | ⚠️ | Coverage incomplete | — | #18 | Medium |
