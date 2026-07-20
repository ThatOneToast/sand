# 15. Death And Respawn

Trailforge resets its traversal state on death so a respawned player starts
from a clean, steady baseline rather than carrying stale penalties or
depleted resources into their new life:

{{#include ../../examples/book_project/src/lib.rs:event_on_death}}

## Why death needs explicit handling here

Three pieces of state would otherwise persist incorrectly across a death:

- **`EXHAUSTED.disable(Selector::self_())`** — a player who died while
  exhausted shouldn't respawn still slowed and stamina-locked; exhaustion
  is a *penalty state* tied to the life that just ended, not a punishment
  that should follow them into the next one.
- **`GRAPPLE.stop(Selector::self_())`** — clears any in-progress cooldown,
  so a respawned player isn't stuck waiting out a dash cooldown from a
  dash that (from their perspective, after respawning at the world spawn
  or their bed) has no continuity with their new position.
- **`STAMINA.set(event.player(), 100)`** — restores full stamina, mirroring
  vanilla's own full-health/full-hunger respawn convention. Trailforge's
  own resource should follow the same "you get a clean slate" expectation
  players already have from vanilla respawn.

## `OnDeath` vs. a respawn event

Trailforge listens for `OnDeath`, not a separate respawn event, and
performs the reset immediately rather than waiting for the respawn screen
to close. This works because none of the three resets above are
player-visible until the player is actually back in the world (a dead
player doesn't see actionbars or receive dash inputs), and because
`OnDeath` fires exactly once per death with a reliable player context —
there's no race with the respawn UI to account for. If a pack needed to
distinguish "died" from "actually respawned and back in control" (for
example, to play a respawn-specific title/subtitle only once control
returns), that's a real distinction vanilla's own event surface is limited
on — see the [event trigger
coverage](reference/vanilla-limitations.md) note; Sand's `on_death`-style
handlers fire on the death signal itself, and Sand cannot invent a
finer-grained "control returned to player" signal vanilla doesn't expose.

## The pattern generalizes

Any per-life state your pack tracks — combat cooldowns, temporary buffs,
quest-run-scoped counters — should get the same treatment: an `OnDeath`
handler that explicitly decides, for each piece of state, whether it resets
(most gameplay-session state should) or persists (permanent unlocks like
`HAS_STRIDERS` deliberately do *not* reset here, since owning the upgrade
is meant to survive death exactly like a vanilla inventory item would if
Trailforge chose to keep it un-droppable). Writing that decision out
explicitly, one state variable at a time, is cheaper to get right than
discovering the gap in play-testing.
