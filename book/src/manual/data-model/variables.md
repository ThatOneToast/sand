# Variables, Scores, Flags, And Timers

Minecraft datapacks do not have Rust-like runtime variables. The normal numeric variable is a **scoreboard objective** plus a **score holder**. An objective is the named column (`arcane_mana`); a holder is a player, entity, or fake name (`@s`, `@a`, `#global`) with a value in that column.

Sand wraps that model without hiding it. `ScoreVar<i32>` is an objective name; `Flag` is an objective whose values are `0`/`1`; `Timer` and `Cooldown` are scoreboard timers.

## Minimal example

```rust
use sand_core::prelude::*;

static MANA: ScoreVar<i32> = ScoreVar::new("arcane_mana");
static CASTING: Flag = Flag::new("arcane_casting");

#[component(Load)]
pub fn load_scores() {
    MANA.define();
    CASTING.define();
    MANA.set("#global", 0);       // a fake-player global value
    MANA.set(Selector::all_players(), 100);
}
```

## Practical example

```rust
static BLINK: Cooldown = Cooldown::new("arcane_blink", Ticks::seconds(3));
static CHANNEL: Timer = Timer::new("arcane_channel", Ticks::seconds(5));

#[component(Tick)]
pub fn tick_state() {
    BLINK.tick(Selector::all_players());
    CHANNEL.tick(Selector::all_players());
}

#[function]
pub fn blink() {
    when(all![MANA.of("@s").gte(20), BLINK.ready("@s"), CASTING.of("@s").is_false()])
        .then_all([MANA.remove("@s", 20), BLINK.start(Selector::self_())]);
}
```

## What gets generated

`MANA.define()` lowers to `scoreboard objectives add arcane_mana dummy`. `MANA.add("@s", 1)` lowers to `scoreboard players add @s arcane_mana 1`. A flag uses the same command form with `0`/`1`. Cooldown/timer tick calls emit scoreboard decrement/update commands; `ready`, `active`, and `is_true` are score conditions used by `execute`.

## Common mistakes

- Define objectives in load, not every tick.
- `reset` removes a holder's score; it is not the same as setting it to zero.
- A selector can target many holders. Use `@s` inside an execute-as context when each player must spend their own score.
- Objective names are global to the datapack world; prefix them (`arcane_...`).

## Limitations

Scores are signed 32-bit integers. They are excellent for counters, booleans, and timers—not arbitrary nested records or decimal/vector data. Use [Storage](storage.md) for records and [Locations](locations.md) for position choices.
