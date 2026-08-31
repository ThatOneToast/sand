# 27. Flat Worlds

🌍 World (datapack).

`Generator::Flat` lowers to a `minecraft:flat` chunk generator — the same
mechanism as vanilla's superflat preset, built from an ordered layer stack:

```rust
use sand::build::{FlatGenerator, FlatLayer, Generator};
use sand::ResourceLocation;

let generator = Generator::Flat(
    FlatGenerator::new(vec![
        FlatLayer::new(ResourceLocation::new("minecraft", "bedrock").unwrap(), 1),
        FlatLayer::new(ResourceLocation::new("minecraft", "dirt").unwrap(), 2),
        FlatLayer::new(ResourceLocation::new("minecraft", "grass_block").unwrap(), 1),
    ])
    .biome(ResourceLocation::new("minecraft", "plains").unwrap())
    .with_structures(false),
);
```

- **Layers** — `FlatLayer::new(block, height)`, listed bottom-to-top. Total
  height is validated against the vanilla build-height limit (384 blocks);
  a zero-height layer or an empty layer list is also rejected — see
  [Generated World Resources](./32-generated-world-resources.md#validation)
  for the exact diagnostics.
- **Biome** — `FlatGenerator::biome(...)`. Defaults to `minecraft:plains`
  (vanilla's superflat default) if not set. A flat world uses exactly one
  biome for every column.
- **Structures** — `FlatGenerator::with_structures(true)` enables vanilla
  structure generation (villages, strongholds, …) on top of the flat
  terrain; disabled by default, matching vanilla's superflat behavior.

## Why flat for `dev`/`test`

Flat generation is essentially free — there's no noise sampling, no biome
blending, nothing to pre-generate. That makes it the natural choice for
`dev` (fast iteration) and `test` (deterministic terrain a test can assert
on) profiles; see [Development Profiles](./25-development-and-release-profiles.md).

```rust,ignore
let overworld = if ctx.profile().is_dev() {
    Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld)
        .generator(Generator::Flat(FlatGenerator::new(vec![
            FlatLayer::new(ResourceLocation::new("minecraft", "grass_block").unwrap(), 1),
        ])))
} else {
    // ... noise generation for release
};
```

## Generated JSON

A flat generator lowers to:

```json
{
  "type": "minecraft:flat",
  "settings": {
    "biome": "minecraft:plains",
    "lakes": false,
    "features": false,
    "layers": [
      { "block": "minecraft:bedrock", "height": 1 },
      { "block": "minecraft:dirt", "height": 2 },
      { "block": "minecraft:grass_block", "height": 1 }
    ],
    "structure_overrides": []
  }
}
```

See [Void Worlds](./28-void-worlds.md) for the special case of no terrain
at all.
