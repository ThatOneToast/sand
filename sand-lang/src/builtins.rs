//! The built-in vocabulary of the Sand language, with its documentation.
//!
//! One table drives argument checking in analysis and the explanations shown
//! by hover and completion, so documentation cannot drift from behaviour.

/// How a builtin is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinKind {
    /// A statement call: `play_sound("...");`.
    Statement,
    /// A macro taking a format string: `chat!("...");`.
    Macro,
    /// An expression: `random(1..=6)`.
    Expression,
    /// A predicate in a condition: `if has_state(Mana) { ... }`.
    Condition,
}

/// What an argument accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    /// A typed vanilla registry value such as `Item::Diamond`, named by its
    /// registry. A namespaced string is accepted for modded or custom values.
    Registry(&'static str),
    /// A text format string with `{...}` interpolation.
    Format,
    /// A plain string.
    Text,
    /// An integer constant.
    Int,
    /// A decimal or integer constant.
    Number,
    /// `true` or `false`.
    Bool,
    /// A function name.
    Function,
    /// A constant range such as `1..=6`.
    Range,
    /// A declared `entity state`.
    EntityState,
    /// A `timer` State field, written `State.field`.
    Timer,
    /// A block position: `pos(x, y, z)` or `offset(dx, dy, dz)`.
    Position,
    /// A declared `bossbar`.
    BossBar,
    /// A declared `loot` table.
    Loot,
    /// A declared `event`.
    Event,
}

/// One builtin parameter.
#[derive(Debug, Clone, Copy)]
pub struct Param {
    /// Parameter name, usable as `name = value`.
    pub name: &'static str,
    /// Accepted value.
    pub kind: ParamKind,
    /// Whether the argument must be supplied.
    pub required: bool,
    /// Explanation shown in hover.
    pub doc: &'static str,
}

/// One builtin statement, macro, or expression.
#[derive(Debug, Clone, Copy)]
pub struct Builtin {
    /// Source name, without `!` for macros.
    pub name: &'static str,
    /// How it is written.
    pub kind: BuiltinKind,
    /// Signature shown in hover and completion.
    pub signature: &'static str,
    /// One-line summary.
    pub summary: &'static str,
    /// Longer explanation.
    pub details: &'static str,
    /// The Minecraft command it becomes.
    pub minecraft: &'static str,
    /// Whether it needs a player executor.
    pub needs_player: bool,
    /// Parameters in positional order.
    pub params: &'static [Param],
    /// Completion snippet.
    pub snippet: &'static str,
}

const fn p(name: &'static str, kind: ParamKind, required: bool, doc: &'static str) -> Param {
    Param {
        name,
        kind,
        required,
        doc,
    }
}

const FORMAT_DOC: &str = "Text with `{...}` interpolation: `{State.field}` shows a live score, `{player}` the player's name, and `{gold}`, `{bold}`, `{italic}`, `{underlined}`, `{reset}` change the style of the text that follows. Write `{{` and `}}` for literal braces.";

/// Filters shared by `entities(...)` and `count_entities(...)`.
const QUERY: &[Param] = &[
    p(
        "type",
        ParamKind::Registry("Entity"),
        false,
        "Only this entity type, such as `Entity::Zombie`.",
    ),
    p(
        "tag",
        ParamKind::Text,
        false,
        "Only entities with this tag.",
    ),
    p(
        "distance",
        ParamKind::Range,
        false,
        "Only entities this far from the executor, such as `..=16`.",
    ),
    p(
        "with",
        ParamKind::EntityState,
        false,
        "Only entities with this `entity state` attached.",
    ),
    p(
        "limit",
        ParamKind::Int,
        false,
        "At most this many entities.",
    ),
    p(
        "sort",
        ParamKind::Registry("Sort"),
        false,
        "Order before `limit`: `Sort::Nearest`, `Sort::Furthest`, `Sort::Random`, or `Sort::Arbitrary`.",
    ),
];

/// Builtins that act on the executing entity (`@s`), so they need a player or
/// an entity to run as.
pub const EXECUTOR_BUILTINS: &[&str] = &[
    "kill",
    "add_tag",
    "remove_tag",
    "damage",
    "attach_state",
    "detach_state",
    "teleport",
    "teleport_by",
    "teleport_to",
    "holding",
    "is_sneaking",
    "is_sprinting",
    "is_swimming",
    "is_on_fire",
];

