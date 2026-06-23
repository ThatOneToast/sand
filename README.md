# Sand

**A Rust-first framework for Minecraft Java datapacks and optional resource packs.**

Sand lets you write typed Rust builders for datapack behavior and compile them
to vanilla Minecraft output: optimized `.mcfunction` files, component JSON,
function tags, generated load/tick helpers, and version-correct pack layouts.

Sand's default path is typed APIs. Raw command strings still exist, but they are
escape hatches for interop, modded commands, snapshot syntax, future Minecraft
features, and advanced debugging.

## Design Principles

- Typed builders first: commands, text, selectors, state, conditions, dialogs,
  items, NBT, events, recipes, loot tables, predicates, tags, and resource refs.
- Minecraft output remains ordinary datapack/resource-pack files.
- Version behavior flows through `VersionProfile` instead of scattered checks.
- Raw strings are explicit escape hatches, not the beginner workflow.

## Quick Start

### Requirements

- Rust 1.93+ with edition 2024 support
- Java 21+ for Minecraft data generation during builds
- Network access on first build so Sand can cache Minecraft assets

The CLI is currently built from this workspace; it is not published with a
stable `cargo install sand` flow yet.

```sh
cargo run -p sand -- new my_pack
cd my_pack
cargo run -p sand -- build
```

### Typed Datapack Code

```rust
use sand_core::prelude::*;
use sand_macros::{component, function};

static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static CASTING: Flag = Flag::new("casting");
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));
static PLAYER_DATA: StorageVar<i32> = StorageVar::new("example:data", "player.mana");

#[component(Load)]
pub fn load() {
    MANA.define();
    CASTING.define();
    DASH.define();
    PLAYER_DATA.set_int(100);
}

#[component(Tick)]
pub fn tick() {
    DASH.tick(Selector::all_players());
    TypedExecute::as_players()
        .when(all![
            MANA.of("@s").gte(25),
            any![DASH.ready("@s"), PLAYER_DATA.exists()],
            CASTING.of("@s").is_false(),
        ])
        .run(Actionbar::show(
            Selector::self_(),
            Text::new("Dash ready").aqua().bold(true),
        ));
}

#[function]
pub fn start() {
    cmd::tellraw(
        Selector::all_players(),
        Text::new("Hello from Sand").gold().bold(true),
    );
}

#[component]
pub fn welcome_dialog() -> Dialog {
    Dialog::multi_action_local("welcome")
        .title(Text::new("Welcome").gold())
        .body(DialogBody::text(Text::new("Choose your next action.").aqua()))
        .button(
            DialogButton::new(Text::new("Start").green())
                .action(DialogAction::run_function(start))
        )
        .button(
            DialogButton::new(Text::new("Rules").yellow())
                .action(DialogAction::open_dialog(DialogRef::local("rules")))
        )
}

#[function]
pub fn open_welcome_menu() {
    cmd::show_dialog(Selector::self_(), DialogRef::local("welcome"));
}
```

## Typed State

Use `ScoreVar<T>`, `Flag`, `Timer`, `Cooldown`, `Ticks`, and `StorageVar<T>`
instead of writing scoreboard or storage commands directly.

```rust
static HEALTH: ScoreVar<i32> = ScoreVar::new("health");
static HAS_DASH: Flag = Flag::new("has_dash");

#[component(Load)]
pub fn load_state() {
    HEALTH.define();
    HEALTH.set("@s", 20);
    HEALTH.add("@s", 1);
    HAS_DASH.enable("@s");
}
```

## Typed Conditions

Conditions lower into valid `execute if/unless` plans. Nested `any!` inside
`all!` expands into multiple legal commands.

```rust
let cond = all![
    MANA.of("@s").between(10, 100),
    any![DASH.ready("@s"), Condition::predicate("example:can_dash")],
    CASTING.of("@s").is_false(),
];

let commands = TypedExecute::as_players().when(cond).run(cmd::function(
    ResourceLocation::new("example", "dash").unwrap(),
));
```

## Typed Damage And Events

Damage builders model vanilla target rules. Direct `/damage` requires one
entity, while high-level damage can safely target many entities:

```rust
#[event]
pub fn on_damaged(event: DamageEvent<MyDamageEvent>) {
    event
        .reflect_damage()
        .to(EntityTargets::nearby(5.0).excluding_players().excluding_self())
        .amount(DamageAmount::fixed(4.0))
        .damage_type(DamageKind::Generic)
        .run();
}
```

Sand lowers the multi-target case through `execute as ... run damage @s ...`
instead of generating invalid `damage @e[...]` commands.

## Typed Status Effects

Use `EffectId` for vanilla and modded status effects. `effect give` is a builder
so duration, amplifier, and particles are explicit:

```rust
cmd::effect_give(Selector::self_(), EffectId::Speed)
    .duration(Ticks::seconds(10))
    .amplifier(1)
    .particles(false);

cmd::effect_clear(Selector::self_());
cmd::effect_clear_effect(Selector::self_(), EffectId::Regeneration);

let custom = EffectId::custom("mymod:arcane_burn").unwrap();
cmd::effect_give(Selector::self_(), custom).seconds(3);
```

