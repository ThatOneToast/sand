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

### A real bug this pass found and fixed

The previous (#280) pass got stuck on summoned entities becoming
unselectable before a full scenario could complete, and left the
`Correlated`/`ExactSnapshot` reliability claims resting on structural/export
evidence only. This pass got far enough past that (see "RCON-direct
scenario invocation" below) to actually inspect *correct* live-captured
data — and found that `EntityParticipant::execute_at`
(`sand-core/src/participant/reference.rs`) generated a bare
`execute at <selector> run <cmd>`. Vanilla's `execute at` only moves the
execution *position* — it never rebinds the executing entity (`@s`). Every
caller (every correlated-attacker/killer/bridge-killer capture in this
pack, built exactly per that method's own documented usage) builds `cmd`
referencing `@s` to mean "this participant" — so `@s` silently kept
resolving to the *caller's* own entity (the victim) instead, in every
single case. Concretely: a real summoned "attacker" zombie's combat
relation was captured correctly via `execute on attacker` (that part
always worked), but reading its UUID back out through the old `execute_at`
wrote the **victim's own UUID** into `attacker_uuid`/`killer_uuid`/
`bridge_killer_uuid` every time — never the attacker's. Every export-level
test asserted the exact (wrong) generated command string, so nothing
caught it; only live evidence with a genuine two-entity combat relation to
compare against did. Fixed to `execute as <selector> at @s run <cmd>` (see
that method's doc for the full before/after). This is the finding this
document exists to catch, and the primary reason `Correlated` reliability
claims below are now backed by matching-UUID live evidence rather than
"storage got populated."

**Runtime-verified against a real, live Minecraft Java 26.2 server**
(downloaded from Mojang's own version manifest, `java -jar server.jar`,
not a mock), re-run after the `execute_at` fix above:
- Real server startup with the current datapack — including
  `ComposedAttackerParent`/`ComposedAttackerChild`/`ComposedAttackerSibling`
  (#264's `inherit_entity` demonstration) — loaded, zero datapack load
  errors.
- Real `/reload` of that same pack over real RCON — zero reload errors,
  confirmed via `datapack list`.
- The generated functions actually execute without error on a real server
  (`function paudit:init` run over RCON; the audit storage schema
  initializes to the expected shape).
- **Correlated attacker** (`EntityDamagePlayerEvent` →
  `audit_on_hurt_by_entity_a`): a real combat "last attacker" relationship
  is established via vanilla's own `/damage ... by <entity>` between two
  freshly summoned, real entities, then the actual generated handler
  function is invoked directly over RCON — the identical commands a real
  advancement reward would call. The captured `attacker_uuid` in storage
  is compared byte-for-byte against the real summoned attacker's own
  `UUID` (queried independently) — genuinely matching, not merely
  "populated." See `scripts/mc_validation/run_audit.py`'s
  `correlated_attacker_rcon_direct_invocation`/
  `correlated_attacker_matches_real_attacker_uuid` checks.
- **Correlated killer** (`PlayerKillEvent` → `audit_on_killed`): identical
  technique and identical UUID-match proof for `killer_uuid`.
- **Advancement-bridge inheritance** (`PlayerKillEvent` bridge parent →
  `SpecialKillEvent` → `audit_on_special_kill`): identical technique,
  invoking the synthesized bridge entry function
  (`paudit:__sand_event_advancement_bridge/f6a08801`) directly instead of a
  plain handler — `bridge_killer_uuid` genuinely matches the real
  attacker's UUID, proving the inherited-participant plan is applied
  correctly around the bridge entry at real runtime, not just structurally.
- **Stale-state cleanup**: after a successful correlated-attacker
  invocation, the temporary `__sand_observed_<key>` tag the handler's setup
  creates and its own cleanup command removes was confirmed actually gone
  from the real entity afterward (`data get entity
  @e[tag=__sand_observed_f797eaf3]` → "No entity was found"), not merely
  present as text in the generated function.
- **Weapon snapshot, absent-mainhand branch** (`PlayerDamageEntityEvent` →
  `audit_on_hurt_entity`): a fresh entity with no `SelectedItem` NBT at all
  invoked directly produces `weapon_present: 0b`, genuinely confirmed live.
- The generated command *content* for the composed scenario was inspected
  directly (`examples/participant_audit/tests/deterministic_export.rs`'s
  `composed_scenario_*` tests): `audit_on_composed_parent`,
  `audit_on_composed_child`, and `audit_on_composed_sibling` all reference
  the exact same `__sand_observed_<key>` tag, and neither dependent emits
  its own `execute on attacker` — proof the inheritance is genuinely
  zero-cost, not a second capture that happens to agree. This is
  structural/export evidence, not a live-fire proof of the composed
  scenario's *runtime* correctness.
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

### RCON-direct scenario invocation: what it proves and what it does not

The checks above summon two real entities, use vanilla's own
`/damage ... by <entity>` to establish the exact combat relation
`execute on attacker` reads, then invoke the actual generated handler
function directly over RCON — the identical generated commands a real
advancement reward would call — rather than triggering the advancement
criterion itself (`entity_hurt_player`/`entity_killed_player`, which
require a real player victim; see "Not runtime-verified" below for why
that path is still blocked). This is real command execution against a
real running server, exercising the real implementation end-to-end
(participant setup, the correlated capture, cleanup) with a real,
independently-verifiable two-entity relation to check the result against —
**not** proof that a live player hit/kill fires the advancement criterion
that leads here. Getting this far required two harness fixes discovered
during this pass, both applied in `scripts/mc_validation/run_audit.py`:
- A ~0.5s settle delay between summoning an entity and the first command
  that selector-matches it. Empirically confirmed necessary and
  sufficient: a single-mcfunction batch of summon → immediate
  selector-check (same server tick, zero RCON round-trip latency) failed
  the check *every* time, while the same commands issued as separate RCON
  round-trips with this delay between them succeeded the large majority of
  the time — ruling out the RCON-round-trip/tick-boundary theory #280 had
  proposed for the "entities become unselectable" symptom; it is a
  same-tick selector-visibility lag, not a multi-tick persistence bug.
- Retrying the full summon → damage → invoke sequence with fresh entities
  (up to 6 attempts) when a `damage` call still reports "No entity was
  found" or "Target is invulnerable to the given damage type" (the latter
  is a freshly-spawned mob's own brief post-spawn invulnerability window,
  a genuine vanilla mechanic, not a bug) — in this pass, 1–2 attempts
  reliably sufficed once the settle delay above was in place.

**Not runtime-verified — attempted, not achieved, in this validation
pass:**
- Weapon snapshot's **present** branch (an actual captured item). Its
  backend is unaffected by the `execute_at` bug above (`@s` for this event
  is already the attacking player's own entity — a direct mainhand-slot
  NBT read, no relation traversal or `execute_at`/`execute on` indirection
  involved at all), but it could not be exercised with a real captured
  item in this pass: real command execution confirmed a summoned
  non-player entity cannot stand in for a real player here — vanilla's
  `data merge entity` validates merged NBT against the target's own data
  component schema, and `SelectedItem` is not a real component of a
  `zombie` (only of a player-controlled entity), so injecting one is
  silently dropped server-side (confirmed: `data merge entity` itself
  reports "Modified entity data of ...", but a follow-up
  `data get entity ... SelectedItem` reliably reports "Found no elements
  matching SelectedItem"). This is a hard requirement for a real player
  client, not a settle-timing issue like the entity-selectability gap
  above — retrying did not help. See
  `scripts/mc_validation/run_audit.py`'s `weapon_snapshot_present_branch`.
- Held-item snapshot is the same underlying backend as weapon snapshot
  (`ItemLocation::PlayerMainHand`/`PlayerOffHand`, `EventParticipantPlan::observe_held_item`)
  — same status as above, not separately re-attempted.
- Tracked-transition participant capture (#270's fix — a tracked-transition
  event's own direct participant plan being applied around its handler
  body) has structural/export coverage
  (`sand-core/tests/tracked_event_participant_setup.rs`) but no dedicated
  scenario in `examples/participant_audit` and was not added in this pass;
  live evidence for it remains open.
- The composed scenario's actual *firing*: it dispatches via
  `SandEventDispatch::tick().as_players()`, which requires a real player
  entity present as `@s` — the same stable-Play-phase-connection gap below
  blocks summoning one under scripted control, so no evidence exists (in
  either direction) for `compose_child_uuid`/`compose_sibling_uuid`
  actually landing correctly at real runtime, only that the generated
  commands reference the right tag structurally.
- Player-triggered combat scenarios (a real player actually taking damage
  from a real or summoned entity, driven through the real advancement
  criterion rather than a direct RCON handler invocation). The minimal
  client's Play-phase connection is not yet stable enough to survive long
  enough for a scripted follow-up command to land reliably — see
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
  not attempted in this pass; a genuine "does the same entity attack twice
  in one tick from two different sources" reproduction requires more setup
  than the single-relation checks added here.
- Custom-data weapon snapshot correctness and inventory-mutation-after-capture
  isolation under real gameplay (both require the same real-player-client
  precondition as the present-branch weapon check above).

A complete, precise manual validation procedure for a human tester with a
real Minecraft 26.2 client is in `scripts/mc_validation/README.md`. Do not
treat the unverified items above as claims of failure — the reliability
levels in this document (`Correlated`, never `Exact`; `ExactSnapshot`,
never `Exact`) were already chosen conservatively enough that they do not
depend on the outcome of that verification, and #265 remains open pending
either a stabilized automated client, a completed manual pass, or a real
player-client-driven weapon/held-item present-branch check.

## Participant propagation across event graph edges (#264)

Before #264, `EventContextCapabilities::for_event_with_participants` and
the `capabilities::full` propagation helpers (`propagate_after`,
`merge_after_any`/`merge_after_all`, `propagate_within`) computed what a
composed child event could *honestly promise* about an inherited
entity/item participant — but they were pure Rust-level bookkeeping with
zero call sites in the export pipeline, so a chained child's generated
commands never actually referenced a parent's captured binding. #264
closed that gap for the same-cycle case with a genuine command-level
mechanism — `EventParticipantPlan::inherit_entity`/`inherit_item` — rather
than by wiring the old capability-merge functions into codegen. An #274
audit confirmed those helpers had gained no production call sites in the
time since, so they were removed outright rather than left as dead public
API; `sand-core/src/participant/capabilities.rs` now only describes an
event's **subject** capability (`EventContextCapabilities::for_event` and
its `propagate_*`/`merge_*` helpers), which is a genuinely separate,
still-used concern from participant (entity/item) propagation.
`Event<E>::attacker()`/`.weapon()`/etc. resolve an inherited
declaration exactly like a directly-declared one for `AdvancementEvent`
handlers. As of #273/#280, plain `SandEvent` (tick/chain/tracked-dispatched)
handlers get the identical accessor sugar via the `SandEventParticipants`
blanket trait (`impl<T: SandEvent + 'static> SandEventParticipants for T`),
implemented as default trait methods on the concrete marker type itself
rather than a second `impl<E: SandEvent> Event<E>` block — this sidesteps
the trait-coherence conflict a second blanket `Event<E>` impl would hit
(nothing prevents a type implementing both `AdvancementEvent` and
`SandEvent`, as every built-in combat event already does) without a
supertrait migration. Both accessor surfaces are also now infallible: they
return the typed participant directly rather than a
`ParticipantAvailability<T>` wrapper, panicking internally (converted to a
structured `SAND-EVENT-PARTICIPANT` `sand build` diagnostic by the export
pipeline's panic-hook boundary, never a raw unhandled panic) if the plan
does not declare the requested role — see
`sand-core/tests/missing_participant_diagnostic.rs`. The old
`EventParticipantPlan::resolve`/`.resolve_item()` are now crate-private;
public code no longer has a reason to call them directly.

### Edge/role support matrix

| Edge type | Entity participant | Item snapshot | Reliability | Lifetime | Behavior |
|---|---|---|---|---|---|
| Direct declaration (no composition) | ✅ | ✅ | As declared (`Correlated`/`ExactSnapshot`) | `SynchronousDescendants` | Unchanged, pre-#264 baseline. |
| Single-parent `.after(...)`/`chain::<...>()` | ✅ `inherit_entity` | ✅ `inherit_item` | Unchanged from source (never upgraded) | `SynchronousDescendants` | Zero extra commands; resolves to the source's exact generated tag/storage path. Works through an arbitrary-depth chain of plain single-parent edges (grandchild may `inherit_*::<OriginalCapturer>` directly). |
| Same edge, but source only itself inherits (transitive) | ❌ Rejected | ❌ Rejected | — | — | Export diagnostic: "transitive inheritance is not supported... name the actual capturing ancestor directly." |
| `after_any` (multi-parent, disjunctive) | ❌ Rejected | ❌ Rejected | — | — | Export diagnostic: reached through `after_any`/multi-parent fan-in; #264 does not choose a winner. |
| `after_all` (multi-parent, conjunctive) | ❌ Rejected | ❌ Rejected | — | — | Same diagnostic path as `after_any` — any edge with more than one occurrence clause/parent is rejected uniformly. |
| `.while_(...)` (persistent condition) | ❌ Rejected (also structurally impossible — see below) | ❌ Rejected | — | — | A `while_` parent is required to have an empty `EventSetup` (#240 Phase 6 precedent), so it can never carry a plan to inherit from in the first place; the validator's diagnostic names this. |
| `.within(...)` (bounded cross-tick correlation) | ❌ Rejected (entity) | ✅ `inherit_item_within` (#272) | Preserved from source (`ExactSnapshot`/`ExactPostTrigger` qualifier unchanged) | `BoundedWindow` (new #272 lifetime — see below) | #272 adds an automatic bounded-item transport: the export pipeline copies the source's `ItemSnapshot` into per-subject entity NBT storage (`sand-core/src/participant/bounded_item.rs`) the moment the source occurs, reusing the existing `.within(...)` age counter for expiry — no hand-wired `ItemSnapshot::copy_to` needed for this case anymore (that API remains available for any other custom persistence a caller wants to build). Entity participants are still never safe to keep alive across a tick boundary with the current temporary-tag mechanism (see "Bounded entity decision" in the #264 PR description) and still always resolve unavailable/rejected — #272 explicitly does not attempt this; see its own PR description for the full rationale. |
| Advancement-bridge parent (`.after::<AdvancementEvent>()` with no direct handler) | ✅ `inherit_entity` (#269) | ✅ `inherit_item` (#269) | Unchanged from source | `SynchronousDescendants` | Fixed in #269/#280: the bridge parent's own `participants()` plan is now applied directly around its synthesized entry (setup after revoke, before every dependent; cleanup after every synchronous descendant) — see `sand-core/tests/event_chain_advancement_bridge_nested_siblings.rs` for exact-output proof (siblings, a nested grandchild, and ordering). A grandchild reached through another same-cycle child may still `inherit_*` directly from the original bridge parent (multi-hop, not transitive). Real-server validation for this specific scenario (`PlayerKillEvent -> SpecialKillEvent`) via `scripts/mc_validation/run_audit.py`'s `advancement_bridge_rcon_direct_invocation`/`advancement_bridge_matches_real_attacker_uuid` checks: a real, non-mocked firing of the synthesized bridge entry function against a real, independently-verified combat attacker relationship, with `bridge_killer_uuid` confirmed to genuinely match the real attacker's UUID (see `docs/testing/participant-role-evidence.md`'s "What has and has not been runtime-verified" and `scripts/mc_validation/README.md`'s "#265 (this pass)" section for the full before/after, including a real `execute_at` bug this uncovered and fixed). |
| Tracked-transition parent (#263) | ❌ Rejected (nothing to inherit) | ❌ Rejected | — | — | Same-cycle graph-parent bridging for tracked-transition `SandEvent`s remains unsupported and is unrelated to #270: #270 fixed a tracked-transition event's own *direct* participant plan being silently ignored by its dispatch backend (now applied around its own handler body — see `sand-core/tests/tracked_event_participant_setup.rs`); using a tracked event as a graph parent for another event's inheritance is still rejected by `discover()` (`sand-core/src/events/graph.rs`), unchanged. |

Every rejection above is a real export-time diagnostic (`sand-core/src/compiler/export/participant_transport.rs`), not a silent downgrade — see `sand-core/tests/event_chain_participant_inheritance_diag_{after_any,within,transitive}.rs` for end-to-end proof each one actually surfaces through the real export pipeline.

## Bounded item-snapshot transport through `.within(...)` (#272)

Extends the #264 propagation model with exactly one new case: an
`EventParticipantPlan::inherit_item_within::<Source>(role, hand, window)`
declaration on a `.within::<Source>(window)`-bounded child. Distinct from
every other row in the matrix above because it is **owned storage** (a
persistent, per-subject copy), not a same-cycle borrowed reference — the
same distinction #264's own PR description already draws when explaining
why bounded entity inheritance is not attempted.

- **Storage**: per-subject entity NBT on the subject entity (`data ...
  entity @s ...`), not command storage — command storage has no native
  per-player keying, so a single shared path would let a later occurrence
  (same player or a different one) overwrite an unexpired copy before a
  bounded child reads it. Entity NBT is scoped to the entity itself, making
  cross-player leakage structurally impossible rather than merely unlikely.
  See `sand-core/src/participant/bounded_item.rs`'s module doc.
- **Freshness**: reuses the *existing* per-subject `.within(...)` age
  objective (`se_{key}_wa`) unmodified — no second, parallel age-tracking
  mechanism was introduced.
- **Replacement**: every source occurrence unconditionally resets the
  storage to explicit absence, then presence-gated copies and marks —
  identical reset-then-conditionally-write shape to
  `ItemSnapshot::capture`, so a bounded child can never observe a torn
  mix of an old and new item.
- **Expiry**: cleared once the source's age counter first exceeds the
  longest window any consumer declared against the same
  `(source, role, hand)` triple.
- **Consumption**: deliberately does **not** eagerly clear on a successful
  read — the stored value is a caller-owned copy, not a scarce
  single-owner resource, so multiple sibling bounded children may read the
  same occurrence's copy within the window without racing each other's
  cleanup. Only expiry and replacement ever clear it (plus
  `BoundedItemSnapshot::reset_commands` for an explicit/manual reset path).
- **New lifetime**: `ParticipantLifetime::BoundedWindow` — wider than
  `EventCycle` (spans multiple ticks) but still an explicit, bounded,
  Sand-managed lifetime, never silently promoted to a durable one.
- **Entity participants unaffected**: the bounded-entity-inheritance
  diagnostic `find_borrowable_ancestor_path` produces is untouched — #272
  is item-only, validated by a separate function
  (`participant_transport::validate_bounded_item_transport`) so the two
  validators' opposite soundness conditions (same-cycle borrowing must
  *not* cross a `.within(...)` edge; bounded item transport must cross
  *exactly* one, with a matching window) never have to share one code path.

See `sand-core/tests/event_chain_bounded_item_transport.rs` for exact-output
proof of the generated persist/expire commands, subject-isolation structure,
and export determinism, and `sand-core/src/participant/plan.rs`/
`bounded_item.rs`'s unit tests for the declaration/resolution/command-shape
coverage. Minecraft 26.2 validation status: see the PR description — this
document is updated once real load/reload evidence for a pack exercising
this path exists (structural export/JSON coverage above is not itself
runtime evidence).
