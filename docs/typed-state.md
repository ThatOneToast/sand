# Typed State

Use typed state wrappers instead of hand-writing scoreboard plumbing.

```rust
use sand_core::prelude::*;

static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static CASTING: Flag = Flag::new("casting");
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));
static TIMER: Timer = Timer::new("blink", Ticks::seconds(5));
```

State APIs return command builders or command strings that can be used directly
inside `#[function]`, `#[component(Load)]`, and `#[component(Tick)]` bodies.

```rust
#[component(Tick)]
pub fn tick_state() {
    MANA.define();
    MANA.set(Selector::all_players(), 100);
    MANA.add(Selector::all_players(), 1);
    CASTING.disable(Selector::all_players());
    DASH.tick(Selector::all_players());
    TIMER.tick(Selector::all_players());
}
```

Enum-backed gameplay states use explicit scoreboard values while keeping callers
on named Rust variants:

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
enum BossPhase {
    Idle = 0,
    Fighting = 1,
    Enraged = 2,
}

impl TypedGameState for BossPhase {
    fn to_score(self) -> i32 {
        self as i32
    }

    fn from_score(score: i32) -> Option<Self> {
        match score {
            0 => Some(Self::Idle),
            1 => Some(Self::Fighting),
            2 => Some(Self::Enraged),
            _ => None,
        }
    }
}

static PHASE: GameState<BossPhase> =
    GameState::with_default_score("boss_phase", 0);

#[component(Load)]
pub fn load_phase() {
    PHASE.define();
}

#[function]
pub fn enrage_boss() {
    PHASE.of("@s").set(BossPhase::Enraged);
}

#[function]
pub fn reset_phase() {
    PHASE.of("@s").reset();
}
```

Use explicit discriminants for persistent player/world state. Reordering or
renumbering variants changes the meaning of stored scoreboard values.
`with_default_score` stores the default as the enum's scoreboard value so
`reset()` can restore it without storing the enum type itself. `clear()` removes
the score entry when you need Minecraft's missing-score behavior.

For structured state, prefer `StorageSchema<T>` and typed fields:

```rust
#[derive(Debug)]
struct PlayerMagic;

static MAGIC: StorageSchema<PlayerMagic> =
    StorageSchema::new("example:data", "players.self.magic");
static MANA_DATA: StorageField<PlayerMagic, i32> = MAGIC.field("mana");
static SCHOOL: StorageField<PlayerMagic, String> = MAGIC.field("school");

#[component(Load)]
pub fn load_storage() {
    MANA_DATA.set(100);
    SCHOOL.set("pyromancy");
}
```

`StorageVar<T>` remains available for simple legacy variables. Use `RawSnbt`
only as an explicit escape hatch when typed `SnbtValue`/`SnbtCompound` builders
do not cover the shape.

`#[derive(SandStorage)]` is the preferred schema declaration for new code. `PlayerDataSchema` (`PlayerSchema` is an alias) tracks score/flag/timer/cooldown initialization and storage descriptors but does not create per-player dynamic NBT: Minecraft storage is global. See [storage reference](storage-nbt.md) and the [player-data guide](../book/src/manual/player-data.md).
