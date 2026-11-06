# Independent CLI-only assessment: Vanilla Expedition

This is a compiling/exported vertical slice, not a complete adventure release. No native Minecraft session or server was launched. Generated commands/resources are evidence of export, not evidence that Minecraft 26.3 accepts or behaves correctly with every resource.

## Scores (10 = excellent)

| Dimension | Score | Rationale |
|---|---:|---|
| Discoverability | 5 | Excellent breadth and exportable signatures, but gameplay searches are polluted by substrings; macro-generated accessors are not taught; examples are frequently tautological; a returned score type is absent; scene registration workflow is omitted. `alternatives` is significantly more useful than search for native behavior. |
| Composability | 7 | Custom item/recipe, advancement/function, dialog/callback, predicates, shared dimension generator, and State conditions compose well. BlockId vs BlockState conversions and parallel inventory command/location surfaces force type hunting. Scene definition registration and start are separate with insufficient guidance. |
| Diagnostics | 7 | Rust diagnostics repaired both wrong State accessor syntax and block conversions. Export collision error names the exact resource identity. Freshness check is useful. Missing local function call from an unregistered Scene passed normal build and required artifact inspection. |
| Coverage | 8 | Broad typed resources and native command builders enabled every requested category at representative depth without explicit raw code. This score is for discoverable authoring/export breadth, not full vanilla schema parity or native runtime correctness. |

## Feature matrix

All implemented rows are native-unverified. Status “resource sample” means exported representative content rather than a finished gameplay subsystem.

| Category | Implementation | Status |
|---|---|---|
| Player progression/multiplayer | Persistent per-player kills and dimension crossings; five kills awards a custom enchanted blade; first-ever join gives compass; per-owner scene | Implemented/exported; initialization and @s commands inspected |
| Quests/advancements | Nether frontier advancement, 100 XP, apple and cache loot reward; combat/dimension event advancements | Implemented/exported |
| Custom items/recipes | Named/glint compass, namespaced custom data, shapeless iron/redstone recipe; Pathfinder Blade | Implemented/exported |
| Loot | Two-roll supply cache used by Nether reward | Implemented/exported |
| Predicates | Nether-location resource gates periodic objective HUD | Implemented/exported |
| Item modifiers | Typed set_name modifier applied from journal callback to held equipment | Implemented/exported |
| Combat/events | Built-in kill event increments personal state and emits milestone reward; not a bespoke combat engine | Implemented/exported |
| Menus/dialog/text/HUD | Quick Actions journal, survival callbacks, objectives, cosmetic equipment inscription, actionbar | Implemented/exported; numeric HUD deferred due missing returned score type contract |
| Sound/particles | Level-up sound and happy-villager particles at frontier reward | Implemented/exported |
| Display entities | Five-second owner-bound veteran text scene with explicit resource registration | Implemented/exported after registration repair |
| Teams/bossbars | Explorer team membership; shared season banner refreshed every ten seconds | Implemented/exported; banner is not individual progress |
| World/block interaction | Operator camp function replaces air at feet with campfire | Implemented/exported; not an automatic terrain generator |
| Tags/functions/scheduling/storage | Supplies tag, lifecycle setup contribution, scheduled season pulse, global season status, generated State storage | Implemented/exported |
| Dimension/type | Typed superflat daylight sanctuary with bedrock/dirt/grass | Resource sample exported; no sanctuary transport/gameplay claim |
| Worldgen/structure references | Additional structure set reuses vanilla trail_ruins with typed random-spread placement | Resource sample exported; worldgen native-unverified |
| Enchantment | Pathfinder typed linear damage effect, iron-sword support, used on veteran blade | Implemented/exported |
| Full worldgen | Biome, noise, feature, carver, jigsaw, binary structure authoring | Catalog probed; deferred, not proven unsupported |
| Rich combat/quest graph | Entity participants, AI/boss encounters, branching quest persistence and economy | Deferred to bound experiment |
| Compass interaction | Right-click activation/navigation UX | Deferred; compass identity/recipe only |
| Native validity | Minecraft registry parsing, loaded/unloaded ownership, multiplayer actual behavior | Unverified by explicit experiment constraint |

## Raw/weakly-typed boundary inventory

No raw command, RawJson, RawSnbt, or handwritten command text was used. Some supported APIs still accept strings: a generated team command takes a rendered typed target; schedule takes `"10s"`; Inventory::modify takes an ID rendered from ResourceLocation; StorageVar takes a namespaced storage/path pair. These are observable API type seams, not a claim that they are all dedicated raw escape hatches.

## Specific CLI fixes

1. Rank token/semantic gameplay matches before substring matches. `quest` should not prioritize `requested_version`.
2. Add complete macro recipes: declare State, use generated associated field constant, mutate through event, query it, display a numeric value, and explain lifecycle initialization.
3. Audit all signature return/parameter types for catalog reachability; EntityScoreValue is referenced but `show` cannot resolve it.
4. Add runnable end-to-end Scene/Lifetime examples with component registration AND start. Explicitly explain that start emits a reference only.
5. Validate unresolved locally owned function references (at least generated Sand-system references) during export. The successful unregistered-scene build is a concrete reproduction.
6. Document that `minecraft:load`/`tick` must use lifecycle/function-tag contributions when compiler-owned tags exist. Improve collision diagnostics with the exact alternative.
7. Surface conversion support for `impl Into<T>` arguments. Show accepted generated vanilla and typed identifier inputs; consistently support BlockId where practical.
8. Do not classify every `Raw*` spelling as a raw escape hatch: RawCopper is an ordinary vanilla item.
9. Prefer practical examples over `fn demonstrate(value: X) { value.method(arg); }`. Show imports, construction, registration, invocation, and expected native output together.
10. Cross-link canonical live ItemLocation and command Inventory so authors can locate modifiers without guessing module names.

## Experiment boundary

Scores and recommendations assess the frozen 11,625-entry CLI catalog used during discovery. The parent subsequently began repairs to the framework. The final project's dependency compilation may include those repairs; no updated API catalog or framework source was used to solve the authoring task. Parent-induced baseline-ratchet interruption is excluded from scores. Original successful-but-dangling scene evidence is preserved separately.

## Final reproduction result

`build-06.log`: success, 96 components, 49.6 seconds. `sand-check-final.log`: “Sand agent validation passed.” `final-verification.log`: all artifact assertions and local function-reference checks passed, 98 files including metadata. `cargo fmt --check`: passed. Final source is in `/tmp/sand-api-authoring-2026-11-05/vanilla_expedition`; generated pack is its `dist/vanilla_expedition` directory. Approximate elapsed exercise effort: 14 minutes, including builds and concurrent dependency interruption.
