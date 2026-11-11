# The Sand Language

Sand packs can be written in Rust or in the **Sand language**: `.sand` files
with Rust's syntax that compile straight to a datapack. Both are entry points
into the same compiler. A `.sand` package lowers to Sand's portable program
model and is validated and emitted by the same backend as Rust and JSON
frontends, so equivalent packs produce byte-identical datapacks.

The difference is *when* code runs. In Rust, an `if` runs while the pack is
being built. In Sand source, every `if`, `for`, and assignment is gameplay that
runs inside Minecraft. You don't need macros or a Cargo build, and you don't
have to think in two stages.

```rust,ignore
//! Coins for everyone.
#![pack(namespace = "coins", minecraft = "26.2")]

player state Wallet {
    #[clamp(0..=9999)]
    coins: number,
}

#[on(first_join)]
fn welcome() {
    chat!("{gold}Welcome, {player}!{reset} You have {Wallet.coins} coins.");
    give_item(Item::Bread, 4);
}

#[on(death)]
fn tax() {
    Wallet.coins = Wallet.coins * 9 / 10;
    actionbar!("{red}You dropped some coins.");
}
```

## Getting started

```sh
sand lang new my-pack      # create my-pack/ with pack.sand and a module
cd my-pack
sand lang check            # report errors and warnings
sand lang build            # write the datapack to dist/<namespace>
sand lang run              # compile and launch a local Minecraft server
sand lang emit             # print the portable program JSON
```

`sand lang run` downloads the matching server, installs the pack in a test
world under `dist/server/`, and starts it (add `--offline` for easier local
joining). Language projects have no `sand.toml`; `sand run` also works from a
folder containing `pack.sand`. While the server runs, `sand lang build` updates
the test world, so `/reload` picks up your edits.

To use a pack elsewhere, copy `dist/<namespace>` into a world's `datapacks`
folder and run `/reload`.
A larger example lives in
[`examples/lang_encounters`](../../examples/lang_encounters/pack.sand).

## Editor setup (Zed)

The language server gives you diagnostics as you type, hover explanations for
every item, builtin, attribute, and keyword, plus completion, go to definition,
references, rename, an outline, and inlay hints showing where each function
runs.

1. Install the `sand` CLI so editors can start `sand lsp`:
   `cargo install --path sand-cli` from the Sand repository.
2. Run `sand lang zed`. It writes a Zed extension and a tree-sitter grammar
   for highlighting to `~/.sand/zed` (or `--dir <path>`).
3. In Zed, run **`zed: install dev extension`** from the command palette and
   choose the printed `zed-sand` folder. Zed compiles it with Rust (rustup).

Open any `.sand` file to start the server. Sand finds the package by walking
up from the file to the nearest `pack.sand`. If `sand` is not on Zed's `PATH`,
point Zed at it in `settings.json`:

```json
{ "lsp": { "sand": { "binary": { "path": "/path/to/sand", "arguments": ["lsp"] } } } }
```

After upgrading Sand, run `sand lang zed` again and rebuild the extension from
Zed's extensions page. Other editors can use any LSP client: run `sand lsp` as
a stdio server for `*.sand` files.

## Naming

Sand follows one naming convention, the same one Rust uses. The compiler
warns when a name breaks it and suggests the fixed spelling.

| Style | Used for | Examples |
| --- | --- | --- |
| `PascalCase` | things you declare and refer to: `state`, `enum` and its variants, `flow`, `item`, `recipe`, `tag`; typed value sets and their values | `Wallet`, `Phase::Idle`, `RubySword`, `Item::Diamond`, `Rarity::Epic` |
| `snake_case` | things that run or hold values: functions, builtins, fields, variables, parameters, modules | `buy_heart`, `give_item`, `Wallet.coins`, `let price` |
| `SCREAMING_SNAKE_CASE` | constants | `MAX_HEALTH` |

