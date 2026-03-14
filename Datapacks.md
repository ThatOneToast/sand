# Minecraft 1.21.11 Datapacks — Full Breakdown

This document explains how datapacks work in **Minecraft Java Edition 1.21.11**, including directory structure, file placement, tags, functions, predicates, loot tables, advancements, and common examples.

---

# 1. What is a Datapack?

A **datapack** is a folder or `.zip` file placed inside a world's `datapacks` directory that modifies or adds data-driven content to Minecraft without mods.

Datapacks can modify:

- Recipes
- Loot tables
- Advancements
- Structures
- Functions (command scripts)
- World generation
- Item modifiers
- Predicates
- Damage types
- Chat types
- Trim materials/patterns
- Enchantments
- Tags
- Dimension settings
- Biomes
- Banner patterns
- Paintings
- Wolf variants
- Armor trims
- Instrument definitions
- Jukebox songs
- etc.

They rely entirely on **JSON files and `.mcfunction` scripts**.

---

# 2. Datapack Folder Location

Datapacks are placed inside a world save.

```
.minecraft/
└── saves/
    └── MyWorld/
        └── datapacks/
            └── my_datapack/
```

You can also use a `.zip` instead of a folder.

```
datapacks/
└── my_datapack.zip
```

---

# 3. Required Root Files

Every datapack requires a **pack.mcmeta** file.

```
my_datapack/
│
├── pack.mcmeta
└── data/
```

---

# 4. pack.mcmeta Format

Example:

```json
{
  "pack": {
    "pack_format": 48,
    "description": "My Datapack"
  }
}
```

### pack_format (Minecraft 1.21.11)

```
pack_format: 48
```

If incorrect, the datapack loads with a warning or fails.

---

# 5. Full Datapack Directory Structure

```
my_datapack/
│
├── pack.mcmeta
├── pack.png (optional)
│
└── data/
    │
    ├── minecraft/
    │   └── tags/
    │       └── functions/
    │           ├── load.json
    │           └── tick.json
    │
    └── my_namespace/
        │
        ├── functions/
        │   └── hello.mcfunction
        │
        ├── advancements/
        ├── loot_tables/
        ├── recipes/
        ├── predicates/
        ├── item_modifiers/
        ├── structures/
        ├── tags/
        │   ├── blocks/
        │   ├── items/
        │   ├── entity_types/
        │   └── functions/
        │
        ├── worldgen/
        │   ├── biome/
        │   ├── configured_feature/
        │   ├── placed_feature/
        │   ├── structure/
        │   ├── structure_set/
        │   ├── template_pool/
        │   ├── processor_list/
        │   └── noise_settings/
        │
        ├── damage_type/
        ├── enchantment/
        ├── painting_variant/
        ├── instrument/
        ├── trim_material/
        ├── trim_pattern/
        ├── banner_pattern/
        ├── jukebox_song/
        ├── chat_type/
        ├── wolf_variant/
        ├── dimension/
        ├── dimension_type/
        └── biome/
```

---

# 6. Namespaces

Every datapack uses a **namespace**.

```
data/<namespace>/
```

Example:

```
data/my_datapack/
```

Minecraft's default namespace is:

```
data/minecraft/
```

You **only override vanilla behavior** when using the `minecraft` namespace.

Example:

```
data/minecraft/loot_tables/entities/zombie.json
```

This overrides the zombie loot table.

---

# 7. Functions (.mcfunction)

Functions are **lists of commands executed sequentially**.

```
data/my_namespace/functions/example.mcfunction
```

Example:

```mcfunction
say Hello World
give @p diamond
```

Run with:

```
/function my_namespace:example
```

---

# 8. Tick and Load Functions

Minecraft allows automatic execution.

### load.json

Runs once when datapack loads.

```
data/minecraft/tags/functions/load.json
```

Example:

```json
{
  "values": [
    "my_namespace:init"
  ]
}
```

---

### tick.json

Runs every game tick (20 times per second).

```
data/minecraft/tags/functions/tick.json
```

Example:

```json
{
  "values": [
    "my_namespace:tick"
  ]
}
```

---

# 9. Recipes

Recipes modify crafting.

Location:

```
data/<namespace>/recipes/
```

Example:

```json
{
  "type": "minecraft:crafting_shaped",
  "pattern": [
    "DDD",
    "D D",
    "DDD"
  ],
  "key": {
    "D": {
      "item": "minecraft:diamond"
    }
  },
  "result": {
    "item": "minecraft:diamond_block"
  }
}
```

---

# 10. Loot Tables

Loot tables control drops.

Location:

```
data/<namespace>/loot_tables/
```

Example:

```json
{
  "pools": [
    {
      "rolls": 1,
      "entries": [
        {
          "type": "minecraft:item",
          "name": "minecraft:diamond"
        }
      ]
    }
  ]
}
```

---

# 11. Advancements

Location:

```
data/<namespace>/advancements/
```

Example:

```json
{
  "criteria": {
    "tick": {
      "trigger": "minecraft:tick"
    }
  },
  "rewards": {
    "function": "my_namespace:reward"
  }
}
```

---

# 12. Predicates

Predicates allow **conditional checks**.

Location:

```
data/<namespace>/predicates/
```

Example:

```json
{
  "condition": "minecraft:entity_properties",
  "entity": "this",
  "predicate": {
    "flags": {
      "is_on_fire": true
    }
  }
}
```

Used in commands like:

```
execute if predicate my_namespace:on_fire run say Burning
```

---

# 13. Item Modifiers

Modify items dynamically.

Location:

