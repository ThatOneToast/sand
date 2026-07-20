# 14. Timers And Cooldowns

Chapter 7 introduced `Cooldown` and `Timer` as distinct state primitives.
This chapter looks at how they're actually driven each tick and why the
two-step "check, then restart" pattern matters.

## `Cooldown`: gate an action, then start it

```rust,ignore
static GRAPPLE: Cooldown = Cooldown::new("trail_grapple", Ticks::seconds(4));
```

A `Cooldown` is consumed by player action, not by continuous ticking on its
own. Its lifecycle in Trailforge:

1. **`GRAPPLE.define()`** in `load` — sets up the backing scoreboard
   objective.
2. **`GRAPPLE.ready("@s")`** — read in every gate that decides whether the
   dash may run (`tick`'s actionbar condition and `trail:grapple`'s
   dispatch condition both check this).
3. **`GRAPPLE.start(Selector::self_())`** — written once, in
   `trail:grapple/execute`, the moment the dash actually happens. This is
   what makes it a *cooldown* rather than a *timer*: nothing restarts it
   automatically on a schedule, only the triggering action does.
4. **`GRAPPLE.stop(Selector::self_())`** — written on death (chapter 15),
   so a respawned player isn't stuck waiting out a cooldown against a dash
   they can no longer be mid-cooldown from meaningfully.

`Cooldown` internally compares stored gametime against the current tick, so
`.ready("@s")` becomes an `execute if score`-shaped comparison Sand
generates for you — no hand-written gametime arithmetic.

## `Timer`: restart continuously, drive system pacing

```rust,ignore
static REGEN: Timer = Timer::new("trail_regen", Ticks::seconds(2));
```

Compare that to the regen timer's tick-loop usage (chapter 3):

```rust,ignore
TypedExecute::as_players()
    .when(all![REGEN.expired("@s"), STAMINA.of("@s").lt(100)])
    .run(STAMINA.add(Selector::self_(), 10));
TypedExecute::as_players()
    .when(REGEN.expired("@s"))
    .run(REGEN.start(Selector::self_()));
```

This is two separate guarded statements, not one — and that split is
deliberate. The first statement *acts* on expiry (grants stamina, but only
if the player isn't already capped, so `REGEN.expired` firing doesn't waste
a write when there's nothing to regenerate). The second statement
*restarts* the timer on expiry, unconditionally, regardless of whether the
cap check above did anything. Merging these into one `when(...).run(...)`
would either restart the timer only when stamina was below cap (silently
pausing regen forever once a player is capped, since `REGEN.expired` would
then never be "handled" again) or require duplicating the restart logic
inside a branch. Keeping "read expiry and act" separate from "read expiry
and restart" as two independent statements avoids that trap entirely — each
one only needs to be correct about its own concern.

## The naming distinction, restated

Use a **`Cooldown`** when the question is "has enough time passed since the
player last did X" (consumed and restarted by an action). Use a **`Timer`**
when the question is "has this repeating interval elapsed" (restarted by
itself, forever, independent of player action) — Trailforge's stamina
regen pulse is the clearest example: it has nothing to do with what any
individual player just did, only with how much real time has passed since
the last pulse.
