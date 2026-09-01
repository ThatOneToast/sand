# 25. Development Profiles

`BuildProfile` selects which world/server configuration a `sand.build.rs`
script produces. `sand build --profile <name>` / `sand run --profile <name>`
resolve it; both default to `dev` when omitted (`sand build --release`
without an explicit `--profile` defaults to `release` instead — see
[Release Profiles](./26-release-profiles.md)).

## The well-known profiles

| Profile | Typical intent |
|---|---|
| `dev` | Fast local iteration — flat/void worlds, auto-reset, minimal generation cost. |
| `test` | Deterministic worlds for automated testing — fixed seed, known structures. |
| `bench` | Reproducible performance benchmarking — fixed seed, fixed pre-generated region. |
| `release` | The shipped configuration — typically full vanilla noise generation. |

Anything else becomes `BuildProfile::Custom("your-name")` — useful for a
`staging` profile or similar project-specific need. See
[Testing Worlds](./32-testing-worlds.md) and
[Benchmark Worlds](./33-benchmark-worlds.md) for `test`/`bench` specifically.

## Branching on the profile

```rust
use sand::build::{BuildContext, BuildProfile};

fn describe(ctx: &BuildContext) -> &'static str {
    if ctx.profile().is_dev() {
        "flat, fast-iteration world"
    } else if ctx.profile().is_release() {
        "full vanilla noise world"
    } else {
        "other profile"
    }
}

let ctx = BuildContext::new(BuildProfile::Dev);
assert_eq!(describe(&ctx), "flat, fast-iteration world");
```

`BuildProfile` predicates (`is_dev`, `is_test`, `is_bench`, `is_release`,
`is_custom`) are mutually exclusive, so an `if`/`else if` chain like the one
above is exhaustive in practice without needing a `match` on the enum.

## Why `dev` gets fast, disposable worlds

Local iteration cares about turnaround time, not terrain quality: a `dev`
profile typically pairs [flat](./28-flat-worlds.md) or
[void](./29-void-worlds.md) generation (near-instant, no noise sampling)
with `ServerConfig::world_reset_policy(WorldResetPolicy::AlwaysReset)` so
every `sand run` starts from a clean, predictable world instead of
accumulating changes from previous sessions:

```rust,ignore
let overworld = Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(
    Generator::Flat(FlatGenerator::new(vec![
        FlatLayer::new(ResourceLocation::new("minecraft", "bedrock").unwrap(), 1),
        FlatLayer::new(ResourceLocation::new("minecraft", "dirt").unwrap(), 2),
        FlatLayer::new(ResourceLocation::new("minecraft", "grass_block").unwrap(), 1),
    ])),
);

let server = ServerConfig::new()
    .view_distance(6) // smaller render distance = faster local startup
    .world_reset_policy(WorldResetPolicy::AlwaysReset);
```

`sand add worldbuild`'s starter `sand.build.rs` follows exactly this
pattern. See [Server Configuration](./31-server-configuration.md) for the
full `ServerConfig` type — remember none of its fields travel with the
exported datapack, regardless of profile.
