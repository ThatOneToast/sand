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
    .gamerule(sand::command::gamerule_keep_inventory().value(true))
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

// Flat — see chapter 29
let flat = Generator::Flat(FlatGenerator::new(vec![
    FlatLayer::new(ResourceLocation::new("minecraft", "grass_block").unwrap(), 1),
]));

// Void — see chapter 30
let void = Generator::Void;

// Noise — full vanilla or referenced custom settings
let noise = Generator::Noise(NoiseGenerator::vanilla(VanillaNoiseSettings::Overworld));

// Vanilla debug block-state showcase
let debug = Generator::Debug;

// Unsupported generator schemas use an explicit inline JSON object.
let custom = Generator::Raw(sand::component::RawJson::new(serde_json::json!({
    "type": "my_pack:custom_generator"
})));
```

`NoiseGenerator` also supports referencing a custom `noise_settings`
resource you author directly (`NoiseGenerator::custom_settings`), and
overriding biome placement to a single fixed biome
(`NoiseGenerator::single_biome`) — vanilla's "single biome" preset. Use the typed worldgen resource builders for supported schemas and explicit
raw objects for unsupported edges in advanced
Minecraft worldgen authoring.

See [Custom Dimensions](./30-custom-dimensions.md) for the full custom-
dimension workflow, and [Generated World Resources](./34-generated-world-resources.md)
for exactly what JSON each generator variant produces.

### Noise preset biome placement

`NoiseGenerator::vanilla` selects biome placement to match its terrain preset:
Nether uses `minecraft:multi_noise` with the Nether preset, End uses
`minecraft:the_end`, and other presets use Overworld multi-noise placement.
Custom noise settings also default to Overworld placement.
`single_biome(...)` overrides any of these defaults with a fixed biome.
