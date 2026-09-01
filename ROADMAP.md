# Sand Roadmap

Sand is pre-1.0 and evolving; this file tracks genuinely future direction
only. For what's already shipped, see the [book](book/src/introduction.md)
and `CHANGELOG.md`.

## Target versions

Minecraft Java 26.2 is the canonical export/profile target; 1.21.4 is
retained as an explicit oldest-profile/compatibility boundary. See
`sand-version/src/lib.rs` and `docs/architecture/adr-001-crate-boundaries.md`.

## Not yet stable

- **Event system.** `SandEvent` composition (same-cycle chained dispatch,
  multi-parent `after_any`/`after_all`, persistent `while_<E>()` conditions,
  bounded `.within(...)` correlation, advancement-backed graph parents) is
  implemented but not macro-transparent: authors must call
  `EventSetup::with_participants(...)` themselves, nothing auto-merges
  participant capabilities into graph propagation, and there is no typed
  `Event<T>` handler-context accessor yet. Victim, interacted-entity, and
  projectile-owner participant recovery are unimplemented.
- **Resource pack generation** — functional but requires manual asset setup.
- **crates.io publishing** — not yet available; install from the workspace
  (`cargo install --path sand-cli`).
- **Typed build-time world/server configuration (#317, #356)** — `sand::build`
  (`SandBuild`/`BuildContext`/`BuildProfile`, `World`/`Dimensions`/
  generators, `ServerConfig`) shipped covering flat/void/noise generators,
  world border, spawn, gamerules/time/weather, and `sand build`/`sand run
  --profile`. Validation combines structural checks (ranges, duplicate
  slots, layer sanity) with a biome-registry check generated from real
  per-version Minecraft data (`sand-build/src/codegen/biomes.rs`); not yet a
  full `sand-vanilla-audit` real-server audit of world-build resources, or
  registry coverage beyond biomes — see "Next work" below.

## Next work

- Expand golden export tests to full datapack directory fixtures.
- Add a typed item stack builder with component API.
- Harden dialog actions with typed function references.
- Complete resource pack example crates.
- Extend the registry codegen pipeline (`sand-build/src/codegen/biomes.rs`)
  to cover `worldgen/structure`/`worldgen/structure_set` alongside biomes,
  and extend `sand-vanilla-audit` with a real-server load pass for
  `sand::build`-generated world resources — broader coverage than the
  generated biome check alone (#317 follow-up, tracked as issues #355/#356
  follow-on work).
- Wire the `bench` `BuildProfile`/`WorldResetPolicy` primitives into a real
  in-game runtime benchmark harness (TPS/chunk-gen throughput) — today's
  `BENCHMARKS.md` only measures Cargo/exporter build time (#317 follow-up,
  tracked as issue #357).
