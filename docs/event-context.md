# Participant reliability and event context capabilities

This page documents `sand_core::participant`: the typed reliability,
availability, lifetime, and context-capability model for richer event
participants (attacker, victim, interacted entity, ...), plus (as of Phase
9) Sand's first real participant-recovery backend. Phase 8 established the
type system; Phase 9 adds exactly one observation mechanism —
[`observe_correlated_attacker`](#phase-9-correlated-attacker-observation) —
on top of it. There is still no interacted-entity correlation, no
projectile-owner recovery, and no proximity/heuristic (`Inferred`)
observer anywhere in this module. See `ai/known-limitations.md`
`LIM-CTX-001`..`LIM-CTX-005`, `LIM-VAL-009`, `LIM-VAL-010` for what remains
architecture-only or unverified.

## Root problem

Sand's event contexts today reliably expose one thing: the triggering
player subject (`Event::player()`/`.subject()`, always `@s`). Richer
participants — an attacker, a victim, an interacted entity, a projectile's
owner — differ enormously in evidence strength: a damage event may expose
a victim exactly but only correlate an attacker; a projectile event may
identify the projectile but not its original item; some participants
cannot be recovered from vanilla at all. A single unqualified
`EntityContext` or `Option<Entity>` would hide those differences and
invite mechanics built on evidence that isn't actually there. This phase
adds the vocabulary to keep that honest before any recovery backend
exists.

## Participant roles

```rust
use sand_core::participant::{EntityParticipantRole, LocationParticipantRole, ItemParticipantRole};

EntityParticipantRole::Subject;   // the event's own player/entity subject
EntityParticipantRole::Attacker;
EntityParticipantRole::DirectAttacker;
EntityParticipantRole::Victim;
EntityParticipantRole::Killer;
EntityParticipantRole::Target;
EntityParticipantRole::InteractedEntity;
EntityParticipantRole::Projectile;
EntityParticipantRole::ProjectileOwner;

LocationParticipantRole::EventBlock; // the one location role with existing evidence (placed_block, item_used_on_block)

// Reused directly from Phase 7 — no second competing item-role enum:
let _: ItemParticipantRole = sand_core::item::ItemRole::Weapon;
```

Roles are deliberately minimal — every variant has either an existing
vanilla mechanism backing it (documented per-variant in
`sand-core/src/participant/role.rs`) or a clear Phase 9 use. Item roles
reuse Phase 7's `ItemRole` (`UsedItem`, `Weapon`, `Tool`, `Ammunition`,
`DroppedItem`, plus `ProjectileItem`/`EquippedItem`) rather than
duplicating it — `ItemParticipantRole` is a type alias for it.

## Reliability

```rust
use sand_core::participant::ParticipantReliability;

ParticipantReliability::Unavailable;
ParticipantReliability::Inferred;    // heuristic query, may be ambiguous
ParticipantReliability::Correlated;  // bounded observation, stated constraints
ParticipantReliability::ExactSnapshot; // copied at the event boundary, immutable
ParticipantReliability::Exact;       // live, authoritative reference (e.g. @s)
```

One reliability vocabulary covers every kind of participant — entities,
players, items, locations. Variants are declared weakest-first so the
derived `Ord` doubles as a strength ordering:
`reliability.meets(required)` is exactly `reliability >= required`. `Exact`
outranks `ExactSnapshot`: a live reference can still be traversed/re-queried
(`execute on attacker run ...`), while a snapshot is deliberately frozen
data — both are strong evidence, they just answer different questions.

