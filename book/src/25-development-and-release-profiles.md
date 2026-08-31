# 25. Development And Release Profiles

`BuildProfile` selects which world/server configuration a `sand.build.rs`
script produces. `sand build --profile <name>` / `sand run --profile <name>`
resolve it; both default to `dev` when omitted (`sand build --release`
without an explicit `--profile` defaults to `release` instead).

## The well-known profiles

| Profile | Typical intent |
|---|---|
| `dev` | Fast local iteration — flat/void worlds, auto-reset, minimal generation cost. |
| `test` | Deterministic worlds for automated testing — fixed seed, known structures. |
| `bench` | Reproducible performance benchmarking — fixed seed, fixed pre-generated region. |
| `release` | The shipped configuration — typically full vanilla noise generation. |

Anything else becomes `BuildProfile::Custom("your-name")` — useful for a
`staging` profile or similar project-specific need.

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

## Dev vs. release, concretely

The starter `sand.build.rs` from `sand add worldbuild` demonstrates the
canonical pattern — flat terrain for `dev`/`test`, full noise for everything
else:

```rust,ignore
let overworld = if ctx.profile().is_dev() || ctx.profile().is_test() {
    Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(
        Generator::Flat(FlatGenerator::new(vec![
            FlatLayer::new(ResourceLocation::new("minecraft", "bedrock").unwrap(), 1),
            FlatLayer::new(ResourceLocation::new("minecraft", "dirt").unwrap(), 2),
            FlatLayer::new(ResourceLocation::new("minecraft", "grass_block").unwrap(), 1),
        ])),
    )
} else {
    Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld)
        .generator(Generator::Noise(NoiseGenerator::vanilla(VanillaNoiseSettings::Overworld)))
};
```

`sand build --profile dev` produces a `minecraft:flat` generator;
`sand build --release` (or `--profile release`) produces a `minecraft:noise`
generator referencing vanilla's Overworld noise settings — genuinely
different generated JSON, not just a cosmetic label change. `sand-core`'s
own test suite asserts on this directly
(`dev_and_release_worlds_produce_different_dimension_json` in
`sand-core/src/build/resources.rs`).

## Server-side profile differences

🖥️ `ServerConfig` can differ per profile too — for example, a `dev` profile
might use `WorldResetPolicy::AlwaysReset` for a clean slate every run, while
`release` keeps `WorldResetPolicy::Keep` (the default) since it's not
actually used for local iteration:

```rust,ignore
.server(
    ServerConfig::new()
        .world_reset_policy(if ctx.profile().is_dev() {
            WorldResetPolicy::AlwaysReset
        } else {
            WorldResetPolicy::Keep
        }),
)
```

See [Server Configuration](./30-server-configuration.md) for the full type,
and remember: none of `ServerConfig`'s fields travel with the exported
datapack, regardless of profile.
