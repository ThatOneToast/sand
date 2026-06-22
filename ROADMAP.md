# Sand Roadmap

## Stable Direction

- Attribute-first datapack authoring with `#[function]`, `#[component(Load)]`,
  and `#[component(Tick)]`.
- Typed state, typed conditions, typed execute, typed text, typed storage, typed
  selectors, and typed datapack components.
- Explicit escape hatches through `cmd::raw(...)` and advanced `mcfunction!`
  command collection.

## Experimental Areas

- Dialog command helpers and dialog registration/export ergonomics.
- Event rustdocs and event examples that still need typed-state rewrites.
- Resource-pack and HUD workflows.
- Generated registry coverage for future Minecraft 26.x releases.

## Target Versions

Sand targets modern Minecraft Java datapacks across 1.19 through 1.21.x and the
emerging 26.x series. Capability decisions should flow through `VersionProfile`.

## Next Work

- Add typed advancement grant/revoke builders.
- Convert legacy event docs away from raw command snippets.
- Promote example crates for playable datapacks.
- Expand golden export tests to full datapack directory fixtures.
