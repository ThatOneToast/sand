# 32. Testing Worlds

🌍 World (datapack) for everything generated; 🖥️ Server (host) for the
`ServerConfig` half (world-reset policy in particular).

`test` is one of the four well-known
[`BuildProfile`](./25-development-profiles.md) values, shaped around
automated-testing needs distinct from ordinary `dev` iteration.

## Why a dedicated `test` profile

Automated tests need a **deterministic** world: a fixed seed, known
structures placed at known coordinates, and terrain that doesn't vary
between runs. Flat generation with an explicit fixed-seed dimension is the
natural fit:

```rust
use sand::build::{Dimension, DimensionSlot, DimensionType, FlatGenerator, FlatLayer, Generator, Seed, World};
use sand::ResourceLocation;

fn test_world() -> World {
    World::new()
        .seed(Seed::Fixed(42))
        .dimensions(sand::build::Dimensions::new().with(
            Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(
                Generator::Flat(FlatGenerator::new(vec![FlatLayer::new(
                    ResourceLocation::new("minecraft", "grass_block").unwrap(),
                    1,
                )])),
            ),
        ))
}
```

Pair this with `ServerConfig::world_reset_policy(WorldResetPolicy::AlwaysReset)`
so every `sand run --profile test` starts from the same clean world —
critical for tests that assert on world state (chest contents, structure
placement, scoreboard values seeded by a setup function) rather than
player-driven state that would otherwise accumulate across runs.

## Choosing a seed deliberately

A `test` profile should always set an explicit `Seed::Fixed(n)` rather than
`Seed::Random` — reproducibility is the entire point. `dev` and `release`
profiles more commonly leave the seed unspecified (`Seed::Random`,
effectively vanilla's own random-seed behavior at world creation). See
[Benchmark Worlds](./33-benchmark-worlds.md) for the related, but distinct,
`bench` profile.
