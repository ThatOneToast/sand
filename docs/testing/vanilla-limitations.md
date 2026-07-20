# Vanilla Minecraft limitations

Constraints of vanilla Java Edition itself — no amount of Sand typing can
remove them. This is the internal, evidence-linked companion to the book's
short [Vanilla Limitations](../../book/src/reference/vanilla-limitations.md)
page; read that page for the user-facing summary and this one when you need
the supporting source/test references.

- **Storage is global.** Command storage NBT is not namespaced per player by
  the game. Dynamic per-player storage keys are a Sand/authoring convention,
  not a vanilla guarantee. See `storage`-related typed APIs.

- **Advancement triggers are coarse.** They do not cover every gameplay
  action, and most triggers fire a condition check rather than an
  event-with-data — there is no rich payload to read off a trigger.
  Combine a trigger with typed state/storage reads in the triggered function
  to recover context. Evidence: `sand-components/src/advancement/trigger_coverage.rs`
  (`EventWrapperStatus`).

- **No damage-taken event.** Vanilla has no "damage taken" event with a
  payload; damage tracking is a cumulative-scoreboard-stat approximation
  (total damage taken since last reset), not a per-hit event. Evidence:
  `sand-core/src/systems/damage.rs`.

- **No arbitrary physics.** Vanilla has no arbitrary entity physics/velocity
  API reachable from datapacks. Push/launch/speed-boost/slow helpers are
  built from teleports and potion-effect-like mechanics, not free-form
  physics.

- **No proximity trigger.** Interaction entities (armor stands, interaction
  hitboxes) react to right-click/attack, not proximity. There is no vanilla
  "on approach" trigger without tick-polling distance checks.

- **No shield-block/axe-disable event.** There is no dedicated vanilla event
  for "successful shield block" or "axe disabled a shield" — these must be
  approximated (e.g. via damage-amount heuristics), not detected exactly.

- **No client respawn packet.** Vanilla datapacks do not receive the client
  respawn packet. Sand observes a death, then infers the player is active
  again when `minecraft.custom:minecraft.time_since_death` advances from its
  death-reset value of zero. This is reliable at tick boundaries, including a
  held-open death screen and immediate respawn, but a complete respawn
  followed by another death between two Sand ticks can coalesce into one
  observed lifecycle. Evidence: `sand-core/src/component.rs`,
  `sand-core/tests/respawn_lifecycle_export.rs`.

- **`.within(...)` age counters only advance for online players.** The
  generated age update runs under `execute as @a`, which only iterates
  currently-online players, so a bounded correlation window pauses (does not
  advance) while a player is offline and is not reset by disconnect/
  reconnect or `/reload` — a returning player resumes aging from wherever it
  paused. Treat `.within(...)` as an approximation of recency under load,
  not wall-clock recency, when players may disconnect mid-window. Evidence:
  `sand-core/src/component.rs` (bounded age-counter maintenance),
  `sand-core/tests/event_chain_within_export.rs`.

- **Participant roles without a vanilla relation/NBT read path are
  `Unavailable`, not guessed.** Victim, direct attacker, interacted entity,
  projectile, and ammunition have no credible vanilla-exposed evidence for
  any current event family — see
  [`participant-role-evidence.md`](participant-role-evidence.md) for the
  full role-by-role audit, including why `execute on origin` (a real,
  implemented relation) still isn't wired to any event today.
