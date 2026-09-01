# 36. Migrating From `sand.toml`

**Honest status:** as of this feature, `sand.toml`'s `[pack]` section
(`namespace`, `description`, `mc_version`, `pack_format`,
`supported_formats`, `overlays`) and `[resourcepack]` section have **no**
world- or server-shaped fields to migrate away from. Sand never had
built-in `sand.toml` world/server configuration before issue #317 — there
was no `[world]` or `[server]` table to deprecate. This chapter is
therefore forward-looking: it explains the new typed API's relationship to
`sand.toml`, not a migration off of something that existed.

## Nothing is deprecated

Because there was nothing to migrate, **no `sand.toml` keys are deprecated
by this feature**, and `sand build`/`sand run` print no deprecation
warning. `sand.toml` continues to own pack-level metadata
(namespace/description/version/format) exactly as before;
`sand.build.rs` is purely additive, optional configuration layered on top.

## `sand migrate`

Running `sand migrate` reflects this directly — it explains that there are
no `sand.toml` world/server fields to move, then scaffolds a starter
`sand.build.rs` (the same output as `sand add worldbuild`) so you can start
using the typed API if you want to:

```console
$ sand migrate
sand migrate

  Note: sand.toml has no world/server fields to migrate. Sand's previous
  configuration surface was limited to [pack] (namespace, description,
  mc_version, pack_format, supported_formats, overlays) and [resourcepack]
  — neither ever covered world generation, dimensions, gamerules, or server
  bootstrap settings, so there is nothing to move out of sand.toml.

  Scaffolding a starter sand.build.rs so you can start using the typed
  World/ServerConfig API described in the "Migrating from Sand.toml"
  mdBook chapter:

Adding sand.build.rs to my_pack...
  created sand.build.rs
  updated Cargo.toml
```

## Adopting the typed API on an existing project

If your project currently hand-authors `data/<namespace>/dimension/*.json`
(or relies entirely on vanilla default generation), adopting
`sand.build.rs` is optional and additive:

1. Run `sand add worldbuild` (or `sand migrate` — identical scaffolding).
2. Move any hand-authored dimension JSON into the typed builders where they
   fit ([Worlds, Dimensions, And Generators](./27-worlds-dimensions-and-generators.md)),
   or reference them unchanged via `Generator::CustomReference`/
   `DimensionType::Custom` if they're too advanced for the typed API (see
   [Custom Dimensions](./30-custom-dimensions.md#non-goal-advanced-worldgen-authoring)).
3. If your project's dev workflow has any manual `server.properties`
   editing you do by hand every time you set up a fresh test server, that's
   a candidate for [`ServerConfig`](./31-server-configuration.md).

## Follow-up: full `sand.toml` scope review

`sand.toml` may grow world/server-adjacent fields in the future as Sand
evolves (this chapter will be updated with real deprecation guidance if
that ever happens). Tracking a full `sand.toml` field audit against the
typed API as a deliberate follow-up issue is recommended so this chapter
stays accurate rather than becoming stale documentation for a migration
that was never needed.
