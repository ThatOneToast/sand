# 27. Worlds, Dimensions, And Generators

🌍 World (datapack) — everything in this chapter lowers into resources
inside the exported datapack.

## `World`

[`World`](https://github.com/ThatOneToast/sand) is the top-level 🌍 type: a
build script constructs one, configures it, and hands it to
`SandBuild::world`.

```rust
use sand::build::{Spawn, TimeConfig, WeatherConfig, World, WorldBorder};

let world = World::new()
    .spawn(Spawn::at(0, 64, 0))
    .border(WorldBorder::diameter(6000.0))
    .gamerule("keepInventory", "true")
    .time(TimeConfig::set(6000).frozen())
    .weather(WeatherConfig::Clear);
```

Spawn, border, gamerules, time, and weather all lower into one generated
function, `data/<namespace>/function/__sand_world_init.mcfunction`, wired
into vanilla's `minecraft:load` tag alongside anything your ordinary
`#[function(Load)]` hooks already contribute (Sand merges the tag file
rather than overwriting it).

The one exception is `World::preset` (`WorldPreset`) — see
[Server Configuration](./31-server-configuration.md#worldpreset) for why
that field is 🖥️ despite living on `World`.

## Dimensions

```rust
use sand::build::{Dimension, DimensionSlot, DimensionType, Dimensions, Generator};

let dims = Dimensions::new()
    .with(Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld))
    .with(Dimension::new(DimensionSlot::Nether, DimensionType::Nether));
```

- **`DimensionSlot`** — which dimension a `Dimension` occupies:
  `Overworld`, `Nether`, `End`, or `Custom(ResourceLocation)` for a
  brand-new dimension.
- **`DimensionType`** — the dimension_type reference used (skylight, height
  limits, coordinate scale, …): `Overworld`, `OverworldCaves`, `Nether`,
  `End`, or `Custom(ResourceLocation)` for a project-authored
  `dimension_type` resource.

Omitting a vanilla dimension from `Dimensions` leaves it at default vanilla
generation — Sand only overrides what you explicitly configure.

## Generators

`Generator` selects a dimension's chunk generator:

```rust
use sand::build::{FlatGenerator, FlatLayer, Generator, NoiseGenerator, VanillaNoiseSettings};
use sand::ResourceLocation;

// Flat — see chapter 27
let flat = Generator::Flat(FlatGenerator::new(vec![
    FlatLayer::new(ResourceLocation::new("minecraft", "grass_block").unwrap(), 1),
]));

// Void — see chapter 28
let void = Generator::Void;

// Noise — full vanilla or referenced custom settings
let noise = Generator::Noise(NoiseGenerator::vanilla(VanillaNoiseSettings::Overworld));

// A generator resource you authored by hand and are only referencing
let custom = Generator::CustomReference(
    ResourceLocation::new("my_pack", "hand_authored").unwrap(),
);
```

`NoiseGenerator` also supports referencing a custom `noise_settings`
resource you author directly (`NoiseGenerator::custom_settings`), and
overriding biome placement to a single fixed biome
(`NoiseGenerator::single_biome`) — vanilla's "single biome" preset. Sand
does not attempt to express recursive density functions in the typed API;
advanced worldgen definitions stay hand-authored resources you reference,
matching the project's explicit non-goal of not replacing general advanced
Minecraft worldgen authoring.

See [Custom Dimensions](./30-custom-dimensions.md) for the full custom-
dimension workflow, and [Generated World Resources](./34-generated-world-resources.md)
for exactly what JSON each generator variant produces.
