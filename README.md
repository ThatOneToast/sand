# Sand

Sand is a simple language for compiling Minecraft java datapacks. 
Currently, Sand is in an early stage of development, and is not ready for use.

Run `tree-sitter generate` in the `tree-sitter-sand` directory to generate the parser.

Currently only functions are supported.

## Preview

Sand.toml
```toml
name = "Testing-Sand"
version = "1.21.3"
description = "A test datapack"
```

main.sand
```sand
fn buff_players {
    effect @a[gamemode=survival] minecraft:strength 300 2;
    effect @a[gamemode=survival] minecraft:resistance 300 2;
    effect @a[gamemode=survival] minecraft:regeneration 300 1;
    effect @a[gamemode=survival] minecraft:speed 300 1;
    tellraw @a Buffs have been applied to all survival players;
}

fn start_boss_fight {
    // Announce boss fight
    tellraw @a Boss fight is starting!;

    // Buff the boss (assumed to be a named zombie)
    effect @e[type=minecraft:zombie,name="Boss"] minecraft:strength 600 3;
    effect @e[type=minecraft:zombie,name="Boss"] minecraft:resistance 600 2;
    effect @e[type=minecraft:zombie,name="Boss"] minecraft:speed 600 1;
    effect @e[type=minecraft:zombie,name="Boss"] minecraft:regeneration 600 1;

    // Give players some initial buffs
    effect @a[distance=..30] minecraft:resistance 60 1;
    effect @a[distance=..30] minecraft:strength 60 1;
}

fn night_vision_toggle {
    // Give night vision to players underground
    effect @a[y=..63] minecraft:night_vision 600 1;
    tellraw @a[y=..63] Night vision activated for underground players;
}

fn clear_nearby_mobs {
    // Make all hostile mobs nearby glow and levitate
    effect @e[type=minecraft:zombie,distance=..20] minecraft:glowing 30 1;
    effect @e[type=minecraft:skeleton,distance=..20] minecraft:glowing 30 1;
    effect @e[type=minecraft:creeper,distance=..20] minecraft:glowing 30 1;
    effect @e[type=minecraft:spider,distance=..20] minecraft:glowing 30 1;

    effect @e[type=minecraft:zombie,distance=..20] minecraft:levitation 10 1;
    effect @e[type=minecraft:skeleton,distance=..20] minecraft:levitation 10 1;
    effect @e[type=minecraft:creeper,distance=..20] minecraft:levitation 10 1;
    effect @e[type=minecraft:spider,distance=..20] minecraft:levitation 10 1;

    tellraw @a Nearby monsters are now glowing and floating away;
}

fn pvp_mode {
    // Setup for PVP matches
    time set noon;
    tellraw @a PVP match is starting!;

    // Give PVP effects
    effect @a[gamemode=survival] minecraft:resistance 30 1;
    effect @a[gamemode=survival] minecraft:strength 30 1;
    effect @a[gamemode=survival] minecraft:speed 30 1;
    effect @a[gamemode=survival] minecraft:regeneration 30 0;
}

fn raid_defense {
    // Buff players for raid defense
    effect @a[distance=..50] minecraft:strength 300 1;
    effect @a[distance=..50] minecraft:resistance 300 1;
    effect @a[distance=..50] minecraft:regeneration 300 0;

    // Make raiders glow
    effect @e[type=minecraft:pillager,distance=..100] minecraft:glowing 300 1;
    effect @e[type=minecraft:ravager,distance=..100] minecraft:glowing 300 1;
    effect @e[type=minecraft:vindicator,distance=..100] minecraft:glowing 300 1;

    tellraw @a Raid defense buffs active! All raiders within 100 blocks will glow;
}

fn underwater_exploration {
    effect @a[distance=..30] minecraft:water_breathing 600 1;
    effect @a[distance=..30] minecraft:night_vision 600 1;
    effect @a[distance=..30] minecraft:dolphins_grace 600 1;
    tellraw @a[distance=..30] Ocean exploration buffs applied!;
}

fn parkour_challenge {
    effect @p minecraft:jump_boost 300 1;
    effect @p minecraft:speed 300 1;
    effect @p minecraft:slow_falling 300 0;
    tellraw @a The parkour challenge has begun!;
}

fn boss_summoning_ritual {
    time set midnight;
    tellraw @a The ritual has begun...;

    effect @e[type=minecraft:zombie,name="Ritual Zombie"] minecraft:strength 600 4;
    effect @e[type=minecraft:zombie,name="Ritual Zombie"] minecraft:resistance 600 2;
    effect @e[type=minecraft:zombie,name="Ritual Zombie"] minecraft:speed 600 2;
    effect @e[type=minecraft:zombie,name="Ritual Zombie"] minecraft:regeneration 600 2;
    effect @e[type=minecraft:zombie,name="Ritual Zombie"] minecraft:fire_resistance 600 1;

    // Give nearby players resistance to give them a fighting chance
    effect @a[distance=..50] minecraft:resistance 60 2;
    tellraw @a The ritual is complete. Good luck...;
}
```