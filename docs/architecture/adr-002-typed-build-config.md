# ADR 002 — Typed build-time world/server configuration (`sand::build`)

Date: 2026-08-31
Status: accepted
Scope: issue #317's typed `SandBuild`/`BuildContext`/`BuildProfile`/`World`/
`Dimensions`/generator/`ServerConfig` API and its `sand-cli` integration.

## Context

Issue #317 asked for a `sand.build.rs` build script (modeled on Rust's own
`build.rs`) that lets a project select a `BuildProfile` and construct a
typed `World`/`ServerConfig`, with Section 3.1 of the issue suggesting
`sand-build` as the primary home for the new types (`SandBuild`,
`BuildContext`, `BuildProfile`, World/Dimension/Generator builders).

[ADR-001](./adr-001-crate-boundaries.md) already fixes `sand-build`'s scope
precisely: codegen (`sand-build/src/codegen/*`) and Minecraft server jar
management (`download.rs`, `cache.rs`, `manifest.rs`) — an internal
implementation crate `sand-cli` depends on, never a home for new authoring
vocabulary. ADR-001 also establishes the general pattern every other
authoring surface in this workspace follows: `sand-core` (and siblings)
hold the implementation, `sand` re-exports it as a curated façade module,
and authors only ever add `sand` to `Cargo.toml`.

## Decision

**Deviate from issue #317 Section 3.1's suggested crate placement.**
Instead of adding the new types to `sand-build`:

- The typed authoring API (`SandBuild`, `BuildContext`, `BuildProfile`,
  `World`, `Dimensions`, `Dimension`, `DimensionSlot`, `DimensionType`,
  `Generator`/`FlatGenerator`/`FlatLayer`/`NoiseGenerator`/
  `NoiseSettingsRef`/`VanillaNoiseSettings`/`BiomeSource`, `WorldBorder`,
  `Spawn`/`SpawnPlatform`, `TimeConfig`, `WeatherConfig`, `Seed`,
  `WorldPreset`, and the separate `ServerConfig`/`Difficulty`/
  `WorldResetPolicy`) lives in a new `sand_core::build` module —
  implementation only, never imported directly by authors.
- `sand` gains a new façade module, `sand::build`, re-exporting exactly
  that surface, following the identical pattern already used for
  `sand::vfx`, `sand::state`, `sand::entity`, etc. A new `build-source`
  scope was added to `sand/api-scopes.toml` (mirroring `vfx-source`), and
  the committed API-surface ratchet (`sand/api-surface-baseline.txt`,
  `sand/api-surface-profiles.toml`) was updated for the ~160 newly
  reachable items this adds to Sand's enforced public surface.
- `sand-cli` is responsible for *discovering and driving* a project's
  `sand.build.rs`, exactly as it already drives `sand-core`'s ordinary
  export pipeline (`sand_export`). Concretely: `sand.build.rs` is wired in
  as an ordinary Cargo `[[bin]] name = "sand_build_world"` target (added by
  the new `sand add worldbuild` subcommand, mirroring the existing
  `sand add resourcepack` pattern for `sand_resource_export`), compiled the
  same way (`cargo build --bin sand_build_world`), and run with
  `SAND_BUILD_PROFILE`/`SAND_EXPORT_MC_VERSION` environment variables. Its
  JSON stdout (world resources + optional server config) is parsed and
  merged into `dist/` by a new `sand-cli/src/build/worldbuild.rs` module,
  which lives beside (and reuses the wire format of) the existing
  `export.rs`/`records.rs` component-export machinery rather than
  inventing a second build pipeline.
- `sand-build` itself is untouched by this issue — no new types, no new
  responsibilities. It remains scoped to codegen and server-jar management
  exactly as ADR-001 describes.

## Consequences

- Authors who want typed world/server configuration add nothing new to
  `Cargo.toml` beyond the `[[bin]]` target `sand add worldbuild` inserts —
  `sand` is still the only crate dependency, consistent with ADR-001's
  "one dependency, one import" goal.
- `sand-cli`'s relationship to `sand.build.rs` mirrors its relationship to
  `sand_export`/`sand_resource_export` exactly: compile via Cargo, run the
  binary, parse one JSON value from stdout. No new build-system concepts
  were introduced.
- `ServerConfig` is a structurally separate type from `World`, reinforced
  at the file-writing layer: `sand-cli` writes `World`-derived resources
  into `dist/<namespace>/data/...` and `ServerConfig` into
  `dist/.sand-server-config.json` — a sibling of `dist/<namespace>/`, never
  inside it, so it can never be accidentally packaged into the datapack or
  synced into a running server's `world/datapacks/` directory.
- `docs/architecture/adr-001-crate-boundaries.md`'s crate graph is
  unchanged by this ADR — no new crate was added, and no existing crate
  changed its dependency direction. This ADR only documents where within
  the existing `sand-core → sand` façade pattern the new types landed, and
  why that differs from the issue's own suggested placement.