Builtins are named `verb` or `verb_object`: `give_item`, `clear_item`,
`add_effect`, `play_sound`, `spawn_particle`, `add_tag`, `get_attribute`,
`set_attribute`, `count_entities`. Value sets are singular nouns: `Item`,
`Block`, `Entity`, `Sound`, `Particle`, `Attribute`, `Effect`, `Potion`,
`Rarity`, `Sort`. Attribute words such as `#[tick]` and `#[on(death)]` are
lowercase, as in Rust.

Resource identities are always snake_case: `item RubySword` in the root
module becomes `<namespace>:ruby_sword`.

## Packages and modules

A package starts at a root file, conventionally `pack.sand`, which declares the
pack with an inner attribute:

```rust,ignore
#![pack(namespace = "arena", minecraft = "26.2", description = "Arena minigame")]
```

`namespace` is required. `minecraft` defaults to the newest supported target,
and `description` defaults to the root file's `//!` docs.

Modules follow Rust's layout. `mod combat;` in the root loads `combat.sand` or
`combat/mod.sand`, and `mod hits;` inside `combat.sand` loads
`combat/hits.sand`. Inline `mod name { ... }` blocks work too. Paths and
privacy follow Rust: `use crate::Hero;`, `use super::util::{a, b as c};`, and
items are private to their module (and its children) unless marked `pub`.

Every generated identity derives from the namespace, module path, and item
name: `fn grant` in `rewards.sand` becomes `arena:rewards/grant`, and
`player state Hero` becomes the State `arena:hero`.

## Declarations

| Item | Meaning |
| --- | --- |
| `player state Name { field: number = 0, ... }` | One value per player |
| `global state Name { ... }` | One shared value for the server |
| `enum Phase { Idle, Active = 5 }` | Named integer phases, usable as field types |
| `const NAME: number = expr;` | Compile-time constant (`+ - * / %`, `as number`) |
| `fn name() { ... }` | A Minecraft function |
| `extern "ns:path" fn name();` | A function implemented by another pack |
| `flow Name on State.field { ... }` | A state machine over a field |
| `mod name;` / `use path;` | Modules and imports |
| `item Name { ... }` | A custom item |
| `recipe Name { ... }` | A crafting, cooking, or stonecutting recipe |
| `tag Name: Item { ... }` | A group of items, blocks (`Block`), or entities (`Entity`) |

Fields default to `0`, or to the first variant for enum fields.
`#[clamp(min..=max)]` clamps every write to an `number` field.

Function attributes:

- `#[tick]` / `#[load]` run the function every tick or on load, on the server.
- `#[on(event)]` runs the function as the player when a player event happens:
  `join` (every join, including the first), `first_join`, `death`, `kill`
  (any entity, or `kill = Entity::Zombie`), `kill_player`,
  `killed_by = Entity::Creeper`, `use_item = Item::CarrotOnAStick`,
  `craft_item = Item::Bread`, `pick_up_item = Item::Diamond`,
  `mine_block = Block::DiamondOre`, `jump`, `drop_item`, `sleep`,
  `take_damage`, and `deal_damage`.
- `#[tag("ns:name")]` adds the function to any function tag.
- `#[player]` requires a player executor explicitly. Without it, the context is
  inferred (see below).

## Types

| Type | Holds | Stored as |
| --- | --- | --- |
| `number` | whole numbers | a scoreboard score |
| `float` | decimals with three places, like `1.5` | a score holding `value * 1000` |
| `bool` | `true` or `false` | a score holding one or zero |
| an `enum` | one of its named phases | a score holding the phase's number |

Scoreboards only hold whole numbers, so `float` is fixed point, the same
scheme as Rust's `Fixed` fields. Sand rescales arithmetic for you. Its range
is about +/-2,147,483, and multiplying two large fields can overflow; constant
factors are reduced first to make that less likely.

`number` and `float` do not mix implicitly (a whole-number literal is fine in
a `float` calculation); convert with `as float` and `as number`. `as number`
also converts enum phases and rounds a `float` down.