/// Every builtin, in documentation order.
pub const BUILTINS: &[Builtin] = &[
    Builtin {
        name: "emit",
        kind: BuiltinKind::Statement,
        signature: "emit(Event, args...)",
        summary: "Run every handler of a custom event, in declaration order.",
        details: "Declare the event with `event Name(params);` and handlers with `#[on(Name)]`. Arguments follow the event name and are evaluated once. Handlers run as the emitter, so an event with a handler that needs a player must be emitted by a player.",
        minecraft: "function <ns>:sand_event/<name>",
        needs_player: false,
        params: &[p("event", ParamKind::Event, true, "The event to emit.")],
        snippet: "emit(${1:Event});",
    },
    Builtin {
        name: "start_timer",
        kind: BuiltinKind::Statement,
        signature: "start_timer(State.field, ticks = .. | seconds = ..)",
        summary: "Start a `timer` field counting down.",
        details: "Give exactly one of `ticks` (20 per second) or `seconds`. Starting a running timer restarts it. Timers are numbers of ticks left, so you can also read and compare them.",
        minecraft: "scoreboard players set <holder> <objective> <ticks>",
        needs_player: false,
        params: &[
            p("timer", ParamKind::Timer, true, "The `timer` field."),
            p("ticks", ParamKind::Int, false, "Duration in game ticks."),
            p("seconds", ParamKind::Int, false, "Duration in seconds."),
        ],
        snippet: "start_timer(${1:State.field}, seconds = ${2:5});",
    },
    Builtin {
        name: "stop_timer",
        kind: BuiltinKind::Statement,
        signature: "stop_timer(State.field)",
        summary: "Stop a `timer` field, so it reads as done.",
        details: "Sets the time left to zero.",
        minecraft: "scoreboard players set <holder> <objective> 0",
        needs_player: false,
        params: &[p("timer", ParamKind::Timer, true, "The `timer` field.")],
        snippet: "stop_timer(${1:State.field});",
    },
    Builtin {
        name: "timer_running",
        kind: BuiltinKind::Condition,
        signature: "timer_running(State.field)",
        summary: "Whether a `timer` field still has time left.",
        details: "Use it as a cooldown check: `if timer_running(Ability.dash) { return; }`.",
        minecraft: "execute if score <holder> <objective> matches 1..",
        needs_player: false,
        params: &[p("timer", ParamKind::Timer, true, "The `timer` field.")],
        snippet: "timer_running(${1:State.field})",
    },
    Builtin {
        name: "timer_done",
        kind: BuiltinKind::Condition,
        signature: "timer_done(State.field)",
        summary: "Whether a `timer` field has reached zero.",
        details: "True before the timer is first started.",
        minecraft: "execute if score <holder> <objective> matches 0",
        needs_player: false,
        params: &[p("timer", ParamKind::Timer, true, "The `timer` field.")],
        snippet: "timer_done(${1:State.field})",
    },
    Builtin {
        name: "any_entities",
        kind: BuiltinKind::Condition,
        signature: "any_entities(type = .., tag = .., distance = .., with = ..)",
        summary: "Whether at least one entity matches the filters.",
        details: "Takes the same filters as `entities(..)`. Distances are measured from the executor.",
        minecraft: "execute if entity @e[...]",
        needs_player: false,
        params: QUERY,
        snippet: "any_entities(type = Entity::${1:Zombie})",
    },
    Builtin {
        name: "block_is",
        kind: BuiltinKind::Condition,
        signature: "block_is(Block::..., position)",
        summary: "Whether a position holds a block, or a block of a `tag`.",
        details: "Write the position as `pos(x, y, z)` or `offset(dx, dy, dz)`: `if block_is(Block::Water, offset(0, -1, 0)) { ... }`.",
        minecraft: "execute if block <position> <block>",
        needs_player: false,
        params: &[
            p(
                "block",
                ParamKind::Registry("Block"),
                true,
                "Block, or a declared `Block` tag.",
            ),
            p("at", ParamKind::Position, true, "Position to test."),
        ],
        snippet: "block_is(Block::${1:Stone}, offset(${2:0}, ${3:-1}, ${4:0}))",
    },
    Builtin {
        name: "holding",
        kind: BuiltinKind::Condition,
        signature: "holding(Item::..., offhand = false)",
        summary: "Whether the executing entity holds an item.",
        details: "Custom items match by identity, so a renamed vanilla item does not count.",
        minecraft: "execute if items entity @s weapon.mainhand <item>",
        needs_player: false,
        params: &[
            p(
                "item",
                ParamKind::Registry("Item"),
                true,
                "Item or custom item.",
            ),
            p(
                "offhand",
                ParamKind::Bool,
                false,
                "Test the off hand instead.",
            ),
        ],
        snippet: "holding(Item::${1:Diamond})",
    },
    Builtin {
        name: "is_sneaking",
        kind: BuiltinKind::Condition,
        signature: "is_sneaking()",
        summary: "Whether the executing entity is crouching.",
        details: "Checked at the moment the condition runs.",
        minecraft: "execute if predicate <generated entity_properties predicate>",
        needs_player: false,
        params: &[],
        snippet: "is_sneaking()",
    },
    Builtin {
        name: "is_sprinting",
        kind: BuiltinKind::Condition,
        signature: "is_sprinting()",
        summary: "Whether the executing entity is sprinting.",
        details: "Checked at the moment the condition runs.",
        minecraft: "execute if predicate <generated entity_properties predicate>",
        needs_player: false,
        params: &[],
        snippet: "is_sprinting()",
    },
    Builtin {
        name: "is_swimming",
        kind: BuiltinKind::Condition,
        signature: "is_swimming()",
        summary: "Whether the executing entity is swimming.",
        details: "Checked at the moment the condition runs.",
        minecraft: "execute if predicate <generated entity_properties predicate>",
        needs_player: false,
        params: &[],
        snippet: "is_swimming()",
    },
    Builtin {
        name: "is_on_fire",
        kind: BuiltinKind::Condition,
        signature: "is_on_fire()",
        summary: "Whether the executing entity is burning.",
        details: "Checked at the moment the condition runs.",
        minecraft: "execute if predicate <generated entity_properties predicate>",
        needs_player: false,
        params: &[],
        snippet: "is_on_fire()",
    },
    Builtin {
        name: "set_block",
        kind: BuiltinKind::Statement,
        signature: "set_block(Block::..., position)",
        summary: "Place a block.",
        details: "Write the position as `pos(x, y, z)` for world coordinates or `offset(dx, dy, dz)` relative to the executor.",
        minecraft: "setblock <position> <block>",
        needs_player: false,
        params: &[
            p(
                "block",
                ParamKind::Registry("Block"),
                true,
                "Block to place.",
            ),
            p("at", ParamKind::Position, true, "Where to place it."),
        ],
        snippet: "set_block(Block::${1:Stone}, offset(${2:0}, ${3:0}, ${4:0}));",
    },
    Builtin {
        name: "fill_blocks",
        kind: BuiltinKind::Statement,
        signature: "fill_blocks(Block::..., from, to)",
        summary: "Fill a region with a block, including both corners.",
        details: "Corners are positions: `fill_blocks(Block::Air, offset(-2, 0, -2), offset(2, 3, 2));`. Minecraft limits one fill to 32,768 blocks by default.",
        minecraft: "fill <from> <to> <block>",
        needs_player: false,
        params: &[
            p(
                "block",
                ParamKind::Registry("Block"),
                true,
                "Block to fill with.",
            ),
            p("from", ParamKind::Position, true, "One corner."),
            p("to", ParamKind::Position, true, "Opposite corner."),
        ],
        snippet: "fill_blocks(Block::${1:Air}, offset(${2:0}, ${3:0}, ${4:0}), offset(${5:1}, ${6:1}, ${7:1}));",
    },
    Builtin {
        name: "add_experience",
        kind: BuiltinKind::Statement,
        signature: "add_experience(points = .. | levels = ..)",
        summary: "Give the executing player experience.",
        details: "Give exactly one of `points` or `levels`. Negative amounts remove experience.",
        minecraft: "xp add @s <amount> <points|levels>",
        needs_player: true,
        params: &[
            p("points", ParamKind::Int, false, "Experience points."),
            p("levels", ParamKind::Int, false, "Experience levels."),
        ],
        snippet: "add_experience(points = ${1:10});",
    },
    Builtin {
        name: "set_gamemode",
        kind: BuiltinKind::Statement,
        signature: "set_gamemode(GameMode::...)",
        summary: "Change the executing player's game mode.",
        details: "Values: `GameMode::Survival`, `GameMode::Creative`, `GameMode::Adventure`, `GameMode::Spectator`.",
        minecraft: "gamemode <mode> @s",
        needs_player: true,
        params: &[p(
            "mode",
            ParamKind::Registry("GameMode"),
            true,
            "New game mode.",
        )],
        snippet: "set_gamemode(GameMode::${1:Adventure});",
    },
    Builtin {
        name: "set_time",
        kind: BuiltinKind::Statement,
        signature: "set_time(ticks)",
        summary: "Set the time of day.",
        details: "0 is sunrise, 6000 noon, 12000 sunset, and 18000 midnight.",
        minecraft: "time set <ticks>",
        needs_player: false,
        params: &[p("ticks", ParamKind::Int, true, "Time of day in ticks.")],
        snippet: "set_time(${1:6000});",
    },
    Builtin {
        name: "set_weather",
        kind: BuiltinKind::Statement,
        signature: "set_weather(Weather::..., seconds = ..)",
        summary: "Change the weather.",
        details: "Omit `seconds` and Minecraft chooses how long it lasts.",
        minecraft: "weather <weather> [duration]",
        needs_player: false,
        params: &[
            p(
                "weather",
                ParamKind::Registry("Weather"),
                true,
                "New weather.",
            ),
            p("seconds", ParamKind::Int, false, "How long it lasts."),
        ],
        snippet: "set_weather(Weather::${1:Clear});",
    },
    Builtin {
        name: "show_bossbar",
        kind: BuiltinKind::Statement,
        signature: "show_bossbar(Bar)",
        summary: "Show a boss bar to the executing player.",
        details: "Other players keep whatever they currently see.",
        minecraft: "tag @s add <viewer tag>; bossbar set <id> players @a[tag=<viewer tag>]",
        needs_player: true,
        params: &[p("bar", ParamKind::BossBar, true, "The boss bar.")],
        snippet: "show_bossbar(${1:Bar});",
    },
    Builtin {
        name: "hide_bossbar",
        kind: BuiltinKind::Statement,
        signature: "hide_bossbar(Bar)",
        summary: "Hide a boss bar from the executing player.",
        details: "Other players keep whatever they currently see.",
        minecraft: "tag @s remove <viewer tag>; bossbar set <id> players @a[tag=<viewer tag>]",
        needs_player: true,
        params: &[p("bar", ParamKind::BossBar, true, "The boss bar.")],
        snippet: "hide_bossbar(${1:Bar});",
    },
    Builtin {
        name: "show_bossbar_all",
        kind: BuiltinKind::Statement,
        signature: "show_bossbar_all(Bar)",
        summary: "Show a boss bar to every online player.",
        details: "Players who join later do not see it until it is shown to them.",
        minecraft: "tag @a add <viewer tag>; bossbar set <id> players @a[tag=<viewer tag>]",
        needs_player: false,
        params: &[p("bar", ParamKind::BossBar, true, "The boss bar.")],
        snippet: "show_bossbar_all(${1:Bar});",
    },
    Builtin {
        name: "hide_bossbar_all",
        kind: BuiltinKind::Statement,
        signature: "hide_bossbar_all(Bar)",
        summary: "Hide a boss bar from every online player.",
        details: "Show it again with `show_bossbar` or `show_bossbar_all`.",
        minecraft: "tag @a remove <viewer tag>; bossbar set <id> players @a[tag=<viewer tag>]",
        needs_player: false,
        params: &[p("bar", ParamKind::BossBar, true, "The boss bar.")],
        snippet: "hide_bossbar_all(${1:Bar});",
    },
    Builtin {
        name: "set_bossbar",
        kind: BuiltinKind::Statement,
        signature: "set_bossbar(Bar, value)",
        summary: "Set how full a boss bar is.",
        details: "`value` is a `number` constant or expression, between zero and the bar's `max`.",
        minecraft: "execute store result bossbar <id> value run scoreboard players get ...",
        needs_player: false,
        params: &[
            p("bar", ParamKind::BossBar, true, "The boss bar."),
            p("value", ParamKind::Int, true, "New value."),
        ],
        snippet: "set_bossbar(${1:Bar}, ${2:100});",
    },
    Builtin {
        name: "set_bossbar_max",
        kind: BuiltinKind::Statement,
        signature: "set_bossbar_max(Bar, max)",
        summary: "Change a boss bar's largest value.",
        details: "Overrides the `max` from its declaration until the next `/reload`.",
        minecraft: "execute store result bossbar <id> max run scoreboard players get ...",
        needs_player: false,
        params: &[
            p("bar", ParamKind::BossBar, true, "The boss bar."),
            p("max", ParamKind::Int, true, "New maximum, at least 1."),
        ],
        snippet: "set_bossbar_max(${1:Bar}, ${2:100});",
    },
    Builtin {
        name: "give_loot",
        kind: BuiltinKind::Statement,
        signature: "give_loot(Table)",
        summary: "Give the executing player one roll of a loot table.",
        details: "Declare tables with `loot Name { items: [...] }`.",
        minecraft: "loot give @s loot <table>",
        needs_player: true,
        params: &[p("table", ParamKind::Loot, true, "The loot table.")],
        snippet: "give_loot(${1:Table});",
    },
    Builtin {
        name: "drop_loot",
        kind: BuiltinKind::Statement,
        signature: "drop_loot(Table)",
        summary: "Drop one roll of a loot table at the executor's position.",
        details: "Declare tables with `loot Name { items: [...] }`.",
        minecraft: "loot spawn ~ ~ ~ loot <table>",
        needs_player: false,
        params: &[p("table", ParamKind::Loot, true, "The loot table.")],
        snippet: "drop_loot(${1:Table});",
    },
    Builtin {
        name: "teleport",
        kind: BuiltinKind::Statement,
        signature: "teleport(x, y, z)",
        summary: "Move the executing entity to fixed coordinates.",
        details: "Coordinates are constants. Use `teleport_by` to move relative to where the entity is.",
        minecraft: "tp @s <x> <y> <z>",
        needs_player: false,
        params: &[
            p("x", ParamKind::Number, true, "X coordinate."),
            p("y", ParamKind::Number, true, "Y coordinate."),
            p("z", ParamKind::Number, true, "Z coordinate."),
        ],
        snippet: "teleport(${1:0.0}, ${2:64.0}, ${3:0.0});",
    },
    Builtin {
        name: "teleport_by",
        kind: BuiltinKind::Statement,
        signature: "teleport_by(dx, dy, dz)",
        summary: "Move the executing entity relative to where it is.",
        details: "`teleport_by(0.0, 5.0, 0.0)` lifts it five blocks.",
        minecraft: "tp @s ~<dx> ~<dy> ~<dz>",
        needs_player: false,
        params: &[
            p("dx", ParamKind::Number, true, "X offset."),
            p("dy", ParamKind::Number, true, "Y offset."),
            p("dz", ParamKind::Number, true, "Z offset."),
        ],
        snippet: "teleport_by(${1:0.0}, ${2:5.0}, ${3:0.0});",
    },
    Builtin {
        name: "teleport_to",
        kind: BuiltinKind::Statement,
        signature: "teleport_to(type = .., tag = .., distance = .., sort = ..)",
        summary: "Move the executing entity to the nearest entity that matches.",
        details: "Takes the same filters as `entities(..)`. One entity is chosen, the nearest unless `sort` says otherwise. Nothing happens if none match.",
        minecraft: "tp @s @e[...,limit=1,sort=nearest]",
        needs_player: false,
        params: QUERY,
        snippet: "teleport_to(type = Entity::${1:Player});",
    },
    Builtin {
        name: "attach_state",
        kind: BuiltinKind::Statement,
        signature: "attach_state(State)",
        summary: "Attach an `entity state` to the executing entity.",
        details: "Fields that have no value yet take their declared defaults; existing values are kept, so attaching twice is safe.",
        minecraft: "scoreboard players set @s <fields and presence>",
        needs_player: false,
        params: &[p(
            "state",
            ParamKind::EntityState,
            true,
            "The `entity state` to attach.",
        )],
        snippet: "attach_state(${1:State});",
    },
    Builtin {
        name: "detach_state",
        kind: BuiltinKind::Statement,
        signature: "detach_state(State)",
        summary: "Detach an `entity state` from the executing entity, clearing its fields.",
        details: "Attaching it again later starts from the declared defaults.",
        minecraft: "scoreboard players reset @s <fields and presence>",
        needs_player: false,
        params: &[p(
            "state",
            ParamKind::EntityState,
            true,
            "The `entity state` to detach.",
        )],
        snippet: "detach_state(${1:State});",
    },
    Builtin {
        name: "has_state",
        kind: BuiltinKind::Condition,
        signature: "has_state(State)",
        summary: "Whether the executing entity has an `entity state` attached.",
        details: "Use it in `if` and `while`: `if has_state(Mana) { ... }`.",
        minecraft: "execute if score @s <presence> matches 1",
        needs_player: false,
        params: &[p(
            "state",
            ParamKind::EntityState,
            true,
            "The `entity state` to test.",
        )],
        snippet: "has_state(${1:State})",
    },
    Builtin {
        name: "entities",
        kind: BuiltinKind::Expression,
        signature: "for entity in entities(type = .., tag = .., distance = .., limit = .., sort = ..)",
        summary: "Run a loop body as each matching entity, at its position.",
        details: "Use it as the source of a `for` loop. Inside, the executor is that entity: `kill()`, \
                  `add_tag(..)`, `damage(..)` and position-based builtins act on it, and `continue` skips \
                  to the next one. The entity may not be a player, so player State and player \
                  builtins are not available inside. Distances are measured from the executor \
                  before the loop.",
        minecraft: "execute as @e[...] at @s run function <body>",
        needs_player: false,
        params: QUERY,
        snippet: "for ${1:entity} in entities(type = Entity::${2:Zombie}) {\n    $0\n}",
    },
    Builtin {
        name: "count_entities",
        kind: BuiltinKind::Expression,
        signature: "count_entities(type = .., tag = .., distance = .., limit = .., sort = ..)",
        summary: "How many entities match the filters.",
        details: "An integer expression: `let zombies = count_entities(type = Entity::Zombie, distance = ..=32);`.",
        minecraft: "execute store result score ... if entity @e[...]",
        needs_player: false,
        params: QUERY,
        snippet: "count_entities(type = Entity::${1:Zombie})",
    },
    Builtin {
        name: "kill",
        kind: BuiltinKind::Statement,
        signature: "kill()",
        summary: "Kill the executing entity.",
        details: "Inside `for entity in entities(..)` this kills that entity; in a player function it kills the player.",
        minecraft: "kill @s",
        needs_player: false,
        params: &[],
        snippet: "kill();",
    },
    Builtin {
        name: "add_tag",
        kind: BuiltinKind::Statement,
        signature: "add_tag(\"name\")",
        summary: "Add a tag to the executing entity.",
        details: "Tags mark entities for later queries: `entities(tag = \"boss\")`.",
        minecraft: "tag @s add <name>",
        needs_player: false,
        params: &[p(
            "tag",
            ParamKind::Text,
            true,
            "Tag name: letters, digits, `_ . + -`.",
        )],
        snippet: "add_tag(\"${1:name}\");",
    },
    Builtin {
        name: "remove_tag",
        kind: BuiltinKind::Statement,
        signature: "remove_tag(\"name\")",
        summary: "Remove a tag from the executing entity.",
        details: "Does nothing if the entity does not have the tag.",
        minecraft: "tag @s remove <name>",
        needs_player: false,
        params: &[p("tag", ParamKind::Text, true, "Tag name.")],
        snippet: "remove_tag(\"${1:name}\");",
    },
    Builtin {
        name: "damage",
        kind: BuiltinKind::Statement,
        signature: "damage(amount)",
        summary: "Damage the executing entity.",
        details: "`amount` is in half-hearts: `damage(4.0)` removes two hearts. Armour and effects apply.",
        minecraft: "damage @s <amount>",
        needs_player: false,
        params: &[p(
            "amount",
            ParamKind::Number,
            true,
            "Damage in half-hearts.",
        )],
        snippet: "damage(${1:4.0});",
    },
    Builtin {
        name: "summon",
        kind: BuiltinKind::Statement,
        signature: "summon(Entity::..., tag = \"name\")",
        summary: "Summon an entity at the executor's position.",
        details: "Give it a `tag` to find it again with `entities(tag = ..)`.",
        minecraft: "summon <entity> ~ ~ ~ {Tags:[...]}",
        needs_player: false,
        params: &[
            p(
                "entity",
                ParamKind::Registry("Entity"),
                true,
                "Entity type, such as `Entity::Zombie`.",
            ),
            p(
                "tag",
                ParamKind::Text,
                false,
                "Tag given to the new entity.",
            ),
        ],
        snippet: "summon(Entity::${1:Zombie});",
    },
    Builtin {
        name: "chat",
        kind: BuiltinKind::Macro,
        signature: "chat!(\"text\")",
        summary: "Send a chat message to the executing player.",
        details: FORMAT_DOC,
        minecraft: "tellraw @s [...]",
        needs_player: true,
        params: &[p("text", ParamKind::Format, true, "The message.")],
        snippet: "chat!(\"$0\");",
    },
    Builtin {
        name: "broadcast",
        kind: BuiltinKind::Macro,
        signature: "broadcast!(\"text\")",
        summary: "Send a chat message to every online player.",
        details: FORMAT_DOC,
        minecraft: "tellraw @a [...]",
        needs_player: false,
        params: &[p("text", ParamKind::Format, true, "The message.")],
        snippet: "broadcast!(\"$0\");",
    },
    Builtin {
        name: "actionbar",
        kind: BuiltinKind::Macro,
        signature: "actionbar!(\"text\")",
        summary: "Show text above the executing player's hotbar.",
        details: FORMAT_DOC,
        minecraft: "title @s actionbar [...]",
        needs_player: true,
        params: &[p("text", ParamKind::Format, true, "The text.")],
        snippet: "actionbar!(\"$0\");",
    },
    Builtin {
        name: "title",
        kind: BuiltinKind::Macro,
        signature: "title!(\"text\")",
        summary: "Show large title text to the executing player.",
        details: "Call `subtitle!` first to show a subtitle with this title.\n\n\
                  Text with `{...}` interpolation: see `chat!`.",
        minecraft: "title @s title [...]",
        needs_player: true,
        params: &[p("text", ParamKind::Format, true, "The title.")],
        snippet: "title!(\"$0\");",
    },
    Builtin {
        name: "subtitle",
        kind: BuiltinKind::Macro,
        signature: "subtitle!(\"text\")",
        summary: "Set the subtitle shown with the executing player's next title.",
        details: "Minecraft only displays a subtitle together with a title, so call \
                  `title!` afterwards.",
        minecraft: "title @s subtitle [...]",
        needs_player: true,
        params: &[p("text", ParamKind::Format, true, "The subtitle.")],
        snippet: "subtitle!(\"$0\");",
    },
    Builtin {
        name: "raw",
        kind: BuiltinKind::Macro,
        signature: "raw!(\"command\")",
        summary: "Run one Minecraft command exactly as written.",
        details: "An escape hatch for commands Sand does not model yet. Sand only checks \
                  that it is a single line without a leading `/`; prefer a typed builtin \
                  when one exists.",
        minecraft: "The command itself.",
        needs_player: false,
        params: &[p("command", ParamKind::Text, true, "One command line.")],
        snippet: "raw!(\"$0\");",
    },
    Builtin {
        name: "play_sound",
        kind: BuiltinKind::Statement,
        signature: "play_sound(Sound::..., volume = 1.0, pitch = 1.0)",
        summary: "Play a sound to the executing player at their position.",
        details: "Only the executing player hears it. `pitch` ranges from 0.0 to 2.0.",
        minecraft: "playsound <sound> master @s ~ ~ ~ <volume> <pitch>",
        needs_player: true,
        params: &[
            p(
                "sound",
                ParamKind::Registry("Sound"),
                true,
                "Sound event, such as `Sound::EntityPlayerLevelup`.",
            ),
            p(
                "volume",
                ParamKind::Number,
                false,
                "Loudness; 1.0 is normal.",
            ),
            p(
                "pitch",
                ParamKind::Number,
                false,
                "Pitch from 0.0 to 2.0; 1.0 is normal.",
            ),
        ],
        snippet: "play_sound(Sound::${1:EntityExperienceOrbPickup});",
    },
    Builtin {
        name: "spawn_particle",
        kind: BuiltinKind::Statement,
        signature: "spawn_particle(Particle::..., count = 1, spread = 0.0, speed = 0.0)",
        summary: "Spawn particles one block above the executor.",
        details: "`spread` is the random offset in blocks on each axis.",
        minecraft: "particle <particle> ~ ~1 ~ <spread> <spread> <spread> <speed> <count>",
        needs_player: false,
        params: &[
            p(
                "particle",
                ParamKind::Registry("Particle"),
                true,
                "Particle type, such as `Particle::Heart`.",
            ),
            p("count", ParamKind::Int, false, "How many particles."),
            p(
                "spread",
                ParamKind::Number,
                false,
                "Random offset in blocks.",
            ),
            p("speed", ParamKind::Number, false, "Particle speed."),
        ],
        snippet: "spawn_particle(Particle::${1:Heart}, count = ${2:5});",
    },
    Builtin {
        name: "add_effect",
        kind: BuiltinKind::Statement,
        signature: "add_effect(Effect::..., seconds = .., level = 1, hide_particles = false)",
        summary: "Give the executing player a status effect.",
        details: "Omit `seconds` for an effect that lasts until cleared. `level` is the \
                  displayed level: `level = 2` is Speed II.",
        minecraft: "effect give @s <effect> <seconds|infinite> <level - 1> <hide_particles>",
        needs_player: true,
        params: &[
            p(
                "effect",
                ParamKind::Registry("Effect"),
                true,
                "Status effect, such as `Effect::Speed`.",
            ),
            p(
                "seconds",
                ParamKind::Int,
                false,
                "Duration; omit for infinite.",
            ),
            p("level", ParamKind::Int, false, "Effect level, 1 to 256."),
            p(
                "hide_particles",
                ParamKind::Bool,
                false,
                "Hide the swirling effect particles.",
            ),
        ],
        snippet: "add_effect(Effect::${1:Speed}, seconds = ${2:10});",
    },
    Builtin {
        name: "clear_effect",
        kind: BuiltinKind::Statement,
        signature: "clear_effect(Effect::...)",
        summary: "Remove a status effect, or every effect, from the executing player.",
        details: "With no argument every effect is removed.",
        minecraft: "effect clear @s [effect]",
        needs_player: true,
        params: &[p(
            "effect",
            ParamKind::Registry("Effect"),
            false,
            "Effect to remove; omit to remove all.",
        )],
        snippet: "clear_effect($0);",
    },
    Builtin {
        name: "give_item",
        kind: BuiltinKind::Statement,
        signature: "give_item(Item::..., count = 1)",
        summary: "Give items to the executing player.",
        details: "Items that do not fit in the inventory drop at the player's feet.",
        minecraft: "give @s <item> <count>",
        needs_player: true,
        params: &[
            p(
                "item",
                ParamKind::Registry("Item"),
                true,
                "Item, such as `Item::Diamond`.",
            ),
            p("count", ParamKind::Int, false, "How many, 1 to 6400."),
        ],
        snippet: "give_item(Item::${1:Diamond}, ${2:1});",
    },
    Builtin {
        name: "clear_item",
        kind: BuiltinKind::Statement,
        signature: "clear_item(Item::..., count = ..)",
        summary: "Remove matching items from the executing player's inventory.",
        details: "Omit `count` to remove every matching item.",
        minecraft: "clear @s <item> [count]",
        needs_player: true,
        params: &[
            p("item", ParamKind::Registry("Item"), true, "Item to remove."),
            p(
                "count",
                ParamKind::Int,
                false,
                "Most items to remove; omit for all.",
            ),
        ],
        snippet: "clear_item(Item::${1:Dirt});",
    },
    Builtin {
        name: "clear_inventory",
        kind: BuiltinKind::Statement,
        signature: "clear_inventory()",
        summary: "Remove every item from the executing player's inventory.",
        details: "Includes armour and the off hand.",
        minecraft: "clear @s",
        needs_player: true,
        params: &[],
        snippet: "clear_inventory();",
    },
    Builtin {
        name: "schedule",
        kind: BuiltinKind::Statement,
        signature: "schedule(function, ticks = .. | seconds = ..)",
        summary: "Run a function later on the server.",
        details: "Give exactly one of `ticks` (20 per second) or `seconds`. Scheduling the \
                  same function again replaces the pending run. Scheduled functions run \
                  without a player, so they must not need one; use `for player in \
                  players()` inside them.",
        minecraft: "schedule function <function> <delay>t replace",
        needs_player: false,
        params: &[
            p(
                "function",
                ParamKind::Function,
                true,
                "Function to run; it must run on the server.",
            ),
            p("ticks", ParamKind::Int, false, "Delay in game ticks."),
            p("seconds", ParamKind::Int, false, "Delay in seconds."),
        ],
        snippet: "schedule(${1:function}, seconds = ${2:5});",
    },
    Builtin {
        name: "set_attribute",
        kind: BuiltinKind::Statement,
        signature: "set_attribute(Attribute::..., value, scale = 1)",
        summary: "Set the base value of one of the executing player's attributes.",
        details: "`value` may be a constant like `40.0` or any integer expression, such as \
                  `get_attribute(Attribute::MaxHealth) + 4`. Integer expressions are divided by \
                  `scale`, so `set_attribute(Attribute::MovementSpeed, speed, scale = 1000)` \
                  sets 0.1 when `speed` is 100. Common attributes: `Attribute::MaxHealth`, \
                  `Attribute::MovementSpeed`, `Attribute::AttackDamage`, `Attribute::Armor`, \
                  `Attribute::Scale`.",
        minecraft: "attribute @s <attribute> base set <value> (through a macro for computed values)",
        needs_player: true,
        params: &[
            p(
                "attribute",
                ParamKind::Registry("Attribute"),
                true,
                "Attribute, such as `Attribute::MaxHealth`.",
            ),
            p(
                "value",
                ParamKind::Number,
                true,
                "New base value: a constant or integer expression.",
            ),
            p(
                "scale",
                ParamKind::Number,
                false,
                "Divisor for integer expressions; default 1.",
            ),
        ],
        snippet: "set_attribute(Attribute::${1:MaxHealth}, ${2:24.0});",
    },
    Builtin {
        name: "get_attribute",
        kind: BuiltinKind::Expression,
        signature: "get_attribute(Attribute::..., scale = 1)",
        summary: "The executing player's attribute base value, as an integer.",
        details: "The value is multiplied by `scale` and truncated, so \
                  `get_attribute(Attribute::MovementSpeed, scale = 1000)` reads 0.1 as 100. \
                  Max health is in half-hearts: 20 is ten hearts.",
        minecraft: "execute store result score ... run attribute @s <attribute> base get <scale>",
        needs_player: true,
        params: &[
            p(
                "attribute",
                ParamKind::Registry("Attribute"),
                true,
                "Attribute, such as `Attribute::MaxHealth`.",
            ),
            p(
                "scale",
                ParamKind::Number,
                false,
                "Multiplier before truncation; default 1.",
            ),
        ],
        snippet: "get_attribute(Attribute::${1:MaxHealth})",
    },
    Builtin {
        name: "random",
        kind: BuiltinKind::Expression,
        signature: "random(min..=max)",
        summary: "A uniformly random integer from a range.",
        details: "Use it as the whole right-hand side of `=`: \
                  `Dice.roll = random(1..=6);`.",
        minecraft: "execute store result score ... run random value <min>..<max>",
        needs_player: false,
        params: &[p(
            "range",
            ParamKind::Range,
            true,
            "Inclusive range such as `1..=6`.",
        )],
        snippet: "random(${1:1}..=${2:6})",
    },
];

