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
  multi-parent same-cycle composition (`after`, `after_any`, `after_all`),
  explicit player-scoped persistent `while_<E>()` conditions, bounded
  cross-tick correlation (`within::<E>(TickWindow)`), and advancement-backed
  graph parents bridged from their own reward function as a sole `after`
  dependency (#240 Phase 6). Typed item locations and immutable event-time
  item snapshots (#229 Phase 7) give SandEvent authors a way to capture an
  item's identity before vanilla mutates/consumes it, manually embedded into
  a handler's own setup/body; not yet auto-wired into `#[event]` codegen.
  Typed participant reliability/availability/lifetime and event context
  capability descriptors (#230 Phase 8) establish the vocabulary and graph
  propagation/merge rules future participant recovery will use. Correlated
  attacker/killer entity observation (#230 Phase 9,
  `observe_correlated_attacker`, backed by vanilla's `execute on attacker`
  relation) is the first real participant-recovery backend, manually
  embedded per event; victim, interacted-entity, and projectile-owner
  recovery remain unimplemented, and no automatic participant capability is
  attached to built-in event families.
- Resource pack generation — functional but requires manual setup.
- crates.io publishing — not yet available; build from workspace.

## Next Work

- Expand golden export tests to full datapack directory fixtures.
- Add typed item stack builder with component API.
- Harden dialog actions with typed function refs.
- Complete resource pack example crates.
