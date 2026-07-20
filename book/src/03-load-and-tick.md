# 3. Load And Tick

Every Sand datapack has, at minimum, two entry points that vanilla drives
for you: a **load** function that runs once per `/reload` and world start,
and a **tick** function that runs every game tick. Trailforge uses both to
set up and then continuously drive its stamina/grapple system.

## Load: define, don't compute

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:load}}
```

`#[component(Load)]` registers this function to be called from the
datapack's `#minecraft:load` function tag. Its job is narrow on purpose:
**define** every piece of persistent state Trailforge will read or write
later (`STAMINA`, `GRAPPLE`, `HAS_STRIDERS`, `EXHAUSTED`, `REGEN`, and the
optional `DamageTracker` system), and **seed** one storage value
(`GRAPPLE_RANGE`). Load runs on every `/reload`, not just world creation —
so `.define()` calls must be idempotent (defining an already-defined
objective is a no-op in vanilla) and must never *reset* a player's existing
progress. Trailforge doesn't zero out `STAMINA` here; a player who reloads
the datapack mid-session keeps their current stamina.

## Tick: react, don't poll blindly

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:tick}}
```

`#[component(Tick)]` registers this function to the `#minecraft:tick`
function tag, so it runs once per tick for every online player context you
target. Trailforge's tick body is a sequence of small, independently gated
`TypedExecute::as_players().when(condition).run(...)` statements — each one
reads as "for every player where *this* is true, do *this*." That shape
matters for two reasons:

1. **Multiplayer correctness.** `as_players()` iterates all online players
   each tick; every condition is evaluated per-player, so one player's
   stamina running low never affects another player's actionbar or regen.
2. **Determinism over polling.** Rather than a single monolithic "game
   state machine" function, each concern (timer expiry, exhaustion
   recovery, dash readiness, damage feedback) is its own small `when(...)`
   guard. Adding a new tick-driven behavior later means adding one more
   guarded statement, not threading new state through an existing one.

Two patterns worth naming:

- **Timer-driven regen.** `REGEN.expired("@s")` is true for exactly the tick
  a `Timer` crosses its interval; the tick function both *acts* on that
  (adds stamina) and *restarts* the timer in the same tick, in two separate
  `when` blocks, so the "read expiry" and "restart" steps stay simple and
  independently testable rather than merged into one conditional.
- **`all![...]` composition.** Multiple conditions — upgrade owned, cooldown
  ready, enough stamina, not exhausted — combine with the `all!` macro
  (re-exported through the prelude) into a single `execute if` chain. This
  is the same gating logic `trail:grapple` uses in chapter 8 to decide
  whether the dash is actually allowed to run; the actionbar in `tick` and
  the dash function agree because they use the same condition expressions.

## Why load/tick instead of events for this state

Stamina regen and grapple readiness *could* be modeled as custom events
(chapter 9 shows how), but a continuously-ticking, per-player numeric
condition — "is stamina above 30 and cooldown ready and not exhausted" — is
naturally a tick-time check, not a discrete occurrence. Reserve events for
things that *happen* (a player obtains an item, stamina crosses zero);
reserve tick for continuously-recomputed *state*. Trailforge's
`StaminaExhaustedEvent` in chapter 9 is exactly this "state crossing a
threshold" case turned into a proper event, once — the tick function checks
`STAMINA.of("@s").lte(0)` on every tick internally, but the event system
only fires the handler on the tick where it becomes true and stays quiet
afterward, because the event's own guard excludes the case where
`EXHAUSTED` is already set.
