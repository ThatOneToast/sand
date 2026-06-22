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
inside `mcfunction!`.

```rust
mcfunction! {
    MANA.define();
    MANA.set(Selector::all_players(), 100);
    MANA.add(Selector::all_players(), 1);
    CASTING.disable(Selector::all_players());
    DASH.tick(Selector::all_players());
    TIMER.tick(Selector::all_players());
}
```

For structured state, use `StorageVar<T>`:

```rust
static DATA: StorageVar<i32> = StorageVar::new("example:data", "players.self.mana");

mcfunction! {
    DATA.set_int(100);
    DATA.as_path().key("regen").set_bool(true);
}
```