/// Find a builtin by name and kind.
pub fn find(name: &str, kind: BuiltinKind) -> Option<&'static Builtin> {
    BUILTINS.iter().find(|b| b.name == name && b.kind == kind)
}

/// Markdown documentation for a builtin.
pub fn describe(builtin: &Builtin) -> String {
    let mut out = format!(
        "```sand\n{}\n```\n\n{}\n\n{}",
        builtin.signature, builtin.summary, builtin.details
    );
    if !builtin.params.is_empty() {
        out.push_str("\n\n**Arguments**\n");
        for param in builtin.params {
            let ty = match param.kind {
                ParamKind::Registry(registry) => format!(" `{registry}`"),
                _ => String::new(),
            };
            out.push_str(&format!(
                "\n- `{}`{ty}{}: {}",
                param.name,
                if param.required { "" } else { " (optional)" },
                param.doc
            ));
        }
    }
    if builtin.needs_player {
        out.push_str("\n\nNeeds a player: the function becomes player-context.");
    }
    if EXECUTOR_BUILTINS.contains(&builtin.name) {
        out.push_str(
            "\n\nActs on the executing entity: use it inside `for entity in entities(..)`, or the function runs as an entity.",
        );
    }
    out.push_str(&format!("\n\nMinecraft: `{}`", builtin.minecraft));
    out
}