```rust,ignore
player state Hero {
    #[clamp(0.0..=2.0)]
    speed: float = 0.5,
    coins: number,
    rich: bool,
}

fn update() {
    Hero.speed = Hero.speed * 1.5;
    Hero.rich = Hero.coins >= 50;
    if Hero.rich && !is_tired() { Hero.coins -= 1; }
}
```

A `bool` works directly in conditions, and any condition can be assigned to
one. Text can only show whole numbers, so copy a `float` first:
`let shown = Hero.speed as number;`, then write `{shown}`.

## Functions and variables

Functions take typed parameters and may return a value, just like Rust:

```rust,ignore
/// The price of one extra heart, rising with each purchase.
fn heart_price(owned: number) -> number {
    return 50 + owned * 25;
}

#[on(use_item = Item::CarrotOnAStick)]
fn buy_heart() {
    let health = get_attribute(Attribute::MaxHealth);   // 20 = ten hearts
    if health >= 40 {
        actionbar!("{gold}You already have the maximum health.");
        return;                                       // early return
    }
    let price = heart_price((health - 20) / 2);
    if Wallet.coins < price {
        chat!("{red}You need {price} coins.");
        return;
    }
    Wallet.coins -= price;
    set_attribute(Attribute::MaxHealth, health + 2);
}
```

- `let name = value;` declares a variable; `let mut` allows reassignment.
  Variables are block-scoped and can shadow outer ones.
- Parameters are immutable (copy one into `let mut` to change it).
- `return;` ends a function early; `return value;` returns from a function
  declared with `-> number`. Every path must return a value in such functions.
- Functions cannot call themselves (see "How Sand compiles" for why); use a
  `while` loop instead.
- Entry points (`#[tick]`, `#[load]`, `#[on(...)]`, `schedule`, flow hooks)
  run without arguments, so they cannot take parameters.

## Statements

```rust,ignore
Hero.energy = 10;                       // set a State field
Hero.energy += 5;                       // also -=, *=, /=, %=
let total = Hero.gold * 3 + bonus(2);   // any integer expression
Dice.roll = random(1..=6);              // random integer
grant();                                // call (module paths work: rewards::grant())
for player in players() { ... }         // run the body as each online player
while n > 0 { n -= 1; }                 // loop while a comparison holds
if Hero.energy >= 30 && Hero.gold > Rival.gold { ... } else { ... }
```

Arithmetic follows normal precedence with parentheses, using Minecraft's 32-bit
scoreboard maths (division rounds down). A clamped field is clamped once,
after the whole value is computed.

Conditions compare any integer expressions (`== != < <= > >=`) or test `in` a
range (`a..b`, `a..=b`, `a..`, `..=b`). Combine them with `&&`, `||`, `!`, and
parentheses. Every comparison is evaluated at most once, and exactly one branch
runs. `while` takes a single comparison of fields, variables, or constants.

Inside `for player in players()`, `continue;` skips to the next player. As in
Rust, the loop changes who runs the body but not where: `offset(..)`, sounds,
particles, and summons stay at the caller's position (the world spawn in
`#[tick]`). `#[on(...)]` handlers and `for name in entities(..)` bodies do run
positioned at their player or entity.

Conditions can also ask about the world and the executor:

| Condition | True when |
| --- | --- |
| `has_state(Mana)` | the executor has this `entity state` attached |
| `any_entities(type = .., tag = .., distance = ..)` | at least one entity matches |
| `block_is(Block::Water, offset(0, -1, 0))` | the position holds this block or `Block` tag |
| `holding(Item::Diamond)`, `holding(Wand, offhand = true)` | the executor holds the item |
| `is_sneaking()`, `is_sprinting()`, `is_swimming()`, `is_on_fire()` | the executor is doing it |
| `timer_running(Ability.dash)`, `timer_done(Ability.dash)` | a `timer` field has time left, or none |

