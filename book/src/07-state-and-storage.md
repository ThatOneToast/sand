# 7. Player State And Storage

Trailforge tracks five pieces of persistent state, and each one uses a
different Sand primitive because each represents a different *kind* of
data:

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:state}}
```

## Choosing the right primitive

**`ScoreVar<i32>`** (`STAMINA`) — a per-entity integer backed by a
scoreboard objective. Scores are the right choice for anything numeric that
needs arithmetic (`add`, `remove`, comparisons like `.gte(30)`) and that
vanilla systems (sidebar, `execute if score`, `/scoreboard`) can already
read. Stamina is a quantity, not a boolean or a moment — a score.

**`Cooldown`** (`GRAPPLE`) — a scoreboard-backed timer with `start`,
`ready`, and `stop` built on top of a score plus the current game time.
Trailforge could hand-roll this as a raw score compared against
`gametime`, but `Cooldown` packages the "is it ready yet" comparison and
the "start it now" write as named operations, so `GRAPPLE.ready("@s")`
reads as intent rather than as an inlined tick-math expression repeated at
every call site.

**`Flag`** (`HAS_STRIDERS`, `EXHAUSTED`) — a boolean packed into a
scoreboard score (0/1). Use a `Flag` for state that's purely on/off with no
magnitude: "does this player own the upgrade," "is this player currently
exhausted." `.is_true()` / `.is_false()` read better in a condition list
than comparing a raw score to a literal.

**`Timer`** (`REGEN`) — a repeating interval, distinct from `Cooldown` in
intent even though both are gametime-backed: a `Cooldown` gates *player
action* (you may not dash again yet); a `Timer` drives *system pacing*
(every 2 seconds, do the regen pulse) and is meant to restart itself
continuously rather than being consumed by one player action.

**`StorageVar<i32>`** (`GRAPPLE_RANGE`) — a value in command storage
(`trail:data` → `config.grapple_range`), not a scoreboard score. Reach for
storage instead of a score when the value doesn't need scoreboard-native
arithmetic/comparison operators, when it's closer to configuration than
per-player gameplay state, or when it needs to hold something a score
can't (Trailforge's value here is a plain int, but storage is where you'd
reach for lists, compound NBT, or per-structure data). Storage is *global*
by default — see the callout below.

## Storage is global; "per-player storage" is a convention

Command storage in vanilla Minecraft has no native per-entity partitioning
the way scoreboards do. `GRAPPLE_RANGE` is deliberately pack-wide
configuration, so this doesn't matter for Trailforge. If you need
per-player data in storage (inventories, saved positions, anything richer
than a scalar), the standard pattern is a dynamic NBT path keyed by UUID or
name under one storage location — a runtime convention Sand's storage APIs
support, not a vanilla NBT-namespacing feature. This is called out
explicitly in [Vanilla Limitations](reference/vanilla-limitations.md); Sand
gives you typed storage schemas and path builders, but it can't make
storage per-player at the engine level.

## Defining once, in `load`

All five are `.define()`d in `load` (chapter 3) and never redefined
elsewhere — defining an objective vanilla already has is a harmless no-op,
so `load` running again on `/reload` is safe, but *resetting* a value
(`STAMINA.set(...)`) only happens where it's semantically correct to reset
it: on first join and on death (chapter 15), never in `load` itself.