/// Player events usable in `#[on(...)]`, with the value set a filtered event
/// takes (`None` for events written as a bare word).
pub const EVENTS: &[(&str, Option<&str>, &str)] = &[
    (
        "join",
        None,
        "Every time a player joins, including their first join. Observed on the next tick.",
    ),
    (
        "first_join",
        None,
        "Only the very first time a player joins the world.",
    ),
    (
        "death",
        None,
        "The player died, from any cause. Runs while they are on the death screen.",
    ),
    (
        "kill",
        Some("Entity"),
        "The player killed an entity: any entity as `#[on(kill)]`, or one type as `#[on(kill = Entity::Zombie)]`.",
    ),
    ("kill_player", None, "The player killed another player."),
    (
        "killed_by",
        Some("Entity"),
        "An entity of one type killed the player: `#[on(killed_by = Entity::Creeper)]`.",
    ),
    (
        "use_item",
        Some("Item"),
        "The player used an item: `#[on(use_item = Item::CarrotOnAStick)]`. Fires for right-click items, consumables, and placed blocks.",
    ),
    (
        "craft_item",
        Some("Item"),
        "The player crafted an item: `#[on(craft_item = Item::Bread)]`.",
    ),
    (
        "pick_up_item",
        Some("Item"),
        "The player picked up an item: `#[on(pick_up_item = Item::Diamond)]`.",
    ),
    (
        "mine_block",
        Some("Block"),
        "The player mined a block: `#[on(mine_block = Block::DiamondOre)]`.",
    ),
    ("jump", None, "The player jumped."),
    ("drop_item", None, "The player dropped an item."),
    ("sleep", None, "The player slept in a bed."),
    ("take_damage", None, "The player took damage."),
    ("deal_damage", None, "The player dealt damage."),
];