Use `StatusEffectInstance`, `PotionContents`, and `SuspiciousStewEffect` for
structured datapack JSON/SNBT effect data. Raw command strings remain available
through explicit escape hatches for unsupported future formats.

## Typed Execute

Use `TypedExecute` for common chains and `ExecuteExt::when`/`unless` for typed
conditions:

```rust
TypedExecute::as_players_at_self()
    .when(MANA.of("@s").gte(25))
    .run(
        cmd::playsound_player(ResourceLocation::new("minecraft", "entity.player.levelup").unwrap())
            .targets(Selector::self_()),
    );
```

## Typed Text And HUD Output

Use `Text`/`TextComponent`, `Actionbar`, `Title`, and `Bossbar` builders instead
of hand-written JSON.

```rust
cmd::tellraw(
    Selector::all_players(),
    Text::new("Quest complete").green().bold(true),
);

Title::of(Selector::self_())
    .title(Text::new("Level Up").gold())
    .subtitle(Text::new("+1 skill point").aqua())
    .build();
```

## Dialogs

Dialogs are typed datapack components and are version-gated through
`VersionProfile::supports_dialogs()`.

```rust
let profile = VersionProfile::latest();
if profile.supports_dialogs() {
    Dialog::multi_action_local("welcome")
        .title(Text::new("Welcome").gold())
        .body(DialogBody::text(Text::new("Pick a path.")))
        .button(
            DialogButton::new(Text::new("Start").green())
                .action(DialogAction::run_function(start))
        )
        .button(
            DialogButton::new(Text::new("Rules").yellow())
                .action(DialogAction::open_dialog(DialogRef::local("rules")))
        );
}
```

## Storage And NBT

Use `StorageVar<T>` and `NbtPath` for typed storage paths. Use scoreboards for
fast integer state and storage for structured data, strings, lists, and
cross-function payloads.

```rust
static DATA: StorageVar<i32> = StorageVar::new("example:data", "players.self.mana");

#[component(Load)]
pub fn load_storage() {
    DATA.set_int(100);
    DATA.as_path().key("regen").set_bool(true);
}
```

## Components

Sand provides typed Rust structs for core datapack components:

- Advancements and criteria
- Recipes
- Loot tables and item modifiers
- Predicates
- Function, item, block, and entity tags
- Dialogs
- Damage types, enchantments, jukebox songs, trims, banner patterns, wolf
  variants, chat types, and worldgen components
- Custom item component builders

## Version Support

Sand targets modern Minecraft Java datapacks, including the 1.19 through 1.21.x
series and the emerging 26.x series. Known pack formats and feature gates live
in `VersionProfile`; unknown future 26.x details resolve through fallback
capabilities until confirmed.

```rust
let profile = VersionProfile::resolve(&MinecraftVersion::parse("1.21.6").unwrap()).unwrap();
assert!(profile.supports_feature("dialogs"));
```

## Escape Hatches

Raw strings remain available, but use them deliberately:

```rust
// Escape hatch: another datapack exposes this as its public API.
cmd::raw("function other_pack:api/do_special_thing");
```

Use raw command or JSON hatches for interop, modded commands, snapshot-only
syntax not modeled by Sand yet, unknown future features, and debugging generated
output. Prefer typed builders for normal datapack logic.

## CLI

| Command | Purpose |
|---|---|
| `sand new <name>` | Create a Sand project |
| `sand init` | Initialize the current directory |
| `sand build` | Compile to `dist/` |
| `sand build --release` | Compile and zip output |
| `sand run` | Build and run a local test server |
| `sand clean` | Remove generated output |
| `sand version` | Print CLI version |

## Architecture

This workspace currently has eight crates:

| Crate | Role |
|---|---|
| `sand` | CLI binary |
| `sand-core` | Typed framework APIs, component export, state, conditions, version model |
| `sand-commands` | Typed Minecraft command builders |
| `sand-components` | Typed datapack JSON/component builders |
| `sand-macros` | `#[function]`, `#[component]`, event, schedule, and item macros |
| `sand-build` | Minecraft data generator and codegen pipeline |
| `sand-resourcepack` | Optional resource-pack and HUD helpers |
| `sand-example` | Integration tests and reference coverage |

Build flow:

1. `sand-build` resolves the target Minecraft version and generates Rust types.
2. `sand-core` and `sand-commands` expose typed builders over generated data.
3. `sand-macros` registers functions and components through inventory.
4. `sand build` collects registered output into datapack/resource-pack files.

## Examples

See `examples/README.md` and `docs/examples.md`. Beginner examples use typed
APIs first; raw examples are quarantined under escape-hatch interop docs.

## Contributing And Testing

Run the same checks used during framework work:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test -p sand-macros
cargo doc --workspace --all-features --no-deps
```

Minecraft is large and version-sensitive. Changes that add public APIs should
include focused tests for generated command strings, component JSON, paths, and
version behavior.

## License

MIT