## Attributes

`get_attribute(Attribute::MaxHealth)` reads the executing player's base value
as an integer (pass `scale = 1000` to keep three decimals).
`set_attribute(Attribute::MaxHealth, value)` sets it from a constant or any
integer expression; divide integer expressions with `scale`:
`set_attribute(Attribute::MovementSpeed, speed, scale = 1000)`.

## Text

Text macros take a string with `{...}` interpolation:

| Macro | Shows |
| --- | --- |
| `chat!("...")` | a chat message to the executing player |
| `broadcast!("...")` | a chat message to every player |
| `actionbar!("...")` | text above the executing player's hotbar |
| `title!("...")` / `subtitle!("...")` | a big title (set the subtitle first) |

Inside the text, `{variable}` or `{State.field}` shows a number and `{player}`
shows the player's name. Style names (`{gold}`, `{red}`, `{bold}`, `{italic}`,
`{underlined}`, all sixteen colours, and `{reset}`) style the text that follows.
Write `{{` and `}}` for literal braces.

```rust,ignore
title!("{yellow}{bold}Round {World.round}");
chat!("{gray}Coins: {gold}{Wallet.coins}");
```

## Vanilla values

Items, blocks, entity types, sounds, particles, attributes, status effects,
and potions are typed, exactly as in Rust:

```rust,ignore
give_item(Item::Diamond, 3);
add_effect(Effect::Speed, seconds = 10);
play_sound(Sound::EntityPlayerLevelup);
spawn_particle(Particle::Heart, count = 5);
set_attribute(Attribute::MaxHealth, 24.0);
```

| Sand | Rust |
| --- | --- |
| `Item::...`, `Block::...`, `Entity::...` | `sand::vanilla::Item` and friends |
| `Sound::...`, `Particle::...`, `Attribute::...` | `sand::vanilla::SoundEvent`, ... |
| `Effect::...`, `Potion::...` | `sand::registry::EffectId`, `PotionId` |

