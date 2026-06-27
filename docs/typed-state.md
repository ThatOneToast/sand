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

`#[derive(SandStorage)]` is the preferred schema declaration for new code. `PlayerSchema` tracks score/flag/cooldown initialization and storage descriptors but does not create per-player dynamic NBT: Minecraft storage is global. See [storage reference](storage-nbt.md) and the [player-data guide](../book/src/manual/player-data.md).
