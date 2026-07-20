# Participant role evidence audit (#230)

Role-by-role audit of what vanilla Minecraft Java actually exposes for
combat, interaction, and item participant context, and what backend (if
any) Sand implements as a result. This is the internal, evidence-linked
companion to the book's [Vanilla Limitations](../../book/src/reference/vanilla-limitations.md)
page and `sand-core/src/participant/`'s module docs — read those for the
user-facing summary and typed API, this page for the source evidence and
the full role support table.

Canonical target: Minecraft Java 26.2. Evidence sources used: Sand's own
already-implemented and tested vanilla-relation traversal
(`sand-core/src/entity/relation.rs`), the advancement trigger/reward
contract (every reward function's `@s` is the triggering **player**, never
a non-player entity — this is a structural fact of how advancement rewards
dispatch, not something that varies per trigger), and Sand's existing item
snapshot/location machinery (#229). No live 26.2 server was available in
this environment — see "What has and has not been runtime-verified" below.

## Role support matrix

| Role | Event families | Backend | Reliability | Evidence |
|---|---|---|---|---|
| Subject player | all advancement/tick-backed events | direct (`Event::player`) | `Exact` | The triggering/polled mechanism hands `@s` over directly. |
| Attacker | `EntityDamagePlayerEvent` | `execute on attacker` (`observe_correlated_attacker`) | `Correlated` | `Relation::Attacker`, vanilla 1.20.2+ relation query, single-valued. |
| Killer | `PlayerKillEvent` | same mechanism, `Killer` role | `Correlated` | Same relation; `PlayerKillEvent`'s `@s` is the victim, so the killer is reached identically to `EntityDamagePlayerEvent`'s attacker. |
| Weapon | `EntityKillEvent`, `PlayerDamageEntityEvent` | mainhand item snapshot (`observe_weapon`) | `ExactSnapshot` | `@s` is the player who dealt the damage/kill for these two events — their own mainhand is directly addressable, no relation traversal needed. |
| Direct attacker | none | **Unavailable** (`NotSuppliedByTrigger`) | — | No `execute on <relation>` distinguishes "direct causing entity" (e.g. an arrow) from the credited attacker — vanilla's damage-source direct/causing distinction is not exposed as a queryable relation, only as NBT on a `DamageSource` compound Sand has no verified read path for. |
| Victim | `PlayerDamageEntityEvent` (attacker's own player-hit-entity events) | **Unavailable** (`NotSuppliedByTrigger`) | — | `@s` for these events is already the attacker; there is no `execute on victim`-style relation from the attacker back to who they just hit. |
| Interacted entity | interaction events (`InteractWithEntityEvent`, etc.) | **Unavailable** (`NotSuppliedByTrigger`) | — | Advancement reward functions bind `@s` to the player, never the interacted entity; no relation connects a player to "the entity it just interacted with." |
| Hand (main/off) | any player-subject event | `ItemLocation::PlayerMainHand`/`PlayerOffHand` (exact NBT paths) | `Exact` (addressing), `ExactSnapshot` (captured item) | Always-valid, version-independent NBT paths on `@s` — see `sand-core/src/item/location.rs`'s module doc. Not correlation-dependent at all. |
| Held item | any player-subject event | `EventParticipantPlan::observe_held_item` | `ExactSnapshot` | Same as Hand — a specific hand slot's item snapshot. |
| Projectile | none | **Unavailable** (`NotSuppliedByTrigger`) | — | No player-subject advancement event binds `@s` to a projectile entity; `execute on origin` (see below) requires `@s` to already be the projectile. |
| Projectile origin/shooter | none (not wired to any current event) | investigated, not implemented | — | `Relation::Origin` (`execute on origin`, 1.21.2+) is a real, already-implemented Sand relation — "the entity that fired/summoned this entity." It answers this role correctly **if** `@s` is already the projectile. No current Sand event family scopes `@s` to a projectile entity (all combat/interaction events are player-subject), so there is nothing to wire it into today. Adding an entity-scoped tick-polled projectile event family is a concrete, scoped future improvement (see follow-up issue), not something to fake from a player-subject event. |
| Ammunition | none | **Unavailable** (`NotSuppliedByTrigger`) | — | No relation or NBT read path from a player-subject event to "the ammunition item consumed to fire a projectile" was identified with credible evidence. |

Any role not listed with a backend resolves `Unavailable(NotApplicable)` via
`Event::entity`/`Event::item` for event types that don't declare it in
their `participants()` plan at all (the vast majority of events — combat
plans are only declared on the four event types in the table above).

## Why `execute on attacker`/`execute on origin` are `Correlated`, not `Exact`

Both are genuine, direct vanilla relation queries — not heuristics — but
Sand still reports `Correlated`: there is no verified guarantee that
vanilla's internal relation memory is updated synchronously with, and
scoped exactly to, the specific event that fired the advancement criterion
the observation is embedded in (as opposed to reflecting an earlier
interaction in the same tick). `Exact` is reserved for values the
triggering mechanism *itself* directly hands over (the reward function's
own `@s`); a relation traversal Sand performs itself is one step removed
from that guarantee. See `sand-core/src/participant/observation.rs`'s
module doc for the full reasoning.

## Why Weapon/Held-item are `ExactSnapshot`, not `Exact`

Item participants are always copied into Sand-owned storage
(`ItemSnapshot::capture`), never referenced live — see
`sand-core/src/item/snapshot.rs`'s module doc. `Exact` in Sand's reliability
model is reserved for live, re-queryable references (`Exact` ranks above
`ExactSnapshot` for exactly this reason — a live reference can still be
traversed with further commands; a snapshot is deliberately frozen data).

## What has and has not been runtime-verified

**Runtime-verified (already shipped, tested against real command syntax
and version gating logic, not a live server in this change):**
- `execute on attacker` relation existence and 1.20.2+ version gate
  (`sand-core/src/entity/relation.rs`, pre-existing).
- Item location NBT paths (`SelectedItem`, `Inventory[{Slot:-106b}]`, etc.)
  — long-documented, structurally stable vanilla tags (#229).

**Not runtime-verified as part of this change (structural/export-level
tests only):**
- Whether `execute on attacker`'s "last attacker" memory is scoped exactly
  to the specific `EntityHurtPlayer`/`EntityKilledPlayer` criterion
  occurrence, vs. reflecting a slightly stale prior hit in edge cases
  (rapid multi-hit sequences, mixed melee/projectile damage in one tick).
- Two-player isolation of the correlated-attacker and held-item-snapshot
  backends under genuinely concurrent damage events (structural evidence
  only — see the module docs' "multiplayer safety" sections for the
  `execute as @a` sequential-per-player argument, which is the same
  argument already relied on elsewhere in Sand, not new verification).
- Real Minecraft 26.2 startup/`/reload` with a pack using these participant
  plans.

Do not treat the "not runtime-verified" items as claims of failure — they
are exactly what real-server validation (tracked as follow-up) would
confirm or correct. The reliability levels above (`Correlated`, never
`Exact`) are already chosen conservatively enough that they do not depend
on the outcome of that verification.

## Scope note: capability propagation through composition

`EventContextCapabilities::for_event_with_participants` and the
`capabilities::full` propagation helpers (`propagate_after`,
`merge_after_any`/`merge_after_all`, `propagate_within`) compute what a
composed child event could *honestly promise* about an inherited
entity/item participant, applying the same reliability/lifetime
degradation rules the subject-only versions already used. They are
capability **bookkeeping** — the export pipeline's generated commands do
not yet re-bind a parent's observed participant (its tag/storage path) into
a same-cycle child's own scope, so a chained/composed child cannot
currently call `Event::entity(role)` and reach a parent's declared
participant. Wiring real command-level propagation across chain/compose
graph edges is tracked as focused follow-up scope.
