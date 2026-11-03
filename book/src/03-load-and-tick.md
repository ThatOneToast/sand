# 3. Load And Tick

Every Sand datapack has, at minimum, two entry points that vanilla drives
for you: a **load** function that runs once per `/reload` and world start,
and a **tick** function that runs every game tick. Trailforge uses both to
set up and then continuously drive its stamina/grapple system.

## Load: define, don't compute

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:load}}
```

State owns objective creation, per-player defaults, bounds, and auto-ticking.
Load seeds an illustrative storage value and announces the pack; it does not
reset existing players. The ray's range is a build-time setting in
`anchor_ray`, not a runtime read of that storage value.

## Tick: per-player resource maintenance

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:tick}}
```

`Target::players().each` provides the same player handle used by injected
functions and event participants. `Traversal::on(player)` binds every read
and write to that player. Each player starts with 100 stamina, bounded to
0..100. Every 40 ticks an elapsed regen timer restores 10 and restarts;
the State write clamps at 100, including regeneration from 95. Regeneration
continues while unequipped. A full pool skips the restore write but still
restarts its timer. The grouped timer branch evaluates elapsed once
before either write, so restarting the timer cannot suppress the restore.

Exhaustion clears at 50 stamina. The actionbar uses `ability_ready`, the same
helper as activation: unlock, actual Trail Striders equipped, at least 30
stamina, no exhaustion, and ready cooldown. It signals player readiness;
it does not promise a valid anchor is in view. Only activation casts the ray.

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
