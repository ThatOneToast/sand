# Sand Roadmap

## Stable Direction

- Attribute-first datapack authoring with `#[function]`, `#[component(Load)]`,
  and `#[component(Tick)]`.
- Typed state (`ScoreVar`, `Flag`, `Cooldown`, `StorageVar`, `Timer`), typed
  conditions (`all!`, `any!`), typed execute, typed text, typed storage, typed
  selectors, and typed datapack components.
- Generated typed command builders for all Minecraft commands (advancement,
  recipe, execute, give, playsound, etc.).
- Explicit escape hatches through `cmd::raw(...)` and advanced `mcfunction!`
  command collection.

## Current Status (Alpha)

Sand is in alpha dogfooding stage. The core APIs are stable enough to build
real datapacks. The following are working:

- `#[function]`, `#[component(Load)]`, `#[component(Tick)]` proc macros
- Typed state: `ScoreVar<T>`, `Flag`, `Cooldown`, `StorageVar<T>`, `Timer`
- Typed conditions: `all!`, `any!`, `Condition::*`
- Typed execute: `TypedExecute`, `ExecuteExt::when`/`unless`
- Typed text: `Text`, `Actionbar`, `Title`, `Bossbar`
- Typed commands: generated from Minecraft command tree (advancement, recipe,
  execute, give, playsound, particle, etc.)
- Typed datapack components: advancement, recipe, loot table, predicate, tag,
  dialog, custom item, damage type, enchantment, and more
- Version gating: `VersionProfile` for feature detection
- Scaffold: `sand new` generates attribute-first typed projects

## Experimental Areas

- Dialog command helpers and dialog registration/export ergonomics.
- Resource-pack and HUD workflows.
- Generated registry coverage for future Minecraft 26.x releases.

## Target Versions

Sand targets modern Minecraft Java datapacks across 1.19 through 1.21.x and the
emerging 26.x series. Capability decisions flow through `VersionProfile`.

## Not Yet Stable

- `mcfunction!` macro — available but positioned as advanced tooling, not the
  beginner path.
- Event system — `AdvancementEvent`/`SandEvent` split formalized, with typed
  tick dispatch, lifecycle/setup, generic identity, deterministic single- and
  multi-parent same-cycle composition (`after`, `after_any`, `after_all`), and
  explicit player-scoped persistent `while_<E>()` conditions. Bounded
  correlation and advancement-parent phases of #240 and participant contexts
  #230 remain.
- Resource pack generation — functional but requires manual setup.
- crates.io publishing — not yet available; build from workspace.

## Next Work

- Expand golden export tests to full datapack directory fixtures.
- Add typed item stack builder with component API.
- Harden dialog actions with typed function refs.
- Complete resource pack example crates.