The editor lists every value after `Item::` (or straight away inside a
builtin's argument), hovers show the Minecraft identity, and misspellings
get a "did you mean". The values come from the same generated registries as
Rust, so they always match the target Minecraft version. A namespaced string
such as `"mymod:ruby"` still works for modded or custom content; writing a
vanilla identity as a string gets a warning pointing at the typed value.

## Builtins

| Builtin | Does |
| --- | --- |
| `play_sound(Sound::..., volume = 1.0, pitch = 1.0)` | play a sound to the player |
| `spawn_particle(Particle::..., count = 1, spread = 0.0, speed = 0.0)` | spawn particles above the executor |
| `add_effect(Effect::..., seconds = 10, level = 2, hide_particles = true)` | give a status effect (omit `seconds` for infinite) |
| `clear_effect()` / `clear_effect(Effect::...)` | remove effects |
| `give_item(Item::..., 3)` | give items |
| `clear_item(Item::..., 2)` / `clear_inventory()` | remove items |
| `schedule(function, seconds = 5)` | run a server function later (`ticks = n` also works) |
| `set_attribute(Attribute::..., value, scale = 1)` | set an attribute's base value |
| `get_attribute(Attribute::..., scale = 1)` | read an attribute's base value (an expression) |
| `raw!("command")` | run one command exactly as written (escape hatch) |

Hover any builtin in the editor to see its arguments and the Minecraft command
it becomes.

## Entities

`for name in entities(..)` runs its body as each matching entity, positioned
at it. Filters are named arguments:

```rust,ignore
#[tick]
fn sweep() {
    for player in players() {
        for zombie in entities(type = Entity::Zombie, distance = ..=8, limit = 3, sort = Sort::Nearest) {
            damage(4.0);
            add_tag("hit");
        }
    }
    World.zombies = count_entities(type = Entity::Zombie);
}
```

| Builtin | Does |
| --- | --- |
| `entities(type, tag, distance, limit, sort)` | `for` source: run as each match |
| `count_entities(...)` | how many entities match (an expression) |
| `kill()` | kill the executing entity |
| `add_tag("x")` / `remove_tag("x")` | add or remove a tag |
| `damage(amount)` | damage the executing entity, in half-hearts |
| `summon(Entity::..., tag = "x")` | summon at the executor's position |

An entity may not be a player, so player State and player builtins are errors
inside an `entities` loop. A function that uses `kill()`, `tag`, or `damage`
outside a loop **runs as an entity**, the third execution context: callable
from player functions and entity loops, but not from `#[tick]` directly.

### Entity State

`entity state` stores values on individual entities, using the same model as
Rust's `#[state(scope = entity)]`:

```rust,ignore
entity state Mana {
    #[clamp(0..=100)]
    current: number = 10,
}

#[tick]
fn tick() {
    for mob in entities(type = Entity::Witch) {
        if has_state(Mana) {
            Mana.current += 1;
        } else {
            attach_state(Mana);
        }
    }
    for carrier in entities(with = Mana) { damage(1.0); }
}
```

`attach_state` gives fields without a value their defaults, and keeps existing
values. `detach_state` clears them. A function that touches entity State
outside a loop runs as an entity. Flows only run over player or global State.

### Movement

`teleport(x, y, z)` moves the executor to constant coordinates,
`teleport_by(dx, dy, dz)` moves it relatively, and `teleport_to(type = ...)`
moves it to the nearest entity matching the same filters as `entities(..)`.

## Custom events

An event is a name your own functions can react to. Declare it, optionally with
a typed payload; mark handlers with `#[on(...)]`; run them all with `emit`:

```rust,ignore
event Payday(amount: number);

#[on(Payday)]
fn deposit(amount: number) { Wallet.coins += amount; }

#[on(Payday)]
fn celebrate(amount: number) { chat!("{gold}Paid!"); }

fn wages() { emit(Payday, 25); }
```

Handlers run in declaration order, as whoever emitted the event, and must take
exactly the event's parameters. Arguments are evaluated once. If a handler needs
a player, emitting the event needs one too, and the editor says so. Any module
can handle a `pub event`.

## Timers and cooldowns

A `timer` field counts down by one every tick until it reaches zero, using the
same State lifecycle as Rust's `Timer` and `Cooldown` fields:

```rust,ignore
player state Ability { dash: timer }

fn dash() {
    if timer_running(Ability.dash) {
        actionbar!("{red}{Ability.dash} ticks left");
        return;
    }
    start_timer(Ability.dash, seconds = 3);
    teleport_by(0.0, 0.5, 0.0);
}
```

`start_timer` takes `ticks` or `seconds`, and `stop_timer` ends one early. A
timer reads as the number of ticks left, so comparisons and `{Ability.dash}`
text work too.

## The world

Block positions are `pos(x, y, z)` in world coordinates or `offset(dx, dy, dz)`
relative to the executor. Both take constants.

```rust,ignore
set_block(Block::Stone, offset(0, -1, 0));
fill_blocks(Block::Air, offset(-2, 0, -2), offset(2, 3, 2));
set_time(6000);
set_weather(Weather::Rain, seconds = 60);
set_gamemode(GameMode::Adventure);
add_experience(points = 10);   // or levels = 2
```

## Boss bars

```rust,ignore
bossbar Raid {
    title: "{red}Raid Boss",
    color: BossBarColor::Purple,
    style: BossBarStyle::Notched10,
    max: 200,
}

fn begin() {
    show_bossbar_all(Raid);
    set_bossbar(Raid, Fight.boss_hp);
}
```

`show_bossbar`/`hide_bossbar` change one player and leave the others alone;
`show_bossbar_all`/`hide_bossbar_all` change everyone online. `set_bossbar` and
`set_bossbar_max` take any `number` expression.

## Items, recipes, and tags

Data resources are declarations with typed fields:

```rust,ignore
/// A sword forged from redstone.
item RubySword {
    base: Item::DiamondSword,
    name: "{red}{bold}Ruby Sword",
    lore: ["{gray}Forged in fire."],
    rarity: Rarity::Epic,
    unbreakable: true,
    enchantments: { "minecraft:sharpness": 5 },
}

tag Ores: Item { values: [Item::IronOre, "#minecraft:coal_ores"] }

recipe RubySwordRecipe {
    shaped: [" R ", " R ", " S "],
    key: { R: Item::RedstoneBlock, S: Item::Stick },
    result: RubySword,
}

recipe Nugget { smelting: Ores, result: Item::IronNugget, experience: 0.5 }
recipe RedDye { shapeless: [Item::Poppy, Item::Poppy], result: Item::RedDye, count: 2 }
```

- `item` fields: `base`, `name`, `lore`, `rarity`, `glint`, `unbreakable`,
  `max_stack`, `enchantments`, `custom_model_data`. `give_item(RubySword)` and
  `clear_item(RubySword)` use its identity, a `custom_data` marker, exactly
  like Rust's `CustomItem::custom_data`.
- `recipe` takes exactly one of `shaped` (with `key`), `shapeless`,
  `smelting`, `blasting`, `smoking`, `campfire`, or `stonecutting`, plus
  `result` and optionally `count`, `experience`, and `time`. Custom items can
  be results but not ingredients: vanilla matches ingredients only by item or
  tag.
- `tag Name: Item { values: [...] }` (or `Block`, `Entity`) takes `values`:
  typed values, other tags of the same type, or `"#ns:tag"` strings, and
  optionally `replace`.

These lower through the same typed builders as Rust (`ShapedRecipe`,
`CustomItem`, `TypedTag`, ...). The editor completes field names and
offers the right typed values for each field.

Smithing upgrades name their template, `base`, and `addition`:

```rust,ignore
recipe Upgrade {
    smithing: Item::NetheriteUpgradeSmithingTemplate,
    base: Item::DiamondAxe,
    addition: Item::NetheriteIngot,
    result: Item::NetheriteAxe,
}
```

### Loot tables

```rust,ignore
loot Crate {
    rolls: 2,
    items: [Item::Diamond, { item: Item::Emerald, weight: 4, count: 3 }],
}

fn open_crate() { give_loot(Crate); }   // or drop_loot(Crate) at the executor
```

## Flows

A flow drives a State field through phases. Inside a flow over an enum field,
variants can be written without their enum prefix.

```rust,ignore
enum Phase { Idle, Fighting, Resting }
player state Hero { phase: Phase, energy: number = 100 }

flow Encounter on Hero.phase {
    Idle -> Fighting if Hero.energy >= 30 priority 10;
    Fighting -> Resting if Hero.energy <= 10;

    enter Fighting => start_fight;       // a named function
    tick Fighting every 20 {             // or an inline body
        Hero.energy -= 1;
    }
    exit Resting { Hero.energy = 100; }
}
```

At most one transition fires per cycle (higher `priority` first, then
declaration order). A transition guard compares the field with constants.

## Execution context

Player State belongs to the executing player, so code that reads or writes it
needs one. Sand infers each function's context:

- a function that touches player State, uses a player builtin (`chat!`,
  `give_item`, `play_sound`, `{player}`, ...) outside a `for player in players()` loop,
  or calls a function that does, **runs as a player**;
- a function that acts on the executing entity (`kill()`, `tag`, `damage`)
  outside a loop, or calls one that does, **runs as an entity**;
- everything else **runs on the server** and can be called from anywhere.

Entry points without a player are `#[tick]`, `#[load]`, the
`minecraft:tick`/`minecraft:load` tags, scheduled functions, and global flow
hooks. If one needs a player, the error explains the chain that requires it:

```text
error[context]: `tick` runs from #[tick] without a player, but it calls `regen`, which needs a player
  --> pack.sand:5:4
  |
5 | fn tick() {
  |    ^^^^
  --> pack.sand:4:1
  |
4 | #[tick]
  | ^^^^^^^ runs on the server here
  --> pack.sand:6:5
  |
6 |     regen();
  |     ^^^^^ player function called here
  = help: move player work into `for player in players() { ... }`
```

Player event handlers (`#[on(...)]`) always run as the player.

## How Sand compiles

`sand lang build` runs four stages, all in one process with no Cargo build:

1. **Parse.** Each `.sand` file is parsed into a syntax tree with error
   recovery, so one mistake never hides the rest of the file.
2. **Analyse.** Names resolve through modules and `use`; types, mutability,
   arguments, and returns are checked; and every function's execution context
   (player or server) is inferred from what it touches. This is the same
   analysis the language server runs as you type.
3. **Lower.** The package becomes Sand's portable program: a JSON-shaped model
   of State, functions, and typed operations (`sand lang emit` prints it).
4. **Emit.** The same backend used by Rust and JSON frontends validates the
   program and writes the datapack.

Datapacks have no variables, call stack, or return values, so Sand maps each
language feature onto what Minecraft does have:

| Sand | Minecraft |
| --- | --- |
| `player state` / `global state` field | a scoreboard objective, held by `@s` or a fixed holder |
| `let` variable, parameter, temporary | a slot on one compiler-owned global holder |
| `fn` | a `.mcfunction` file |
| calling `f(a, b)` | copy arguments into `f`'s parameter slots, then `function ns:f` |
| `return value;` | write `f`'s return slot, then `return 1` |
| `return;` from inside an `if` | the branch helper returns 1; the caller runs `execute if function <helper> run return 1` |
| `if` / `else` | `execute if score ... run return run function <then>` in a helper |
| `while` | a helper that runs its body and tail-calls itself while the condition holds |
| `x * 3 + y` | `scoreboard players operation` steps through temporaries |
| `set_attribute(..., expr)` | the value is copied into storage and applied by a macro function |

Because variables are slots rather than stack frames, a function cannot be
re-entered while it is running, which is why recursion is rejected. Commands
run one after another within a tick, so players never see each other's
variables, even inside `for player in players()`.

## Parity with Rust

The language and Rust share one compiler backend, so anything both can
express produces identical output. Today the language covers:

| Area | Language | Rust-only for now |
| --- | --- | --- |
| State | player, global, and entity State; `number`, `float`, `bool`, enum, and `timer` fields; clamps; flows | NBT-backed values, relations, derived values, migrations |
| Logic | functions with parameters and returns, `let`, `if`, `while`, early return, custom events | closures, generics, compile-time Rust code |
| Players and entities | player loops, entity queries, kill/tag/damage/summon, teleport, attributes, effects, items, experience, game mode, sneaking and other flags, held items | selection by NBT or scores, motion, vehicles, equipment edits, archetypes |
| Events | join, first join, death, kills (by type), killed by, item use, crafting, pickups, mining, jumping, dropping, sleeping, damage taken/dealt, custom events with payloads | advancement triggers, equipment changes, tracked transitions, event chains |
| World | blocks and regions, block tests, time, weather | structures, biomes, dimensions, clone, data on blocks |
| Text and feedback | chat, broadcast, actionbar, titles, sounds, particles, boss bars | books, dialogs, clickable text, VFX sequences |
| Data | custom items, crafting/cooking/stonecutting/smithing upgrade recipes, item/block/entity tags, single-pool loot tables | conditional loot, advancements, predicates, trim recipes, worldgen, damage types |
| Gameplay systems | build them from the pieces above | combat, abilities, encounters, jobs, menus, HUD, inventory transfers, rosters, projectiles, progression, scenes, sessions, status effects, checkpoints |

`raw!("...")` covers single commands that have no builtin yet.

