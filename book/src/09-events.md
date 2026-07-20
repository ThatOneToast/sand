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

## Built-in player-state transitions

Sand ships a set of common player-state transition events through
`sand::events` (see the [event trigger matrix](reference/event-trigger-matrix.md)
for the full list), all built on the same reusable tracked-transition
provider that backs `PlayerStartSneakingEvent`/`PlayerStopSneakingEvent`:
movement/posture (`PlayerStartSprintingEvent`/`PlayerStopSprintingEvent`,
swimming, flying), fire (`PlayerCaughtFireEvent`/`PlayerExtinguishedEvent`),
gamemode entry/exit (`PlayerEnteredCreativeEvent`/`PlayerExitedCreativeEvent`
and the survival/adventure/spectator equivalents), and health
(`PlayerHealthChangedEvent`/`PlayerHealthLostEvent`/`PlayerHealthGainedEvent`).

Every one of these fires **once** on the transition, not every tick the
state holds — and multiple handlers subscribing to the same underlying
state (e.g. both a start and a stop handler for sprinting) share one
generated provider rather than each polling independently.

Low health is a typed threshold pair rather than a fixed event:

```rust,ignore
use sand::prelude::*;
use sand::events::PlayerLowHealthEvent;

#[event]
fn warn_low_health(event: Event<PlayerLowHealthEvent<6>>) {
    // Fires once when health drops to 3 hearts (6 half-hearts) or below.
    cmd::say("Low health!");
}
```

Exactly one `HALF_HEARTS` threshold may be used per exported pack — mixing
two different values is a build-time tracker conflict, not a silently wrong
export, since Sand cannot honestly share one previous/current baseline
between two different thresholds under one tracker.

Status effects use a generic pair instead of one type per effect:

```rust,ignore
use sand::prelude::*;
use sand::events::{EffectStarted, EffectStopped, Speed};

#[event]
fn on_speed_start(event: Event<EffectStarted<Speed>>) {
    cmd::say("Speed boost active!");
}

#[event]
fn on_speed_stop(event: Event<EffectStopped<Speed>>) {
    cmd::say("Speed boost ended.");
}
```

`StatusEffectMarker` is implemented for the supported vanilla effects
(`Poison`, `Wither`, `Regeneration`, `FireResistance`, `Strength`,
`Weakness`, `Speed`, `Slowness`, `Resistance`, `Absorption`, `Hunger`,
`MiningFatigue`, `Nausea`, `Blindness`, `Levitation`, `Glowing`,
`Invisibility`) — only effects with a registered handler generate any
detection infrastructure.

**Freezing and drowning are intentionally not covered.** Unlike on-fire
(a stable `flags.is_on_fire` entity predicate flag), vanilla Java exposes
freezing only through the raw `ticks_frozen`/`ticks_frozen_max` NBT ratio
and drowning only through the raw `Air` NBT stat — neither has a boolean
entity predicate flag or scoreboard criterion as of Minecraft Java 26.2.
Exposing them would mean an author-chosen threshold sampled via
`data get`, which is an inferred approximation rather than an exact
transition; see [Vanilla Limitations](reference/vanilla-limitations.md)
for the evidence trail.

## Participant context: attacker, weapon, victim, and friends

Some events know more than just "which player triggered this." Combat
events can (sometimes) name who dealt the damage; damage-dealing events can
snapshot the weapon involved. `Event<E>` exposes this as typed, honestly
labeled participant context:

```rust,ignore
use sand::prelude::*;

#[event]
fn on_hurt(event: Event<EntityDamagePlayerEvent>) {
    if let ParticipantAvailability::Available(attacker) = event.attacker() {
        cmd::tellraw(attacker.selector().selector(), Text::new("tagged you!"));
    }
}

#[event]
fn on_hit(event: Event<PlayerDamageEntityEvent>) {
    if let ParticipantAvailability::Available(weapon) = event.weapon() {
        // build commands against weapon's captured item data
    }
}
```

`ParticipantAvailability<T>` is never collapsed into a plain `Option<T>` —
`Unavailable(reason)` distinguishes "vanilla genuinely cannot supply this"
from an event-semantic absence, and every `Available` value carries its own
`ParticipantReliability` (`Correlated` for the attacker/killer, backed by
`execute on attacker`; `ExactSnapshot` for weapon/held-item, a captured
mainhand/offhand snapshot). Roles vanilla doesn't credibly expose for a
given event — victim, direct attacker, interacted entity, projectile,
ammunition — resolve `Unavailable` rather than guessing; see
[Vanilla Limitations](reference/vanilla-limitations.md) for the full
role-by-role breakdown.

You only see this context on events that declare it. Declaring it yourself
on a custom `AdvancementEvent`/`SandEvent` is one call in the definition,
applied automatically by the compiler — no manual command splicing:

```rust,ignore
impl AdvancementEvent for MyCombatEvent {
    // ...
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().observe_correlated_attacker()
    }
}
```

Reach for `sand::participant` directly (`EntityParticipant`,
`EventParticipantPlan`) when `ParticipantAvailability` pattern-matching
alone isn't enough — its own rustdoc covers the full typed handle API.

## Choosing event vs. state vs. storage

As a rule of thumb Trailforge follows throughout: reach for an **event**
when something *happens* (a discrete transition you want to react to
exactly once); reach for **state** (chapter 7) when something *is*
(continuously-true or continuously-numeric); reach for **storage** when the
data is closer to configuration or structured NBT than per-tick gameplay
state. `StaminaExhaustedEvent` exists specifically because "stamina crossed
zero" is a happening, even though `STAMINA` itself is state.
