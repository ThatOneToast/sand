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

Updated by #265's runtime-validation pass — see
`scripts/mc_validation/README.md` for the full tooling and exact
category-by-category evidence, and `examples/participant_audit/` for the
real (not simulated) datapack used. `examples/participant_audit/src/lib.rs`
is a typed, façade-only Sand datapack — every observed command goes through
public `sand` API (a typed `#[derive(SandStorage)]` evidence schema,
`EntityParticipant::execute_at`, `ItemSnapshot::copy_to`,
`ScoreRef::store_into`, `StorageField::copy_from_entity`) with zero
handwritten Minecraft command strings, enforced by
`sand/tests/example_imports.rs`'s `canonical_examples_use_typed_command_builders_not_raw_strings`
guard test alongside `examples/book_project`.

**Runtime-verified against a real, live Minecraft Java 26.2 server**
(downloaded from Mojang's own version manifest, `java -jar server.jar`,
not a mock):
- Real server startup with the actual merged-#266 automatic
  participant-plan-integrated datapack (`examples/participant_audit`)
  loaded — zero datapack load errors.
- Real `/reload` of that same pack over real RCON — zero reload errors,
  confirmed via `datapack list`.
- The generated functions actually execute without error on a real server
  (`function paudit:init` run over RCON; the audit storage schema
  initializes to the expected shape).
- A real `ServerPlayerEntity` **can** join a real 26.2 server: a
  purpose-built minimal protocol client
  (`scripts/mc_validation/minimal_join_client.py`) completed a genuine
  Handshake → Login → Configuration → Play sequence, confirmed by the
  server's own log (`<name> logged in with entity id N`, `<name> joined
  the game`) across multiple independent runs.
- `execute on attacker` relation existence and 1.20.2+ version gate
  (`sand-core/src/entity/relation.rs`, pre-existing, structurally tested).
- Item location NBT paths (`SelectedItem`, `Inventory[{Slot:-106b}]`, etc.)
  — long-documented, structurally stable vanilla tags (#229).

**Not runtime-verified — attempted, not achieved, in this validation
pass:**
- Player-triggered combat scenarios (a real player actually taking damage
  from a real or summoned entity, and the datapack's attacker/killer/weapon
  capture producing correct evidence). The minimal client's Play-phase
  connection is not yet stable enough to survive long enough for a scripted
  follow-up command to land reliably — see
  `scripts/mc_validation/README.md`'s "What is not proven, and exactly
  why" for the specific, honestly-documented gap (most likely one
  additional serverbound acknowledgement packet this very recent protocol
  version requires, not yet identified with confidence — no official
  protocol documentation exists yet for protocol version 776).
- Two independent concurrent player sessions — blocked by the same gap; a
  single stable session was not achieved, so two was not attempted.
- Whether `execute on attacker`'s "last attacker" memory is scoped exactly
  to the specific `EntityHurtPlayer`/`EntityKilledPlayer` criterion
  occurrence, vs. reflecting a slightly stale prior hit in edge cases
  (rapid multi-hit sequences, mixed melee/projectile damage in one tick) —
  an RCON-only (no player) mob-vs-mob reproduction of this was attempted
  and did not produce a trustworthy result within this pass either (entity
  selector behavior over RCON in this environment had its own
  unresolved quirks — see the PR history for the attempted commands); not
  claimed as evidence either way.
- Custom-data weapon snapshot correctness, empty-hand behavior, and
  inventory-mutation-after-capture isolation under real gameplay.

A complete, precise manual validation procedure for a human tester with a
real Minecraft 26.2 client is in `scripts/mc_validation/README.md`. Do not
treat the unverified items above as claims of failure — the reliability
levels in this document (`Correlated`, never `Exact`; `ExactSnapshot`,
never `Exact`) were already chosen conservatively enough that they do not
depend on the outcome of that verification, and #265 remains open pending
either a stabilized automated client or a completed manual pass.

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