/// Style names usable inside `{...}` in text.
pub const STYLES: &[&str] = &[
    "black",
    "dark_blue",
    "dark_green",
    "dark_aqua",
    "dark_red",
    "dark_purple",
    "gold",
    "gray",
    "dark_gray",
    "blue",
    "green",
    "aqua",
    "red",
    "light_purple",
    "yellow",
    "white",
    "bold",
    "italic",
    "underlined",
    "reset",
];

/// Attribute documentation.
pub const ATTRIBUTES: &[(&str, &str, &str)] = &[
    (
        "pack",
        "#![pack(namespace = \"...\", minecraft = \"...\", description = \"...\")]",
        "Declares the pack in the root file. `namespace` names every generated resource; `minecraft` selects the exact target version (newest supported by default); `description` defaults to the root file's `//!` docs.",
    ),
    (
        "tick",
        "#[tick]",
        "Run this function once every game tick (20 times per second) on the server, after Sand's State maintenance and flows. It has no player: use `for player in players()` for per-player work.",
    ),
    (
        "load",
        "#[load]",
        "Run this function when the datapack loads or `/reload` runs, on the server without a player.",
    ),
    (
        "tag",
        "#[tag(\"namespace:name\")]",
        "Add this function to a function tag. `minecraft:tick` and `minecraft:load` run without a player.",
    ),
    (
        "player",
        "#[player]",
        "Require a player executor explicitly. Usually unnecessary: Sand infers it when a function touches player State. On an `extern fn`, declares that the external function needs a player.",
    ),
    (
        "clamp",
        "#[clamp(min..=max)]",
        "Clamp every write to this `number` or `float` field into the range. The declared default is kept until the first write.",
    ),
    (
        "on",
        "#[on(event)]",
        "Run this function as the player whenever a player event happens, or whenever a custom `event` is emitted (`#[on(Payday)]`). Player events: `join`, `first_join`, `death`, `kill`, `kill_player`, `killed_by`, `use_item`, `craft_item`, `pick_up_item`, `mine_block`, `jump`, `drop_item`, `sleep`, `take_damage`, `deal_damage`.",
    ),
];

