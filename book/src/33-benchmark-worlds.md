# 33. Benchmark Worlds

🌍 World (datapack) for everything generated; 🖥️ Server (host) for the
`ServerConfig` half (world-reset policy in particular).

`bench` is one of the four well-known
[`BuildProfile`](./26-release-profiles.md) values, shaped around
reproducible performance measurement.

## Why a dedicated `bench` profile

Benchmarking needs the *opposite* of a clean-every-time world in one
respect: a **fixed, pre-generated region**, so generation cost doesn't leak
into whatever you're actually measuring (tick performance, function
execution time, …). Use a fixed seed with `WorldResetPolicy::Keep` so the
region generates once and stays generated, typically alongside full vanilla
noise generation (the same terrain shape you'd ship, so the benchmark stays
representative):

```rust
use sand::build::{Generator, NoiseGenerator, Seed, ServerConfig, VanillaNoiseSettings, WorldResetPolicy};

let generator = Generator::Noise(NoiseGenerator::vanilla(VanillaNoiseSettings::Overworld));
let seed = Seed::Fixed(42);
let server = ServerConfig::new().world_reset_policy(WorldResetPolicy::Keep);
```

`examples/book_project/sand.build.rs` wires this up as a genuinely distinct
third branch (not just falling through to the `release` case): `bench`
gets full vanilla noise generation, `Seed::Fixed(42)`, and
`WorldResetPolicy::Keep`, so `sand build --profile bench` /
`sand run --profile bench` produce a real, reproducible benchmark world
today.

## Runtime measurements

`scripts/bench_runtime.sh` starts a real server against this profile. Its RCON
workload records server-reported milliseconds per tick and the throughput of a
fresh 256-chunk forced region, then writes machine-readable JSON under
`target/bench-runtime/`. Shutdown is scoped to that owned server process.
`BENCHMARKS.md` documents the methodology alongside the build-time timings.

## Choosing a seed deliberately

Both `test` and `bench` profiles should set an explicit `Seed::Fixed(n)`
rather than `Seed::Random` — reproducibility is the entire point. See
[Testing Worlds](./32-testing-worlds.md) for the related, but distinct,
`test` profile.