```
data/<namespace>/item_modifiers/
```

Example:

```json
{
  "function": "minecraft:set_name",
  "name": {
    "text": "Legendary Sword",
    "color": "gold"
  }
}
```

---

# 14. Tags

Tags group objects.

Location examples:

```
tags/items/
tags/blocks/
tags/entity_types/
tags/functions/
```

Example item tag:

```
data/my_namespace/tags/items/gems.json
```

```json
{
  "values": [
    "minecraft:diamond",
    "minecraft:emerald"
  ]
}
```

---

# 15. World Generation

Location:

```
data/<namespace>/worldgen/
```

Common folders:

```
worldgen/
├── biome/
├── configured_feature/
├── placed_feature/
├── structure/
├── structure_set/
├── template_pool/
├── processor_list/
└── noise_settings/
```

These define custom terrain and structures.

---

# 16. Dimensions

Location:

```
data/<namespace>/dimension/
data/<namespace>/dimension_type/
```

Example:

```
dimension/my_dimension.json
```

Defines portals and worldgen settings.

---

# 17. Structures

Location:

```
data/<namespace>/structures/
```

Structures are `.nbt` files exported with:

```
/structure save
```

---

# 18. Damage Types

Custom damage systems.

Location:

```
data/<namespace>/damage_type/
```

Example:

```json
{
  "message_id": "laser",
  "exhaustion": 0.1,
  "scaling": "when_caused_by_living_non_player"
}
```

---

# 19. Chat Types

Location:

```
data/<namespace>/chat_type/
```

Controls chat formatting.

---

# 20. Enchantments (1.21+ data-driven)

Location:

```
data/<namespace>/enchantment/
```

Allows creation of **custom enchantments**.

---

# 21. Armor Trims

```
trim_material/
trim_pattern/
```

Allows custom armor trims.

---

# 22. Instruments

For goat horns.

```
data/<namespace>/instrument/
```

---

# 23. Jukebox Songs

Custom music discs.

```
data/<namespace>/jukebox_song/
```

---

# 24. Wolf Variants

Custom wolf skins.

```
data/<namespace>/wolf_variant/
```

---

# 25. Banner Patterns

```
data/<namespace>/banner_pattern/
```

---

# 26. Paintings

```
data/<namespace>/painting_variant/
```

---

# 27. Datapack Priority

Datapacks load **top → bottom**.

Use command:

```
/datapack list
```

Enable:

```
/datapack enable
```

---

# 28. Reloading Datapacks

Reload without restarting world:

```
/reload
```

---

# 29. Overriding Vanilla Data

To override vanilla content:

```
data/minecraft/<system>/
```

Example:

```
data/minecraft/recipes/diamond_sword.json
```

---

# 30. Function Calling Examples

Call function:

```
/function my_namespace:start
```

Run conditionally:

```
execute if entity @p run function my_namespace:test
```

---

# 31. Example Minimal Datapack

```
example_pack/
│
├── pack.mcmeta
└── data/
    ├── minecraft/
    │   └── tags/functions/
    │       └── load.json
    │
    └── example/
        └── functions/
            └── hello.mcfunction
```

pack.mcmeta

```json
{
  "pack": {
    "pack_format": 48,
    "description": "Example Datapack"
  }
}
```

load.json

```json
{
  "values": [
    "example:hello"
  ]
}
```

hello.mcfunction

```
say Datapack Loaded!
```

---

# 32. Debugging

Useful commands:

```
/reload
/datapack list
/function
/log
```

Check logs at:

```
.minecraft/logs/latest.log
```

---

# 33. Performance Tips

Avoid:

```
tick functions with heavy loops
mass entity scans
repeated scoreboard operations
```

Use:

```
scoreboards
predicates
selectors
storage
```

---

# 34. Useful Systems Often Used In Datapacks

Common systems:

```
scoreboards
storage
bossbars
teams
markers
area_effect_cloud entities
```

Example scoreboard:

```
scoreboard objectives add timer dummy
```

---

# 35. Storage System

Persistent NBT storage.

```
data modify storage my_namespace:data value 5
```

Location reference:

```
storage my_namespace:data
```

---

# Minecraft 1.21.11 Datapack Feature Summary

## Major Capabilities

```
Functions (scripted commands)
Custom recipes
Custom loot tables
Custom advancements
Custom structures
Custom world generation
Custom enchantments
Custom armor trims
Custom paintings
Custom wolf variants
Custom banner patterns
Custom goat horn instruments
Custom jukebox songs
Custom chat types
Custom dimensions
Custom damage types
Custom tags
Custom predicates
Item modifiers
Structure processors
Biome definitions
Dimension settings
```

---

## Core File Types

```
.mcfunction
.json
.nbt (structures)
```

---

## Execution Systems

```
tick functions
load functions
advancement triggers
predicate checks
execute command logic
```

---

## Major Datapack Systems

```
Scoreboards
Storage
Tags
Predicates
Loot tables
Recipes
Functions
Worldgen
Structures
Advancements
```

---

## Performance Model

```
Datapacks are interpreted each tick.
Most logic runs through functions.
Optimization requires minimizing tick logic and entity scans.
```

---

## Reload Behavior

```
/reload reinitializes datapack data
load.json runs again
tick.json continues running every tick
```

---

If you'd like, I can also provide:

- **A complete 1.21 datapack template used by large projects**
- **Advanced datapack architecture (modules, libraries, APIs)**
- **A full guide to datapack performance optimization**
- **How massive datapacks like VanillaTweaks are structured**
- **How to build a datapack game engine (RPG systems, abilities, etc.)**
-
