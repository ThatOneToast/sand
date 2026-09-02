# 29. Void Worlds

🌍 World (datapack).

`Generator::Void` (or the equivalent `Generator::void()`) produces the
fastest possible generation: no terrain at all, just the vanilla
`minecraft:the_void` biome.

```rust
use sand::build::Generator;

let generator = Generator::Void;
// equivalently:
let generator = Generator::void();
```

Internally this still lowers to a `minecraft:flat` chunk generator (that's
how vanilla represents "the void" too), just with an empty layer list and
the `minecraft:the_void` biome:

```json
{
  "type": "minecraft:flat",
  "settings": {
    "biome": "minecraft:the_void",
    "lakes": false,
    "features": false,
    "layers": [],
    "structure_overrides": []
  }
}
```

## Pairing void worlds with a spawn platform

A void world has nothing to stand on, so pair it with
[`Spawn::platform`](./27-worlds-dimensions-and-generators.md) so players
don't fall through on join:

```rust
use sand::build::{Spawn, World};
use sand::ResourceLocation;

let world = World::new().spawn(
    Spawn::at(0, 100, 0)
        .platform(ResourceLocation::new("minecraft", "stone").unwrap(), 5),
);
```

This lowers to `setworldspawn 0 100 0 0 0` followed by a `fill` command
placing a 11×11 stone platform one block below spawn — both part of the
generated `__sand_world_init` function, so they run automatically when the
datapack loads.

## When to reach for void vs. flat

Void is the right choice when the world's content is entirely
player-placed or driven by your datapack's own structures/schematics — for
example, a minigame arena. If players need any ambient terrain to look at
or walk on without your intervention, [Flat Worlds](./28-flat-worlds.md) is
usually a better fit.