/// Keyword documentation.
pub const KEYWORDS: &[(&str, &str)] = &[
    (
        "state",
        "`player state Name { ... }` stores one value per player; `global state Name { ... }` stores one value for the whole server. Fields are `number`, `float`, `bool`, or enum values, saved in the world.",
    ),
    (
        "player",
        "`player state` stores values per player. Reading or writing them needs a player executor, such as inside `for player in players()`.",
    ),
    (
        "global",
        "`global state` stores one shared value for the whole server, usable from anywhere.",
    ),
    (
        "entity",
        "`entity state` stores values on individual entities. Attach it with `attach_state(Name)`, then read and write it while running as that entity.",
    ),
    (
        "flow",
        "`flow Name on State.field { ... }` is a state machine. Transitions `From -> To if condition priority n;` move between phases (at most one per tick). Hooks `enter`, `exit`, and `tick Phase every n` run functions or inline blocks.",
    ),
    (
        "enter",
        "`enter Phase => function;` runs after a flow transition moves into `Phase`.",
    ),
    (
        "exit",
        "`exit Phase => function;` runs before a flow transition leaves `Phase`.",
    ),
    (
        "every",
        "`tick Phase every n` runs the hook on every n-th tick spent in the phase.",
    ),
    (
        "priority",
        "Transitions with a higher `priority` are tried first; ties keep declaration order.",
    ),
    (
        "players",
        "`for player in players() { ... }` runs the body once as each online player, so player State and player builtins refer to that player.",
    ),
    (
        "for",
        "`for player in players() { ... }` runs the body once as each online player.",
    ),
    (
        "if",
        "`if condition { ... } else { ... }` compares State fields with constants or other fields (`== != < <= > >=`, `in a..=b`), combined with `&&`, `||`, and `!`. Each comparison is evaluated once and exactly one branch runs.",
    ),
    (
        "fn",
        "A Minecraft function. Sand infers whether it runs as a player (it touches player State or calls something that does) or on the server.",
    ),
    (
        "enum",
        "Named integer phases. Enum fields can only be assigned and compared with `==`/`!=`; convert with `as number`.",
    ),
    (
        "const",
        "A compile-time integer constant. Arithmetic (`+ - * / %`) is folded during compilation.",
    ),
    (
        "mod",
        "`mod name;` loads `name.sand` or `name/mod.sand`; `mod name { ... }` declares an inline module.",
    ),
    (
        "use",
        "Imports a path, as in Rust: `use crate::rewards::grant;`.",
    ),
    ("pub", "Makes an item visible outside its module."),
    (
        "extern",
        "`extern \"namespace:path\" fn name();` declares a function implemented by another datapack.",
    ),
    (
        "event",
        "`event Name(params);` declares a custom event. Handlers are functions marked `#[on(Name)]` with the same parameters; `emit(Name, args)` runs them all in declaration order, as the emitter.",
    ),
    (
        "timer",
        "A `state` field that counts down by one every tick until zero, usable as a timer or cooldown. Start it with `start_timer(State.field, seconds = 5)`, test it with `timer_running` or `timer_done`, and read it as a number of ticks left.",
    ),
    (
        "number",
        "A whole number, stored directly in a scoreboard score (32-bit).",
    ),
    (
        "float",
        "A decimal with three places, such as `1.5`. Scoreboards only hold whole numbers, so Sand stores `value * 1000` and rescales arithmetic for you. Convert with `as float` and `as number`. Range is about \u{b1}2,147,483, and multiplying two large fields can overflow.",
    ),
    (
        "bool",
        "`true` or `false`, stored as one or zero. Use it directly in conditions: `if ready { ... }`, and assign conditions to it: `let ok = coins >= 50;`.",
    ),
];

