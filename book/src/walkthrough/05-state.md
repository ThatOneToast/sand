# 5. State, Flags, Timers, And Cooldowns

## What you will build

Turn mana into a usable spell resource, add a casting boolean, a blink cooldown, and a channel timer. This chapter answers how “variables” work inside Minecraft.

## Concepts introduced

`ScoreVar`, `Flag`, `Timer`, `Cooldown`, score holders, scoreboard objectives, `define`, `set`, `add`, `reset`, `ready`, `active`, and `is_true`.

## File changes

In `arcane/src/lib.rs`, add the declarations and replace the load/tick sections with:

```rust
use sand_core::prelude::*;
use sand_macros::{component, function};

static MANA: ScoreVar<i32> = ScoreVar::new("arcane_mana");
static CASTING: Flag = Flag::new("arcane_casting");
static BLINK: Cooldown = Cooldown::new("arcane_blink", Ticks::seconds(3));
static CHANNEL: Timer = Timer::new("arcane_channel", Ticks::seconds(5));

#[component(Load)]
pub fn arcane_load() {
    MANA.define(); CASTING.define(); BLINK.define(); CHANNEL.define();
    MANA.set(Selector::all_players(), 100);
    CASTING.set(Selector::all_players(), false);
}

#[component(Tick)]
pub fn arcane_tick() {
    BLINK.tick(Selector::all_players());
    CHANNEL.tick(Selector::all_players());
}

#[function("arcane:blink")]
pub fn blink() {
    when(all![MANA.of("@s").gte(20), BLINK.ready("@s"), CASTING.of("@s").is_false()])
        .then_all([MANA.remove("@s", 20), BLINK.start(Selector::self_())]);
}

#[function("arcane:cancel_channel")]
pub fn cancel_channel() { CHANNEL.reset("@s"); CASTING.set("@s", false); }
```

## How it works

Minecraft stores each numeric value in a scoreboard objective/holder pair. `arcane_mana` is the objective; `@s` is the holder. `Flag` uses the same storage with false/true represented as `0`/`1`. A cooldown is a score that Sand starts and decrements through `tick`; `ready` is a score-range condition. `Timer` is similar but models a general duration. `reset` removes/reset its value; `set` writes an explicit value.

## What Sand generates

`MANA.define()` → `scoreboard objectives add arcane_mana dummy`; `CASTING.set(@s, false)` → `scoreboard players set @s arcane_casting 0`; the blink guard lowers to an `execute if score` chain, then executes score removal/start commands.

## Try it in Minecraft

After `/reload`, run `/function arcane:blink`, then `/scoreboard players get @s arcane_mana`. Invoke it again immediately: the cooldown guard prevents another spend. Wait three seconds and retry.

## Common mistakes

- Calling `tick` only for one player when every online player uses the cooldown.
- Forgetting `@s` requires an executor context; `/function` invoked by a player supplies one.
- Expecting `Flag` to be a Rust boolean at runtime—it is a scoreboard value.

## Deeper reading

[Variables](../manual/data-model/variables.md), [State](../manual/state.md), and [Conditions](../manual/conditions.md).
