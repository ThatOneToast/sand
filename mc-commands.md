# Minecraft Commands Reference

## Player & Entity Management

### Player Commands
- `/gamemode <mode>` - Change game mode
- `/tp` or `/teleport` - Teleport entities
- `/kill` - Remove entities
- `/effect` - Apply/remove status effects
- `/clear` - Remove items from inventory
- `/give` - Give items to players
- `/xp` - Modify experience
- `/spawnpoint` - Set player spawn point
- `/spectate` - Make one player spectate another
- `/attribute` - Modify entity attributes

### Entity Manipulation
- `/summon` - Create entities
- `/data` - Modify entity NBT data
- `/team` - Manage teams
- `/ride` - Make entities ride others
- `/damage` - Deal damage to entities

## World Manipulation

### Block Commands
- `/setblock` - Place a single block
- `/fill` - Place multiple blocks
- `/clone` - Copy blocks from one area to another
- `/forceload` - Keep chunks loaded

### Structure Commands
- `/place` - Place features/structures
- `/structure` - Save/load structures
- `/jigsaw` - Manipulate jigsaw blocks

### World Settings
- `/gamerule` - Change game rules
- `/difficulty` - Set difficulty
- `/time` - Change time
- `/weather` - Change weather
- `/worldborder` - Manage world border

## Scoreboard System

### Basic Scoreboard
- `/scoreboard objectives add` - Create objective
- `/scoreboard objectives remove` - Remove objective
- `/scoreboard objectives setdisplay` - Set display location
- `/scoreboard players` - Modify scores
  - `set` - Set exact value
  - `add` - Add to value
  - `remove` - Subtract from value
  - `reset` - Remove scores
  - `operation` - Perform operations between scores

### Scoreboard Criteria Types
```
minecraft.broken:*          # Items broken
minecraft.crafted:*         # Items crafted
minecraft.custom:*          # Custom statistics
minecraft.dropped:*         # Items dropped
minecraft.killed:*          # Entities killed
minecraft.killed_by:*       # Killed by entities
minecraft.mined:*          # Blocks mined
minecraft.picked_up:*       # Items picked up
minecraft.used:*           # Items used
dummy                      # Manual scoring
deathCount                 # Times died
health                     # Health points
```

## Execute Command
The `execute` command is extremely powerful and can chain multiple conditions and actions.

### Execute Subcommands
- `as` - Run as entities
- `at` - Run at entities' locations
- `positioned` - Run at position
- `rotated` - Run with rotation
- `facing` - Run facing direction
- `anchored` - Set execution anchor point
- `align` - Align to block positions
- `in` - Run in dimension
- `store` - Store command results
- `if`/`unless` - Conditional execution

### Execute Conditions
```
block          # Check block type
blocks         # Compare block areas
data           # Check NBT data
entity         # Check entity existence
predicate      # Check predicate
score          # Compare scores
```

## Data Management

### NBT Paths
- `{}` - Compound
- `[]` - List
- `.` - Child element
- `{}` - Match specific NBT

### Data Operations
- `get` - Read data
- `merge` - Combine data
- `modify` - Modify data
  - `append`
  - `insert`
  - `merge`
  - `prepend`
  - `set`

## Function Tags
Function tags allow running multiple functions together:
```json
{
  "values": [
    "namespace:function1",
    "namespace:function2",
    "#namespace:function_group"
  ]
}
```

## Advancement System
Advancements can trigger functions:
```json
{
  "criteria": {
    "requirement": {
      "trigger": "minecraft:event_name",
      "conditions": {}
    }
  },
  "rewards": {
    "function": "namespace:function_name"
  }
}
```

## Predicates
Predicates define complex conditions:
```json
{
  "condition": "minecraft:entity_properties",
  "entity": "this",
  "predicate": {
    "location": {
      "position": {
        "y": {
          "min": 60,
          "max": 100
        }
      }
    }
  }
}
```

## Loot Tables
Control drops and rewards:
```json
{
  "type": "minecraft:generic",
  "pools": [
    {
      "rolls": 1,
      "entries": [
        {
          "type": "minecraft:item",
          "name": "minecraft:diamond"
        }
      ],
      "conditions": [
        {
          "condition": "minecraft:random_chance",
          "chance": 0.5
        }
      ]
    }
  ]
}
```

## Item Modification
Modify items through functions:
```mcfunction
# Give custom item
give @p diamond_sword{display:{Name:'{"text":"Special Sword"}'},Enchantments:[{id:"sharpness",lvl:5}]}

# Modify held item
item modify entity @p weapon.mainhand namespace:modification
```

## Particle Commands
Create visual effects:
```mcfunction
particle minecraft:flame ~ ~ ~ 0 0 0 0.05 100
particle minecraft:dust 1.0 0.0 0.0 1.0 ~ ~ ~ 0 0 0 1 10
```

## Sound Commands
Play sounds:
```mcfunction
playsound minecraft:entity.experience_orb.pickup player @a ~ ~ ~ 1 1
playsound minecraft:block.note_block.pling player @p ~ ~ ~ 1 2
```

## Bossbar
Create and manage bossbars:
```mcfunction
bossbar create namespace:bar "Title"
bossbar set namespace:bar max 100
bossbar set namespace:bar value 50
bossbar set namespace:bar visible true
```

## Performance Considerations

### Best Practices
1. Use tags for entity selection instead of NBT checks
2. Limit the use of @e selectors
3. Use predicates instead of complex execute conditions
4. Minimize commands run per tick
5. Use efficient scoreboard operations
6. Limit area-effect commands (like /fill)

### Common Optimizations
```mcfunction
# Bad (checks all entities)
execute as @e[type=zombie] run ...

# Better (uses tag)
execute as @e[type=zombie,tag=special] run ...

# Bad (constant NBT check)
execute if entity @e[nbt={Item:{id:"minecraft:diamond"}}] run ...

# Better (use tags when item is spawned)
execute if entity @e[type=item,tag=diamond_item] run ...
```

## Examples of Complex Commands

### Raycast System
```mcfunction
# Start raycast
execute anchored eyes positioned ^ ^ ^ run function namespace:raycast

# Raycast function
execute unless block ~ ~ ~ air run function namespace:hit
execute if block ~ ~ ~ air positioned ^ ^ ^0.5 if entity @s[distance=..5] run function namespace:raycast
```

### Area Effect System
```mcfunction
# Create effect area
execute at @e[type=marker,tag=effect_source] run particle minecraft:end_rod ~ ~ ~ 2 2 2 0 10
execute at @e[type=marker,tag=effect_source] as @e[distance=..5] run effect give @s levitation 1 0
```

### Timer System
```mcfunction
# Initialize
scoreboard objectives add timer dummy

# Increment
scoreboard players add #time timer 1

# Reset after 20 ticks (1 second)
execute if score #time timer matches 20.. run scoreboard players set #time timer 0
```

### Custom Crafting
```mcfunction
# Check crafting table with specific items
execute as @e[type=item,nbt={Item:{id:"minecraft:diamond"}}] at @s if block ~ ~-1 ~ crafting_table if entity @e[type=item,distance=..1,nbt={Item:{id:"minecraft:stick"}}] run function namespace:craft_special
```