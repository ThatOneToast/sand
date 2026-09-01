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
- **Typed build-time world/server configuration (#317, #355, #356, #357,
  #358)** — `sand::build` (`SandBuild`/`BuildContext`/`BuildProfile`,
  `World`/`Dimensions`/generators, `ServerConfig`) shipped covering
  flat/void/noise generators, world border, spawn, gamerules/time/weather,
  and `sand build`/`sand run --profile`. Validation is now version-aware
  for biomes (`SandBuild::validate_for_context`, generated from real
  per-version Mojang `biome_parameters` reports, #356) and
  `sand-vanilla-audit` exercises generated world resources against a real
  running server (#355). `scripts/bench_runtime.sh` gives `BuildProfile::
  Bench` a real, if v1-scoped, runtime measurement (server-ready wall
  time, #357) — steady-state TPS/chunk throughput is still not measured.
  See "Next work" below for what's left.

## Next work

- Expand golden export tests to full datapack directory fixtures.
- Add a typed item stack builder with component API.
- Harden dialog actions with typed function references.
- Complete resource pack example crates.
- Extend `sand-vanilla-audit`'s registry validation beyond biomes (e.g.
  dimension-type/noise-settings references) against a selected
  `VersionProfile`'s real registries (#356 follow-up).
- Extend `scripts/bench_runtime.sh` (#357) to measure steady-state TPS or
  chunk-generation throughput away from spawn, not just server-ready wall
  time — needs a scripted in-game workload, not just log-scraping.
- `World::gamerule` takes a free-form string Sand doesn't validate or
  translate across Minecraft versions; Minecraft 26.2 renamed many
  gamerules to snake_case (e.g. `doDaylightCycle` -> `advance_time`),
  discovered via #358's real-server verification. Consider a typed
  gamerule enum (or at least a version-aware rename-detection diagnostic)
  as a future follow-up — filed as #360.
