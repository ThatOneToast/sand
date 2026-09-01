# 26. Release Profiles

`release` is the [`BuildProfile`](./25-development-profiles.md) a project
ships: `sand build --release` defaults `--profile` to `release`
automatically when no explicit `--profile` is passed, so the common case
("build what I'm about to distribute") needs no extra flag.

## Why release gets full vanilla generation

Where `dev` optimizes for turnaround time, `release` optimizes for being
the actual world players get — normally full vanilla noise generation
rather than flat/void placeholder terrain:

```rust,ignore
let overworld = Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld)
    .generator(Generator::Noise(NoiseGenerator::vanilla(VanillaNoiseSettings::Overworld)));
```

`sand build --profile dev` produces a `minecraft:flat` generator;
`sand build --release` produces a `minecraft:noise` generator referencing
vanilla's Overworld noise settings — genuinely different generated JSON,
not just a cosmetic label change. `sand-core`'s own test suite asserts on
this directly
(`dev_and_release_worlds_produce_different_dimension_json` in
`sand-core/src/build/resources.rs`), and
`examples/book_project/sand.build.rs` demonstrates the full split with a
real, buildable project.

## `ServerConfig` for release

🖥️ A `release` profile's `ServerConfig` is still 🖥️ Server (host) only —
it only affects `sand run`'s local dev server, never the exported datapack
— but it's common to widen it back toward vanilla defaults, since `release`
testing is meant to reflect what a real deployment sees more closely than
`dev`'s deliberately narrowed local-iteration settings:

```rust,ignore
.server(
    ServerConfig::new()
        .view_distance(if ctx.profile().is_dev() { 6 } else { 10 })
        .world_reset_policy(if ctx.profile().is_dev() {
            WorldResetPolicy::AlwaysReset
        } else {
            WorldResetPolicy::Keep
        }),
)
```

`WorldResetPolicy::Keep` (the default) makes sense for `release` since it's
not meant for disposable local iteration — a persistent local world lets
you play-test a release build across multiple `sand run` sessions the way
an actual player would experience it. See
[Server Configuration](./31-server-configuration.md) for the full type.

## Seeds

`release` (like `dev`) most commonly leaves the seed unspecified
(`Seed::Random`, effectively vanilla's own random-seed behavior at world
creation) — reproducibility matters for
[testing and benchmark profiles](./32-testing-worlds.md), not for what
ships.