/// What a data-declaration field accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    /// A vanilla item (`Item::Diamond`) or a declared custom item.
    Item,
    /// A recipe ingredient: a vanilla item, an item `tag`, or `"#ns:tag"`.
    Ingredient,
    /// A list of ingredients.
    Ingredients,
    /// Text with style interpolation, such as `"{red}Ruby Sword"`.
    Text,
    /// A list of text lines.
    TextList,
    /// A shaped pattern: a list of one to three strings.
    Pattern,
    /// `{ X: ingredient, ... }` for a shaped pattern.
    Key,
    /// `true` or `false`.
    Bool,
    /// An integer constant.
    Int,
    /// A decimal or integer constant.
    Number,
    /// A value from a typed set, such as `Rarity::Epic`.
    Registry(&'static str),
    /// `{ "minecraft:sharpness": 5, ... }`.
    Enchantments,
    /// A list of tag values for the declaration's registry.
    TagValues,
    /// A list of loot choices.
    LootItems,
}

/// One documented field of a data declaration.
#[derive(Debug, Clone, Copy)]
pub struct DataFieldDoc {
    /// Declaration keyword: `item`, `recipe`, item `tag`, ...
    pub kind: &'static str,
    /// Field name.
    pub name: &'static str,
    /// Accepted value.
    pub value: FieldKind,
    /// Whether the field must be present (recipe kinds are checked separately).
    pub required: bool,
    /// Explanation for hover and completion.
    pub doc: &'static str,
    /// Completion snippet for the value.
    pub snippet: &'static str,
}

const fn f(
    kind: &'static str,
    name: &'static str,
    value: FieldKind,
    required: bool,
    doc: &'static str,
    snippet: &'static str,
) -> DataFieldDoc {
    DataFieldDoc {
        kind,
        name,
        value,
        required,
        doc,
        snippet,
    }
}

/// The recipe fields that choose the recipe type; exactly one is required.
pub const RECIPE_TYPES: &[&str] = &[
    "shaped",
    "shapeless",
    "smelting",
    "blasting",
    "smoking",
    "campfire",
    "stonecutting",
    "smithing",
];

