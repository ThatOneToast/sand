# CLI-only authoring diary

Started shortly after the scaffold's 22:48 local timestamp on 2026-11-05. All Sand knowledge came from `sand api`, its exported catalog, the prepared scaffold, compiler diagnostics, and generated project output. No framework source, dependency source, docs/examples, internet, native game/server, skills, or other agents were consulted. Local catalog filtering is a view of CLI export, not another reference source.

## Exact initial CLI commands

- `sand api --help`
- `sand api export --help`
- `sand api module sand`
- `sand api export -o /tmp/sand-api-authoring-2026-11-05/evidence/catalog.json`
- `sand api show sand::function`
- `sand api show sand::state::State`
- `sand api show sand::system`
- `sand api search quest`
- `sand api search recipe`
- `sand api alternatives 'summon text_display'`
- `sand api show sand::entity::EntityScoreValue`
- `sand api search 'custom biome'`
- `sand api alternatives 'item modify'`

Further exact catalog queries are in `queries.log`; `query.py` prints only the catalog's public contracts. Initial bulk catalog filters searched PlayerState, State::, Advancement::, CustomItem::, Recipe::, FirstJoin, PlayerKilled, dimension, worldgen. `worldgen` found no paths because resource types sit directly under component. Broad filters emitted too much material, so later inspection used exact component prefixes. All evidence lives beside the project.

## Discovery observations and repairs

1. `quest` produced 108 matches; top entries were `requested_version` and `requested`, substring matches unrelated to quests. `recipe` also ranks command overloads and fields ahead of authoring recipes. Saved search output reproduces this.
2. Export is genuinely useful: 11,625 public entries and a machine-readable catalog, with clear signatures and native behavior. But “coverage complete” is declaration coverage, not recipe/tutorial completeness.
3. State's derive example only declares a field. It never shows accessing the generated field constant. Initial guessed `Expedition::kills()` failed E0618; diagnostic said it was an EntityScore. Changed to `Expedition::kills`; compile succeeded. See compile-01/02.
4. `EntityScoreAccessor::get` returns `EntityScoreValue`; `sand api show sand::entity::EntityScoreValue` reports unknown. Numeric HUD presentation was deferred rather than guess internals. Static objective guidance HUD is implemented.
5. Resource composition was smooth: CustomItem -> RecipeResult preserves components; AdvancementRewards takes a typed function; DialogAction::callback supplies survival-friendly dispatch; Dimension shares FlatGenerator with build configuration.
6. Display searches found registry enum variants, while `alternatives 'summon text_display'` correctly found Scene::text. That was decisive for implementing owned, expiring multiplayer scene effects without raw NBT.
7. BlockId accepted by FlatLayer is NOT accepted by BlockContext::matches/replace. compile-03 reported missing From<BlockId> and suggested generated vanilla Block. Used CLI-confirmed Block::Air/Campfire. This is both a composability seam and missing conversion documentation, not proof that block interaction is unsupported.
8. build-01 succeeded: 34 resources / 35 files. Generated state lifecycle initializes per-player scores and dispatches events as @s.
9. build-02 compiled but export rejected ordinary TypedTag<FunctionId> at minecraft:load because generated lifecycle owns that output. Error clearly identifies duplicate identity but offers no lifecycle alternative. Repaired with DatapackRegistration + LifecycleContribution::load, discovered in the catalog.
10. `alternatives 'item modify'` finds command::Inventory::modify, while inventory::ItemLocation has no modify method. The “Advanced/raw escape hatches” section incorrectly lists vanilla RawCopper/RawGold items. Saved evidence demonstrates lexical rather than semantic raw classification.

## Build environment

Every Cargo/Sand build uses `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0`. No clean was run. The provided .cargo configuration is unchanged. Initial cargo check compiled dependencies; repeated cargo check versus build produced additional ~47-second framework compilations. No native runtime was launched.

## Scope decisions

Aim: connected exploration/crafting/combat progression with broad resource samples, not a claim to implement every field of vanilla. No explicit raw command/JSON/SNBT escape hatch is needed in this slice. Generated command builders still take several strings (team name/member selector rendering, sound ResourceLocation, schedule duration, modifier ID string); values originate from typed targets/IDs where possible. Global status storage is intentionally world-wide; player counters are State scoreboard fields, not one shared storage slot.

Deferred: numeric score HUD (undiscoverable return type), compass right-click mechanics, biome/noise/carver/feature authoring beyond the dimension example, custom binary structures/jigsaw pools, entity AI/spawner systems, rich combat participant logic, economy, full quest graph, terrain-safe sanctuary travel, exhaustive enchantment effect variants. These are not asserted impossible. Data-driven structure placement reuses vanilla trail ruins; no authored NBT template is claimed.

11. build-03 succeeded (46 components/47 files), but artifact assertions failed: there was no text_display command. `veteran_reward.mcfunction` called `__sand_system/lifetime/veteran_scene/start`, which did not exist. Scene::start returns a call but does not register the scene. Scene and Scene::start contracts do not explain the need to register a definition; their examples only import or call start. The normal build did not detect the unresolved local function. Preserved pre-fix command as `unregistered-scene-reward.mcfunction`. Added a `#[datapack_component]` scene factory and reused it for start. This is a CLI workflow gap plus validation gap, not lack of display implementation.
12. `sand check` run while sources were newer than the prior export correctly reported SAND_CHECK_STALE_OUTPUT. That is expected freshness validation, not a framework failure. A final check follows final build.

13. Final scene-registration build temporarily blocked by external concurrent framework edits: Sand build script observed 11628 static surface items while baseline recorded 11625 (build-05.log). No project diagnostic was reached. Parent notified to stabilize dependency tree. This is experiment infrastructure interference, excluded from authoring diagnostics score.

## Completion

Finished 2026-11-06 04:02 UTC (2026-11-05 23:02 local), approximately 14 minutes after the prepared scaffold timestamp, including dependency builds and parent-induced interruption. Build-06 succeeded in 49.6 seconds with 96 registered components. Final export inspection counted 98 files including build metadata, parsed every JSON, asserted 11 representative resource categories, verified custom recipe identity, personal score writes, the five-kill reward, display commands, predicate HUD, equipment modifier, camp block operation, schedule and bossbar, and checked that every locally owned literal function target exists. `sand check` and `cargo fmt --check` passed. See final-verification.log and sand-check-final.log. No native proof is claimed.

Explicit scene registration expanded the export from 46 to 96 components; the missing function is now supplied. Final source uses only discovered typed builders; no raw escape hatch was required. The local code was formatted. Framework-wide scripts/check.sh is outside this isolated project and delegated to the parent, who owns framework changes; this authoring agent did not read or alter that repository.