Only `Exact` (for the event's own subject) and `Correlated`/`Unavailable`
(via the Phase 7 mapping below) are produced anywhere in the current
codebase. `Inferred` and non-subject `Exact` are defined but unused —
Phase 9 territory.

### Phase 7 item reliability mapping

Phase 7's `SnapshotReliability` is untouched (no breaking change) and maps
additively into `ParticipantReliability`:

| `SnapshotReliability` | `ParticipantReliability` | `ItemEvidenceQualifier` |
|---|---|---|
| `Exact` | `ExactSnapshot` | `CapturedBeforeVanillaMutation` |
| `ExactPostTrigger` | `ExactSnapshot` | `CapturedAtFirstSandControl` |
| `Correlated` | `Correlated` | — |
| `Unavailable` | `Unavailable` | — |

Both of Phase 7's "exact" levels map to the same `ExactSnapshot` umbrella
level (items are always copied into storage, never referenced live — so
neither ever qualifies for `ParticipantReliability::Exact`), but the
distinction between them is **not** flattened away: call
`.item_evidence_qualifier()` on the original `SnapshotReliability` to get
it back. `EventItem`/`ItemRole` (Phase 7's integration seam for #230) are
unchanged; `ItemParticipantRole` simply re-exports `ItemRole`.

## Availability

```rust
use sand_core::participant::{ParticipantAvailability, ParticipantUnavailableReason};

let attacker: ParticipantAvailability<()> =
    ParticipantAvailability::Unavailable(ParticipantUnavailableReason::AmbiguousCandidates);
```

`Option<T>` alone can't distinguish "this event's semantics make this
optional" from "Sand/vanilla cannot supply this at all." The nine
`ParticipantUnavailableReason` variants (`NotSuppliedByTrigger`,
`UnsupportedVersion`, `UnsupportedBackend`, `AmbiguousCandidates`,
`CorrelationExpired`, `NoMatchingObservation`, `NotApplicable`,
`ItemSourceAlreadyMutated`, `LifetimeExpired`) are a small, stable, public
set — exporter-internal errors never leak through this type. `Option<T>`
remains fine *inside* an already-`Available` value for genuine
event-semantic optionality (e.g. "no offhand item this occurrence").

## Lifetime

```rust
use sand_core::participant::ParticipantLifetime;

ParticipantLifetime::Invocation;            // the generated function call that captured/bound it
ParticipantLifetime::SynchronousDescendants; // + same-cycle synchronous child calls
ParticipantLifetime::EventCycle;             // the coordinator's current pass (e.g. .within(...) state)
```

Participant references are generated-command execution concepts, not
Rust-owned Minecraft entities — Rust's borrow checker cannot enforce how
long a `@s` binding stays meaningful across generated `function` calls, so
this is a documented contract, not a compiler-enforced one (the same
honesty `ItemSnapshot`'s own lifetime doc already commits to).
`captured.covers(needed)` is `captured >= needed`: a reference captured at
`Invocation` does not cover a `SynchronousDescendants` use; one captured at
`EventCycle` covers everything narrower.

## Typed participant references

```rust
use sand_core::participant::{PlayerParticipant, EntityParticipant, EntityParticipantRole, ParticipantLifetime};
use sand_commands::selector::SingleEntity;

let subject = PlayerParticipant::subject(); // @s, Exact, Invocation
assert!(subject.require_exact().is_ok());

let attacker = EntityParticipant::correlated(
    SingleEntity::raw("@e[tag=candidate,limit=1]"),
    EntityParticipantRole::Attacker,
    ParticipantLifetime::Invocation,
);
assert!(attacker.require_exact().is_err()); // correlated never satisfies exact
```

`PlayerParticipant`/`EntityParticipant` are command-building handles, not
live runtime data. The **only** exact constructor either type provides is
`subject()` — the event's own triggering/polled player, the one case Sand
can honestly mark `Exact` today. There is no API path to construct a
non-subject participant claiming `Exact`: `EntityParticipant::correlated`/
`::inferred` are the only other constructors, and they hard-code their own
weaker reliability. `require(floor)`/`require_exact()` return a
`ParticipantReliabilityError` naming the role, requested reliability, and
supplied reliability when a floor isn't met.

## Event context capabilities

```rust
use sand_core::participant::EventContextCapabilities;

let caps = EventContextCapabilities::for_event::<sand_core::events::PlayerSneakEvent>();
assert_eq!(caps.subject, sand_core::participant::SubjectCapability::EXACT_PLAYER_INVOCATION);
assert!(caps.entities.is_empty()); // no participant-recovery backend exists yet
```

`EventContextCapabilities` is a deterministic, `'static`-only descriptor
(`Copy` enums and `&'static` slices — no `TypeId`-derived identity, safe
for generic event monomorphizations, comparable and orderable). It answers
"what can this event type promise," never holding a runtime value.
`EventContextCapabilities::for_event::<E: SandEvent>()` derives it
structurally from `E::dispatch()`:

- `AdvancementTrigger`/legacy `TickCondition` dispatch → exact player
  subject, invocation lifetime.
- Structured `SandEventDispatch::tick()` → exact player subject iff the
  declared `TickScope` has a player subject (`TickScope::Players` and
  Phase 6's `TickScope::AdvancementPlayer` both qualify).
- `SandEventDispatch::chain()`/`compose()` → **not resolved generically.**
  A `ChainEventDispatch`'s parent(s) are identified by type-erased
  function-pointer factories specifically so the parent marker type never
  needs instantiating (see `sand-core/src/events/graph.rs`
  `OccurrenceParent`) — that means `for_event` cannot call
  `for_event::<Parent>()` from inside an already-erased dispatch value.
  `for_event` returns `EventContextCapabilities::NONE` for a chained event
  type. A caller who knows the concrete parent must call
  `for_event::<Parent>()` themselves and combine it with the propagation
  functions below. This is a real, documented limitation
  (`LIM-CTX-001`), not an oversight.

`.validate()` rejects a descriptor that declares the same role twice in
`entities`/`items`/`locations`.

## Graph propagation and merging

Every `SubjectCapability` (`reliability` + `lifetime` + `scope`) has a
pure propagation/merge function in `sand_core::participant::capabilities`:

| Composition | Function | Rule |
|---|---|---|
| `.after::<Parent>()` (single parent, including Phase 6 advancement bridges) | `propagate_after` | Child inherits the parent's reliability/scope; lifetime widens to at least `SynchronousDescendants` (the child runs one level deeper than the parent's own invocation). |
| `.after_any::<G>()` | `merge_after_any` | All parents in the group must share the same subject scope (or the merge is rejected) — the reliability of the result is the *weakest* in the group, since which parent actually fired isn't statically known. |
| `.after_all::<G>()` | `merge_after_all` | Same rule as `after_any` for the subject — Phase 8 does not attempt to combine parent-specific participant fields (e.g. two different attacker contexts), since that would require unioning fields, which is explicitly disallowed. |
| `.while_::<E>()` | `propagate_while` | Identity — a persistent condition never adds or removes participant capability. |
| `.when(...)`/`.unless(...)` | `propagate_when_unless` | Identity. |
| `.within::<E>(...)` | `propagate_within` | Downgrades reliability to at most `Correlated` and lifetime to `EventCycle` — a bounded correlation crosses tick boundaries, so anything captured at the original synchronous invocation is gone by the time the condition is later observed true; only the tracked subject itself (read back from persisted state) remains meaningful, and even that is no longer `Exact`/`Invocation`-scoped. |

All merge functions are order-independent (`merge_after_any(&[a, b]) ==
merge_after_any(&[b, a])`) and reject an empty parent set or incompatible
subject scopes with a `ContextMergeError` rather than guessing.

**Not implemented**: automatic resolution of a `ChainEventDispatch`'s own
parent capabilities (see above) and combining non-subject
entity/item/location capability lists across parents (nothing produces
non-empty lists yet, so there is nothing to merge in practice — the merge
functions exist and are tested at the `SubjectCapability` level only).

## Event-family capability audit

`sand-core/tests/participant_context_capability_audit.rs` is a
table-driven audit of Sand's currently supported `SandEvent`-backed
families: player join/state-tick events, death/respawn-adjacent events,
kill/damage advancement triggers, item-used events, placed-block events,
interaction events, projectile-adjacent events, and ride/vehicle events.
**Every one of them resolves to an exact player subject with empty
entity/item/location capability lists** — none currently declare a real
attacker, victim, interacted entity, or projectile-owner capability,
because no recovery backend exists. This is the honest current state, not
a placeholder; a future Phase 9 change that starts populating those lists
for one of these types will show up as a visible diff in that test file.

## Version awareness

Capability descriptors carry an optional `min_version: Option<(u32, u32,
u32)>` per entity/item/location capability entry (compared via
`McVersion::new(major, minor, patch)`), so a future Phase 9 capability that
only exists on newer profiles can be declared without fabricating
availability on older ones. Nothing in Phase 8 populates a non-`None`
value yet — there is no version-gated participant capability to declare
until Phase 9 adds one.

## Compatibility

Additive only: `SandEvent`, `Event<T>`, `EntityContext`, and every
existing event definition are unchanged. `sand_core::item::SnapshotReliability`
is unchanged (no rename, no new variant) — the Phase 8 mapping is a new
inherent method on top of it. Nothing in `sand_core::participant` is
wired into `#[event]` codegen or the tick coordinator; it is a standalone,
directly-usable type/metadata layer, the same shape Phase 7 shipped
`ItemSnapshot::capture()` as.

## What Phase 8 does not do

- Does not implement attacker observation, victim observation beyond what
  is already exact (none currently is), interacted-entity correlation,
  projectile-owner recovery, ammunition correlation, or nearest-entity
  guesses.
- Does not add a bounded observation tracker for participant recovery.
- Does not automatically snapshot items for every event.
- Does not retain arbitrary event payloads or persist participant
  references across ticks beyond what `.within(...)`'s existing bounded
  correlation state already does (and that state is not treated as a
  participant reference here — see `propagate_within` above).
- Does not wire any of this into `#[event]`/macro-generated handler
  signatures — that remains a manually-composed, directly-callable API
  layer, exactly like Phase 7's `ItemSnapshot::capture()`.

## Phase 9: correlated attacker observation

Phase 9 adds `sand_core::participant::observation`: exactly one
participant-recovery mechanism, `observe_correlated_attacker`, built on
vanilla's `execute on attacker` relation (already wired up as
`EntityContext::attacker()`/`Relation::Attacker` since before this phase).

```rust
use sand_core::entity::EntityContext;
use sand_core::entity::kind::PlayerKind;
use sand_core::participant::{EntityParticipantRole, ObservationSchema, observe_correlated_attacker};
use sand_core::version::{MinecraftVersion, VersionProfile};

let profile = VersionProfile::resolve(&MinecraftVersion::parse("1.21.4").unwrap()).unwrap();
let ctx: EntityContext<PlayerKind> = EntityContext::default();

let commands = observe_correlated_attacker(
    &ctx,
    &profile,
    ObservationSchema::new("mypack:observations", "MyDamageEvent"),
    EntityParticipantRole::Attacker,
    |observation| {
        // Commands here run with the attacker (if any) bound; guard on
        // `observation.is_present()`/`.is_absent()` as needed.
        vec!["say attacker observed".to_string()]
    },
)?;
```

Embed the returned commands into a `SandEvent`'s handler body (for an
advancement-backed event like `EntityDamagePlayerEvent`/`PlayerKillEvent`)
or `EventSetup::pre_observation` (for a tick-backed event) — the same
manual-composition pattern Phase 7 shipped `ItemSnapshot::capture()` as.
Not auto-wired into `#[event]`/tick-coordinator codegen.

### Why only `execute on attacker`

`execute on attacker` is single-valued by vanilla's own construction (at
most one entity, never a set), so there is no "multiple credible
candidates" ambiguity to police — unlike a proximity-based guess, which
Phase 9 deliberately does not implement. This is the entire reason Phase 9
could implement this one mechanism honestly without inventing a selection
policy: the evidence itself has no ambiguity dimension.