/// Every data-declaration field.
pub const DATA_FIELDS: &[DataFieldDoc] = &[
    f(
        "item",
        "base",
        FieldKind::Item,
        true,
        "The vanilla item this custom item is built on, such as `Item::DiamondSword`.",
        "Item::${1:DiamondSword}",
    ),
    f(
        "item",
        "name",
        FieldKind::Text,
        false,
        "Display name. Styles work: `\"{red}{bold}Ruby Sword\"`.",
        "\"${1:Name}\"",
    ),
    f(
        "item",
        "lore",
        FieldKind::TextList,
        false,
        "Tooltip lines under the name: `[\"{gray}Forged in fire.\"]`.",
        "[\"${1:line}\"]",
    ),
    f(
        "item",
        "rarity",
        FieldKind::Registry("Rarity"),
        false,
        "Name colour: `Rarity::Common`, `Rarity::Uncommon`, `Rarity::Rare`, or `Rarity::Epic`.",
        "Rarity::${1:Rare}",
    ),
    f(
        "item",
        "glint",
        FieldKind::Bool,
        false,
        "Force the enchantment shimmer on (`true`) or off (`false`).",
        "true",
    ),
    f(
        "item",
        "unbreakable",
        FieldKind::Bool,
        false,
        "Never loses durability.",
        "true",
    ),
    f(
        "item",
        "max_stack",
        FieldKind::Int,
        false,
        "Largest stack size, 1 to 99.",
        "${1:16}",
    ),
    f(
        "item",
        "enchantments",
        FieldKind::Enchantments,
        false,
        "Enchantments and levels: `{ \"minecraft:sharpness\": 5 }`.",
        "{ \"minecraft:${1:sharpness}\": ${2:1} }",
    ),
    f(
        "item",
        "custom_model_data",
        FieldKind::Int,
        false,
        "Selects a resource-pack model.",
        "${1:1}",
    ),
    f(
        "recipe",
        "shaped",
        FieldKind::Pattern,
        false,
        "A crafting-table pattern of one to three rows; spaces are empty slots. Needs `key`.",
        "[\"${1:AAA}\", \"${2: B }\", \"${3: B }\"]",
    ),
    f(
        "recipe",
        "key",
        FieldKind::Key,
        false,
        "What each pattern character means: `{ A: Item::Diamond, B: Item::Stick }`.",
        "{ ${1:A}: Item::${2:Diamond} }",
    ),
    f(
        "recipe",
        "shapeless",
        FieldKind::Ingredients,
        false,
        "Crafting ingredients in any arrangement, one to nine.",
        "[Item::${1:Dirt}]",
    ),
    f(
        "recipe",
        "smelting",
        FieldKind::Ingredient,
        false,
        "Cook this ingredient in a furnace.",
        "Item::${1:RawIron}",
    ),
    f(
        "recipe",
        "blasting",
        FieldKind::Ingredient,
        false,
        "Cook this ingredient in a blast furnace.",
        "Item::${1:RawIron}",
    ),
    f(
        "recipe",
        "smoking",
        FieldKind::Ingredient,
        false,
        "Cook this ingredient in a smoker.",
        "Item::${1:Beef}",
    ),
    f(
        "recipe",
        "campfire",
        FieldKind::Ingredient,
        false,
        "Cook this ingredient on a campfire.",
        "Item::${1:Beef}",
    ),
    f(
        "recipe",
        "stonecutting",
        FieldKind::Ingredient,
        false,
        "Cut this ingredient in a stonecutter.",
        "Item::${1:Stone}",
    ),
    f(
        "recipe",
        "smithing",
        FieldKind::Ingredient,
        false,
        "The smithing template of an upgrade recipe. Needs `base` and `addition`.",
        "Item::${1:NetheriteUpgradeSmithingTemplate}",
    ),
    f(
        "recipe",
        "base",
        FieldKind::Ingredient,
        false,
        "The item a `smithing` recipe upgrades.",
        "Item::${1:DiamondSword}",
    ),
    f(
        "recipe",
        "addition",
        FieldKind::Ingredient,
        false,
        "The material a `smithing` recipe consumes.",
        "Item::${1:NetheriteIngot}",
    ),
    f(
        "recipe",
        "result",
        FieldKind::Item,
        true,
        "The item produced: a vanilla item or one of your custom items.",
        "Item::${1:Diamond}",
    ),
    f(
        "recipe",
        "count",
        FieldKind::Int,
        false,
        "How many are produced, 1 to 99.",
        "${1:1}",
    ),
    f(
        "recipe",
        "experience",
        FieldKind::Number,
        false,
        "Experience from a cooking recipe.",
        "${1:0.1}",
    ),
    f(
        "recipe",
        "time",
        FieldKind::Int,
        false,
        "Cooking time in ticks (furnace default 200).",
        "${1:200}",
    ),
    f(
        "tag",
        "values",
        FieldKind::TagValues,
        true,
        "Values of the tag's type, or other tags of that type: `[Item::IronOre, CoalOres, \"#minecraft:logs\"]`.",
        "[${1}]",
    ),
    f(
        "tag",
        "replace",
        FieldKind::Bool,
        false,
        "Replace other packs' definitions of this tag instead of merging.",
        "true",
    ),
    f(
        "bossbar",
        "title",
        FieldKind::Text,
        true,
        "Text shown above the bar. Styles work; scores and `{player}` do not.",
        "\"${1:Title}\"",
    ),
    f(
        "bossbar",
        "color",
        FieldKind::Registry("BossBarColor"),
        false,
        "Bar colour; white by default.",
        "BossBarColor::${1:Red}",
    ),
    f(
        "bossbar",
        "style",
        FieldKind::Registry("BossBarStyle"),
        false,
        "How the bar is divided; one solid bar by default.",
        "BossBarStyle::${1:Progress}",
    ),
    f(
        "bossbar",
        "max",
        FieldKind::Int,
        false,
        "Largest value; 100 by default.",
        "${1:100}",
    ),
    f(
        "loot",
        "items",
        FieldKind::LootItems,
        true,
        "What can be drawn: vanilla items, or `{ item: Item::Diamond, weight: 2, count: 3 }`.",
        "[Item::${1:Diamond}]",
    ),
    f(
        "loot",
        "rolls",
        FieldKind::Int,
        false,
        "How many choices are drawn each time; 1 by default.",
        "${1:1}",
    ),
];

/// Fields of one declaration kind.
pub fn data_fields(kind: &str) -> impl Iterator<Item = &'static DataFieldDoc> + '_ {
    DATA_FIELDS.iter().filter(move |field| field.kind == kind)
}

/// Declaration keyword documentation.
pub const DATA_KINDS: &[(&str, &str)] = &[
    (
        "item",
        "`item Name { base: Item::..., ... }` declares a custom item: a vanilla item with a name, lore, and other components. `give_item(Name)` gives it, `clear_item(Name)` removes it, and recipes can produce it. Its identity is a `custom_data` marker, the same as Rust's `CustomItem::custom_data`.",
    ),
    (
        "recipe",
        "`recipe Name { shaped: [...], key: {...}, result: ... }` adds a crafting, cooking, stonecutting, or smithing recipe. Choose the type with exactly one of `shaped`, `shapeless`, `smelting`, `blasting`, `smoking`, `campfire`, `stonecutting`, or `smithing`.",
    ),
    (
        "bossbar",
        "`bossbar Name { title: \"...\" }` declares a bar shown at the top of the screen. Show it with `show_bossbar` and fill it with `set_bossbar`.",
    ),
    (
        "loot",
        "`loot Name { items: [...] }` declares a loot table drawn from weighted item choices. Use it with `give_loot` or `drop_loot`.",
    ),
    (
        "tag",
        "`tag Name: Item { values: [...] }` groups values of one type: `Item`, `Block`, or `Entity`. An item tag can be a recipe ingredient; any tag can include other tags of the same type.",
    ),
];
