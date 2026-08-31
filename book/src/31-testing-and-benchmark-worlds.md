# 31. Testing And Benchmark Worlds

🌍 World (datapack) for everything generated; 🖥️ Server (host) for the
`ServerConfig` half of these profiles (world-reset policy in particular).

`test` and `bench` are two of the four well-known [`BuildProfile`](./25-development-and-release-profiles.md)
values, each shaped around a specific need beyond ordinary `dev` iteration.

## Testing worlds (`test` profile)

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

## Benchmark worlds (`bench` profile)

Benchmarking needs the *opposite* of a clean-every-time world in one
respect: a **fixed, pre-generated region**, so generation cost doesn't leak
into whatever you're actually measuring (tick performance, function
execution time, …). Use a fixed seed with `WorldResetPolicy::Keep` so the
region generates once and stays generated:

```rust
use sand::build::{ServerConfig, WorldResetPolicy};

let server = ServerConfig::new().world_reset_policy(WorldResetPolicy::Keep);
```

Combine this with Sand's benchmark tooling (see `BENCHMARKS.md` in the
repository root) for repeatable performance measurement. **Status note:**
this issue introduces the typed `bench` profile and
`WorldResetPolicy::Keep`/`AlwaysReset` primitives; it does not yet wire
`bench`-profile worlds into `BENCHMARKS.md`'s existing benchmark harness —
that integration is tracked as a follow-up (see the PR that introduced this
chapter for the exact status).

## Choosing seeds deliberately

Both profiles benefit from an explicit `Seed::Fixed(n)` rather than
`Seed::Random` — reproducibility is the entire point of a `test` or `bench`
profile. `dev` and `release` profiles more commonly leave the seed
unspecified (`Seed::Random`, effectively vanilla's own random-seed
behavior at world creation).
