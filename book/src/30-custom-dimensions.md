# 30. Custom Dimensions

🌍 World (datapack).

Beyond overriding the vanilla Overworld/Nether/End, `Dimensions` supports
brand-new dimensions at your own resource location:

```rust
use sand::build::{Dimension, DimensionSlot, DimensionType, Dimensions, Generator, NoiseGenerator, VanillaNoiseSettings};
use sand::ResourceLocation;

let sky_realm = Dimension::new(
    DimensionSlot::Custom(ResourceLocation::new("my_pack", "sky_realm").unwrap()),
    DimensionType::Custom(ResourceLocation::new("my_pack", "sky_realm_type").unwrap()),
)
.generator(Generator::Noise(
    NoiseGenerator::vanilla(VanillaNoiseSettings::FloatingIslands),
));

let dims = Dimensions::new().with(sky_realm);
```

This writes `data/my_pack/dimension/sky_realm.json`, referencing
`my_pack:sky_realm_type` as its dimension type.

## Custom dimension types

`DimensionType::Custom` only supplies the *reference* — Sand does not
author `data/my_pack/dimension_type/sky_realm_type.json` for you. Author
that file by hand (dimension types are a fixed-shape vanilla resource with
no typed builder in Sand today — see the "Advanced worldgen" note below)
and place it under your project's `src/`-adjacent datapack structure, or
reuse one of the vanilla types instead:

```rust
// Reuse vanilla Overworld physics for a custom-slot dimension — perfectly
// valid, and much less to hand-author.
let sky_realm = Dimension::new(
    DimensionSlot::Custom(ResourceLocation::new("my_pack", "sky_realm").unwrap()),
    DimensionType::Overworld,
);
```

## Reusing a custom noise settings resource

If your custom dimension needs terrain shaping beyond the vanilla presets,
author a `data/<namespace>/worldgen/noise_settings/<path>.json` resource by
hand and reference it:

```rust
use sand::build::NoiseGenerator;

let generator = Generator::Noise(NoiseGenerator::custom_settings(
    ResourceLocation::new("my_pack", "sky_realm_noise").unwrap(),
));
```

## Non-goal: advanced worldgen authoring

Sand's typed API deliberately does not attempt to express recursive density
functions, or otherwise replace general advanced Minecraft worldgen
authoring. `Generator::CustomReference` and `NoiseSettingsRef::Custom` are
the escape hatches — Sand validates the *reference* (a well-formed resource
location) but not the referenced file's contents. For truly advanced
dimensions, author every worldgen resource by hand and reference the
top-level pieces from the typed API where it helps (dimension slot/type
selection, generator wiring) without fighting the API to express what it
isn't designed for.
