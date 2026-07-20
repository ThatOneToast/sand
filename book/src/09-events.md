# 9. Events

Trailforge has five event handlers, backed by three different kinds of
event source: vanilla events, an advancement-backed custom event, and a
tick-backed custom event. Choosing among them is the most important
gameplay-design decision this book covers.

## Vanilla events: `FirstJoin` and `OnDeath`

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:event_on_first_join}}
```

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:event_on_death}}
```

`FirstJoin` and `OnDeath` are typed markers from `sand::event::vanilla` for
occurrences vanilla Minecraft already signals natively (an advancement
`minecraft:story/root`-style trigger for first join with no prior
join-flag, and death). `#[event]` generates the scoreboard-backed detection
and dispatch wiring; the handler just receives an `Event<T>` (or, for
built-ins with a stable player accessor, the marker type directly) with
`.player()` giving you the affected entity's selector.

## An advancement-backed custom event: `ObtainedGrappleCoreEvent`

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:event_obtained_grapple_core}}
```

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:event_on_obtained_grapple_core}}
```

`AdvancementEvent` is how you turn a *discrete inventory change* into a
typed event without hand-authoring advancement JSON. `trigger()` returns an
`InventoryChangedTrigger` matching the Grapple Core's custom-data key
(chapter 5) — Minecraft's `minecraft:inventory_changed` advancement
trigger fires whenever a matching item enters the player's inventory by any
means (crafting, `/give`, picking it up), which is exactly "obtained a
Grapple Core," is not something a tick-time score comparison could detect
directly. `guard()` is the second half of the design: without it, this
event would re-fire every time inventory contents shift while a Grapple
Core is still present (dropping and picking items back up, moving items in
the inventory), because advancement-trigger criteria can re-evaluate.
Guarding on `HAS_STRIDERS.of("@s").is_false()` makes the event
functionally "obtained *and don't already own the upgrade*" — once the
handler's own follow-up chain results in the player claiming Trail
Striders, the guard goes false and the event goes quiet permanently for
that player.

Advancement-backed events are the right tool whenever the *thing that
happened* corresponds to something vanilla's own advancement trigger set
already detects (obtaining an item, crafting, killing an entity, changing
dimension, and so on) — see [Advancement
triggers](https://minecraft.wiki/w/Advancement_definition#List_of_triggers)
on the wiki for the full vanilla trigger list, and the
[event trigger coverage](reference/vanilla-limitations.md) note on what
isn't covered.

## A tick-backed custom event: `StaminaExhaustedEvent`

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:event_stamina_exhausted}}
```

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:event_on_stamina_exhausted}}
```

Stamina hitting zero has no advancement trigger — it's pure Sand-managed
scoreboard state, so `SandEvent::dispatch()` describes it as a tick-scoped
condition instead: `SandEventDispatch::tick().as_players().when(...)`.
Compare this to chapter 3's `tick` function, which *also* reads
`STAMINA.of("@s").lte(0)`-shaped conditions directly — the difference is
that a tick-backed event only invokes its handler on the tick the
condition transitions (courtesy of the same guard-against-re-fire pattern:
`EXHAUSTED.of("@s").is_false()` in the dispatch condition, cleared only by
`trail:recover`), while raw `tick` conditions re-evaluate and re-run every
single tick they're true. Use a tick-backed event for "this crossed a
threshold, act once"; use a raw tick guard (chapter 3) for "keep this
continuously true while a condition holds" (like the readiness actionbar,
which *should* redraw every tick while the dash is ready).

## Choosing event vs. state vs. storage

As a rule of thumb Trailforge follows throughout: reach for an **event**
when something *happens* (a discrete transition you want to react to
exactly once); reach for **state** (chapter 7) when something *is*
(continuously-true or continuously-numeric); reach for **storage** when the
data is closer to configuration or structured NBT than per-tick gameplay
state. `StaminaExhaustedEvent` exists specifically because "stamina crossed
zero" is a happening, even though `STAMINA` itself is state.