### Reliability: always `Correlated`, never `Exact`

Even though `execute on attacker` is a direct vanilla relation query (not
a heuristic), every observation this module produces is
`ParticipantReliability::Correlated`. `Exact` is reserved for references
the *triggering mechanism itself* hands over synchronously (the
advancement reward function's own `@s`); the attacker is reached through
an additional relation traversal Sand performs itself, and there is no
real-server evidence proving vanilla's "last attacker" memory is updated
in lockstep with the specific damage event that fired the criterion this
observation is embedded in (see `LIM-VAL-010`). This is a deliberate,
conservative default — not a placeholder pending a future upgrade to
`Exact` "once tests pass." Per Phase 9's contract, tests passing is never
sufficient justification to upgrade reliability.

### Generated commands and lifetime

```mcfunction
data modify storage mypack:observations obs.<key>.present set value 0b
execute on attacker run data modify storage mypack:observations obs.<key>.present set value 1b
execute on attacker run tag @s add __sand_observed_<key>
say attacker observed
tag @e[tag=__sand_observed_<key>] remove __sand_observed_<key>
```

Each `execute on attacker run <command>` line runs a single command
inline — deliberately *not* routed through
`EntityContext::attacker().if_present(...)`'s multi-command
dynamic-function-wrapping (which registers a separate generated function
from inside `SandEvent::setup()`; that registration point relative to the
exporter's dynamic-function drain is not guaranteed deterministic across
repeated exports in the same process, and this module needs only two
single-command lines, so the wrapping was never necessary). Presence
is checked via `.is_present()`/`.is_absent() -> Condition` (identical
`StorageExists` pattern to `ItemSnapshot`), never encoded as an implicit
"empty selector" the caller has to notice on their own. The participant
handle (`.participant() -> EntityParticipant`) addresses the tagged entity
by that unique tag — never a bare `@e[type=...]` query that could match
the wrong entity — and is valid for
`ParticipantLifetime::SynchronousDescendants`: the tag is added and
removed within one straight-line generated sequence (reset → mark/bind →
caller body → cleanup), so it survives through the caller's body and any
same-cycle synchronous children reached from within it, and is gone by the
time the sequence returns. There is no tick-window to refresh or expire —
this is an instantaneous relation query, not a bounded multi-tick
correlation like `.within(...)`; conflating "this event fired recently"
with "this entity was observed recently" is exactly what this design
avoids by not reusing `.within(...)`'s infrastructure here.
`sand-core/tests/participant_attacker_observation_export.rs` proves this
ordering end-to-end through the real export pipeline.

### Multiplayer safety

One deterministic, non-per-player storage path per `event_label` — safe
under the identical `execute as @a`-is-single-threaded-per-player argument
`ItemSnapshot`'s module doc gives in full. The identity tag is unique per
call site (derived from `event_label`), so two *different* event types
never collide; two observations for the *same* `event_label` nested inside
one synchronous call tree (before the first's cleanup runs) are not
supported — see `LIM-CTX-005`.

### Target-version support

`execute on attacker` requires Minecraft 1.20.2+.
`observe_correlated_attacker` returns `ObservationError::UnsupportedVersion`
(a build-time diagnostic, before any commands are generated) rather than
silently degrading or fabricating availability on an older profile.

### False-positive / false-negative risk

- **False positive risk**: vanilla's "last attacker" memory could in
  principle reflect an earlier hit than the one that triggered this
  specific criterion (e.g. two hits in rapid succession), reported as the
  same attacker when they weren't. This is the core reason for
  `Correlated` rather than `Exact` — unverified without a live server.
- **False negative risk**: if the victim entity has no recorded attacker
  (e.g. environmental damage, or the "last attacker" memory expired/was
  cleared by vanilla), the observation is honestly absent
  (`.is_absent()` holds) rather than falling back to a guess.

### What Phase 9 does not do

- Does not implement victim correlation, interacted-entity correlation, or
  projectile-owner recovery — evidence for those was judged too weak or
  architecturally out of reach without inventing a new event pathway (see
  `ai/known-limitations.md` `LIM-VAL-010` for the reasoning per role).
- Does not add a proximity/heuristic (`Inferred`) observer.
- Does not automatically apply to any built-in `SandEvent` — a caller
  embeds `observe_correlated_attacker` themselves, exactly as shown above.
- Does not upgrade any correlated value to `Exact` under any condition.
- Does not persist an observation across ticks or auto-refresh a window —
  there is no window.

## Phase 10: exporter determinism, automatic lifecycle, and role audit

### Exporter dynamic-function registry: root cause and fix

`LIM-EXP-006` (Phase 9's workaround for exporter nondeterminism) is
resolved. Root cause: `register_dyn_fn`/`register_dyn_fn_dedup`/
`drain_dyn_fns` (`sand-core/src/function.rs`) backed a single
process-global `Mutex<Vec<..>>`. Rust's default test harness runs many
`#[test]` functions from one binary **concurrently on separate threads**,
so two tests that both triggered dynamic-function registration (e.g. via
`RelationQuery::if_present`/`if_player`) could race — one test's
`drain_dyn_fns()` could observe or clear entries registered by a
*different, concurrently-running* test. The registry is now **thread-local**:
each thread's export sees only its own registrations, with no cross-thread
interference possible, since a single `try_export_components_json` call
(and everything it invokes — `SandEvent::setup()`, condition lowering,
relation helpers, item-snapshot capture, participant observation) runs as
one synchronous call tree on one thread. Verified by
`sand-core/tests/exporter_dyn_fn_determinism.rs`, which deliberately
reproduces the original `if_present`-from-`SandEvent::setup()` pattern and
asserts byte-identical output across ten repeated exports, and by unit
tests in `sand-core/src/function.rs` (`dyn_fn_registry_tests`) proving
thread isolation directly (a spawned thread never observes or clears the
main thread's registrations, and vice versa).

### Automatic participant-observation lifecycle: `EventParticipantPlan`

`sand_core::participant::EventParticipantPlan` (`sand-core/src/participant/plan.rs`)
lets an event definition *declare* which participant observations it
needs, instead of an author manually sequencing
`observe_correlated_attacker`'s reset/mark/cleanup commands by hand:

```rust
impl SandEvent for HurtEvent {
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().observe_correlated_attacker()
    }

    fn setup() -> EventSetup {
        EventSetup::none()
            .with_participants::<Self>(Self::participants(), "mypack:observations", &profile)
            .expect("target version supports the declared participants")
    }
}
```

`SandEvent::participants()` is a genuinely additive default trait method
(defaults to `EventParticipantPlan::none()`; every existing `SandEvent`
implementation is unaffected). This is **not** fully macro-transparent —
`setup()` still calls `.with_participants(...)` itself; `#[event]`/the
tick coordinator do not inspect `participants()` automatically. What it
removes is the need to hand-assemble the observation's command sequence
and get the pre/post-observation split right: `.with_participants()`
appends the plan's setup commands to `pre_observation` and its cleanup
commands to `post_observation`, deterministically.

**Lifecycle ordering**: for a tick-backed event, one generated check
function runs, in order: existing `pre_observation` commands, then the
plan's reset+mark/bind commands (appended); the condition test; the
handler dispatch and its synchronous descendants (still inside the same
generated function, so the plan's temporary tag is still present); then
existing `post_observation` commands, then the plan's cleanup commands
(appended last). Cleanup runs *after* existing `post_observation`
commands — deliberately, so any legitimate user `post_observation` logic
still has access to the observed participant's tag before it's removed.
Because Sand generates straight-line command sequences (no exception
unwinding, no early return), cleanup is structurally unavoidable: it runs
whether the participant was present or absent, whether the condition
matched or not, and whether the handler ran or not — proven by
`sand-core/tests/participant_plan_export.rs`.

**Capability integration**: `EventParticipantPlan::capabilities()` returns
the `EntityParticipantCapability` entries a declared plan contributes
(role, `Correlated` reliability, `SynchronousDescendants` lifetime,
occurrence-dependent, versioned). `EventContextCapabilities::for_event`
still only derives the subject (it cannot see `participants()` — same
`LIM-CTX-001`-style boundary as chained dispatch, since nothing wires the
two together automatically yet); combining `for_event::<E>()`'s subject
capability with `E::participants().capabilities()` is the documented
pattern for a full descriptor today.

**Reentrancy**: the plan API avoids the same-schema-nesting risk
`LIM-CTX-005` originally flagged for the manual API. Every plan's schema
key is derived from `std::any::type_name::<E>()`, never a caller-supplied
string, so two distinct `SandEvent` types applying the same plan (same
role, same storage namespace) will not collide in practice — proven for
distinct types by
`distinct_event_types_never_collide_even_with_the_same_role_and_storage`
in `sand-core/src/participant/plan.rs`. This relies on the same 32-bit
FNV-1a content-addressed key scheme `tick_event_resource_key` uses
elsewhere (`sand-core/src/events/graph.rs`) — a real but astronomically
unlikely hash collision between two distinct type names is not
mathematically ruled out, and unlike the event-graph exporter (which
checks a `key_registry: BTreeMap<String, &'static str>` at export time and
fails loudly on any collision — see `sand-core/src/component.rs`), the
plan API has no equivalent export-time collision guard today. Within one
plan, declaring the
same role twice is rejected at `.validate()`/`.with_participants()` time
(`DuplicateParticipantRole`), before any commands are generated. The
manual `observe_correlated_attacker` API (Phase 9, unchanged) still
accepts a caller-chosen `event_label` and retains the original caveat —
give a manually-nested observation its own distinct label.

**Graph propagation**: not extended in this phase. A plan is applied
per-event-type via `setup()`; nothing automatically propagates a declared
plan's participant capability through `after`/`after_any`/`after_all`/
`while`/`within`/advancement-bridge composition the way Phase 8's
`propagate_*`/`merge_*` functions handle the subject capability. A child
event wanting the same observation must declare its own plan.

### Additional participant-role evidence audit

Evaluated whether any other `execute on <relation>` (`sand-core/src/entity/relation.rs`:
`Owner`, `Leasher`, `Target`, `Vehicle`, `Controller`, `Origin`,
`Passengers`, alongside `Attacker`) could back another correlated
participant role, using the same standard applied to attacker: single-valued
relation, stable identity, no ambiguity-selection policy required.

| Role | Relation | Classification | Implemented? |
|---|---|---|---|
| Attacker / Killer | `execute on attacker` | Single-valued relation, correlated (Phase 9/10) | Yes |
| Victim (attacker-is-subject direction) | — | No relation exposes "the entity I just hit" for a player; `Target` is a *mob's* current attack/follow target, not "who this player last damaged" — applying it to a player subject would be an unverified semantic assumption, not evidence | No |
| Projectile owner | `execute on origin` ("the entity that fired/summoned this entity") | Single-valued relation with real vanilla semantics for a projectile's shooter — *but* no event in Sand exposes the projectile entity itself as the event subject (`ShotCrossbowEvent` etc. fire with the *player* as subject, not the projectile), so there is nothing to call `execute on origin` from | No — blocked on projectile-as-subject, not on the relation itself |
| Interacted entity | — | No relation or advancement-trigger context was found that exposes "the entity a player just interacted with" as a bindable target from the player subject; only proximity/looked-at heuristics would work, which are `Inferred`, not `Correlated`, and are explicitly out of scope | No |
| Target (mob's own target) | `execute on target` | Single-valued relation, but it names *the mob's own AI target*, not any role a current Sand event family's subject would plausibly want | No — no corresponding event role |
| Vehicle / Controller / Owner (non-combat) | `execute on vehicle`/`controller`/`owner` | Single-valued relations with genuine semantics, but no current event family (ride/vehicle events fire with the player as subject already) has an unresolved participant need these would fill | No — no corresponding event role with a gap to fill |

**Implementation rule applied**: an additional role was implemented only
where (a) a single-valued vanilla relation exists, (b) a concrete Sand
event subject can call it, and (c) no unverified semantic assumption is
required to map the relation to the role. Only attacker/killer satisfied
all three. Projectile owner is the most promising *unimplemented*
candidate — `execute on origin` is real evidence — but is blocked on Sand
not yet having a projectile-entity-as-subject event family, which is a
larger, separate piece of work (a new observation/event architecture, not
a role-audit gap) left for a future phase.

### What Phase 10 does not do

- Does not implement victim, interacted-entity, or projectile-owner
  recovery (see the audit above).
- Does not add a proximity/heuristic (`Inferred`) observer.
- Does not make `EventParticipantPlan` macro-transparent — `setup()` must
  still call `.with_participants(...)` itself.
- Does not wire plan-declared capabilities into `EventContextCapabilities::for_event`
  automatically, or into Phase 8's graph propagation functions.
- Does not add a typed `Event<T>::context().entity(role)` handler-context
  accessor — that remains future work; today, a handler that needs the
  observed entity references it via the selector `@e[tag=__sand_observed_<key>,limit=1]`
  (available through `CorrelatedEntityObservation::participant()` if the
  author keeps a handle to it, or reconstructed manually from the same
  `ObservationSchema`).
- Does not run real Minecraft server/client semantic validation for
  attacker observation — no server is available in this authoring
  environment (see `ai/known-limitations.md` `LIM-VAL-010`, unchanged from
  Phase 9: still architecture/export-structural evidence only). The
  reliability classification remains `Correlated`, not upgraded to
  `Exact`, since no such evidence was gathered.
