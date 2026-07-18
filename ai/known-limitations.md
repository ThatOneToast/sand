# Known limitations

Consolidated, reviewed 2026-07-15. Each entry has a stable ID, so other files
and PRs can reference it (`[[LIM-XXX-NNN]]`). This file summarizes; it does
not replace the detailed source docs it cites — follow the evidence links for
depth.

Historical audit documents (`docs/*-audit.md`,
`docs/research/datapack-parity-audit.md`) are **not** treated as current
truth here. Where this file cites one, it has been reverified against
current source as of `last_reviewed`; unverified historical claims are
explicitly labeled as such below.

## Vanilla Minecraft limitations

These are constraints of vanilla Java Edition itself — no Sand typing change
can remove them.

- **LIM-VAN-001** — Command storage NBT is global, not namespaced per player
  by the game. Dynamic per-player storage keys are a Sand/authoring
  convention, not a vanilla guarantee.
  Affects: `storage`.
  Evidence: `book/src/reference/vanilla-limitations.md`.

- **LIM-VAN-002** — Advancement triggers do not cover every gameplay action,
  and most triggers do not expose a rich event payload — they fire a
  condition check, not an event-with-data.
  Affects: `advancement-triggers`, `events-typed`.
  Workaround: combine a trigger with typed state/storage reads in the
  triggered function to recover context.
  Evidence: `book/src/reference/vanilla-limitations.md`,
  `sand-components/src/advancement/trigger_coverage.rs` (`EventWrapperStatus`).

- **LIM-VAN-003** — There is no vanilla "damage taken" event with a payload;
  damage tracking is a cumulative-scoreboard-stat approximation (total
  damage taken since last reset), not a per-hit event.
  Affects: `damage-tracking`.
  Evidence: `book/src/reference/vanilla-limitations.md`,
  `sand-core/src/systems/damage.rs`.

- **LIM-VAN-004** — Vanilla has no arbitrary entity physics/velocity API
  reachable from datapacks. Push/launch/speed-boost/slow helpers are built
  from teleports and potion-effect-like mechanics, not free-form physics.
  Affects: `movement`.
  Evidence: `book/src/reference/vanilla-limitations.md`.

- **LIM-VAN-005** — Interaction entities (armor stands, interaction hitboxes)
  react to right-click/attack, not proximity. There is no vanilla
  "on approach" trigger without tick-polling distance checks.
  Affects: `entities-interactables`.
  Evidence: `book/src/reference/vanilla-limitations.md`.

- **LIM-VAN-006** — There is no dedicated vanilla event for "successful
  shield block" or "axe disabled a shield" — these must be approximated
  (e.g. via damage-amount heuristics), not detected exactly.
  Affects: `advancement-triggers`, `damage-tracking`.
  Evidence: `book/src/reference/vanilla-limitations.md`.

## Sand API limitations

Vanilla supports the behavior; Sand's typed coverage is incomplete.

- **LIM-API-001** — Worldgen registries `configured_feature`, `structure`,
  `structure_set`, `processor_list`, `template_pool`, `density_function`,
  `noise`, `configured_carver`, and `dimension_type` have no typed Sand
  module at all.
  Affects: `worldgen-registries`.
  Workaround: `sand_components::raw::RawComponent`.
  Evidence: `sand-components/src/registry_coverage.rs`.

- **LIM-API-002** — Loot table pool conditions and several entry types are
  missing from the typed loot table builder.
  Affects: `loot-tables`.
  Workaround: `RawComponent` for the specific pool/entry.
  Tracking: issue #17.
  Evidence: `sand-components/src/registry_coverage.rs` (`minecraft:loot_table`).

- **LIM-API-003** — Predicate coverage for location, weather, and time
  predicate variants is partial.
  Affects: `predicates`.
  Workaround: `Predicate::raw(RawJson)`.
  Evidence: `sand-components/src/registry_coverage.rs` (`minecraft:predicate`).

- **LIM-API-004** — Item modifier registry covers `SetCount`,
  `SetComponents`, `EnchantRandomly`; the full vanilla modifier set is
  incomplete.
  Affects: `item-modifiers`.
  Workaround: `RawComponent`.
  Evidence: `sand-components/src/registry_coverage.rs` (`minecraft:item_modifier`).

- **LIM-API-005** — Dialog registry has a builder and well-known tag helpers
  (`pause_screen_additions`, `quick_actions`), but broader schema validation
  is partial and the API is still marked experimental.
  Affects: `dialogs`.
  Evidence: `sand-components/src/registry_coverage.rs` (`minecraft:dialog`),
  `ROADMAP.md` (Experimental Areas).

- **LIM-API-006** — `minecraft:enchantment_provider` (1.21+) has no typed
  module.
  Affects: none (no capability entry yet — treat as `raw_only` via
  `RawComponent`).
  Evidence: `sand-components/src/registry_coverage.rs`.

- **LIM-API-007** — `sand join --local` is an unimplemented no-op: it parses
  `sand.toml` and returns without joining anything, despite CLI help text
  describing Prism Launcher integration.
  Affects: `cli-join`.
  Evidence: `sand/src/join_cmd.rs:17-19`.

- **LIM-API-008** — Only `BlockId`, `ItemId`, `EntityTypeId`, and
  `FunctionId` have a `TypedTag<T>`. Other taggable registries (fluid,
  damage_type, dimension_type, worldgen tags, enchantment) are reachable
  only through the raw `Tag` type or `RawComponent`.
  Affects: `tags`.
  Evidence: `sand-components/src/registry_coverage.rs` (`TAG_COVERAGE`).

- **LIM-API-009** — The profile-aware `RenderCommand` boundary currently has
  structural implementations for selectors/targets, coordinates, item slots,
  scoreboard holders/objectives/operations, and foundational `Execute`
  arguments. Other command families still migrate incrementally. The export
  boundary always checks collected function-line integrity and applies deeper
  fallback validation only to confidently recognized top-level commands and
  exact argument positions. It deliberately preserves unknown, macro, modded,
  and command-shaped literal JSON/SNBT content rather than speculatively
  parsing it.
  Affects: `command-validation`, `typed-commands`.
  Workaround: prefer typed builders and their `try_*`/`RenderCommand` paths;
  use `cmd::raw` only for advanced vanilla or modded syntax Sand does not model.
  Evidence: `sand-commands/src/render.rs`, `sand-core/src/component.rs`.

- **LIM-API-010** — Advancement-trigger coverage is partial across the
  verified 1.21.4 and 26.2 registries: 50 current registry IDs have typed
  variants, eight current IDs require `AdvancementTrigger::Custom`, and six legacy
  source-compatibility variants are rejected because their IDs exist in
  neither verified registry. Typed location, entity, damage, and item
  predicates are profile-rendered where verified; unverified legacy
  damage-source booleans and unsupported location `feature` filters fail with
  diagnostics instead of being silently ignored. Raw/custom compatibility is
  user-owned because Sand cannot semantically validate arbitrary JSON.
  Affects: `advancement-triggers`.
  Workaround: check `TRIGGER_COVERAGE`; use `Custom`/`RawJson` only with JSON
  verified for the exact target profile.
  Evidence: `sand-components/src/advancement/trigger_coverage.rs`,
  `sand-components/fixtures/trigger-coverage/`.

## Version-sensitive behavior

- **LIM-VER-001** — `VersionProfile::resolve()` never fails for an
  unknown/future Minecraft version; it silently returns a conservative
  fallback (`is_fallback: true`, all optional-feature flags `false`, pinned
  to the latest known pack formats). Code that only checks `Ok(profile)` can
  silently under-target a version. Use `resolve_strict()` when the caller
  needs a hard error instead.
  Affects: `version-profiles`.
  Evidence: `sand-core/src/version.rs` (`resolve`, `resolve_strict`,
  `VersionCaps::conservative`).

- **LIM-VER-002** — Dialogs, chat types, jukebox songs, enchantment
  data-drivenness, damage types, trim material/pattern, and wolf variants
  are each gated to a specific minimum Minecraft version (see each
  capability's `minecraft.minimum` in `capability-manifest.yaml`). Using a
  capability against an older `mc_version` in `sand.toml` than its gate will
  not automatically downgrade gracefully — check the gate before authoring.
  Affects: `dialogs`, `damage-type-registry`, and others.
  Evidence: `sand-components/src/registry_coverage.rs` (`version_gate` field).

## Experimental areas

- **LIM-EXP-002** — `mcfunction!` macro is explicitly positioned as advanced
  tooling, not the beginner authoring path; prefer `#[function]` +
  `#[component]` for new pack code.
  Affects: `functions`.
  Evidence: `ROADMAP.md`, `RELEASE.md` (Stability levels).

- **LIM-EXP-003** — Resource pack generation and HUD workflows are alpha and
  require manual asset setup; do not assume `sand add resourcepack` produces
  a complete, playable resource pack without further authoring.
  Affects: `resource-pack`, `hud-workflows`.
  Evidence: `ROADMAP.md`, `book/src/resource-packs-and-hud.md`.

- **LIM-ITEM-001** — `ItemSnapshot::capture()` is not auto-wired into the
  `#[event]` macro or the tick coordinator's generated pipeline (#229 Phase
  7). A `SandEvent` author must call it themselves and embed the returned
  commands into their own `EventSetup::pre_observation` (tick-backed) or the
  first lines of their handler body (advancement-backed) — there is no
  automatic per-event capture. Capture at a Phase 6 advancement-backed graph
  bridge parent (`sandevent-advancement-graph-parent`) is not supported at
  all: bridge parents have no `EventSetup`/handler-body seam to embed
  capture commands into. Full graph-integrated, participant-rich capture is
  deferred to #230.
  Affects: `item-snapshots`.
  Evidence: `sand-core/src/item/snapshot.rs`, `docs/items.md`,
  `sand-core/tests/item_snapshot_tick_capture_export.rs`.

- **LIM-ITEM-002** — `ItemLocation::PlayerEquipment`/`entity_equipment`
  reject `EquipmentSlot::Body` (`ItemLocationError::UnsupportedLocation`):
  no single stable NBT tag for a "body" armor item was verified across
  Sand's supported version range. `PlayerEquipment` additionally rejects
  `Mainhand`/`Offhand` (use the dedicated `ItemLocation::PlayerMainHand`/
  `PlayerOffHand` variants, which address `SelectedItem`/slot `-106`
  directly rather than through generic equipment-slot addressing).
  Affects: `item-locations`.
  Evidence: `sand-core/src/item/location.rs`
  (`player_equipment`, `entity_equipment`).

- **LIM-ITEM-003** — `SnapshotReliability::ExactPostTrigger` (used for
  advancement-backed captures) is an honest acknowledgment, not a verified
  guarantee: whether a given advancement criterion fires before or after
  vanilla has already mutated the triggering item (e.g. `consume_item`
  decrementing a stack) is criterion-specific and has not been individually
  verified per criterion. Treat `ExactPostTrigger` snapshots as "earliest
  point Sand had control," not as proof the item was unmutated at that
  point. Only tick-backed `SnapshotReliability::Exact` captures (embedded in
  `pre_observation`, which genuinely runs before Sand's own condition test)
  carry a stronger ordering guarantee, and only relative to Sand's own
  generated commands, not to vanilla's internal engine ordering before Sand
  gains control at all.
  Affects: `item-snapshots`.
  Evidence: `sand-core/src/item/snapshot.rs` (module docs, capture
  ordering), `docs/items.md`.

- **LIM-CTX-001** — `EventContextCapabilities::for_event::<E>()` (#230
  Phase 8) does not resolve capabilities for a `SandEventDispatch::Chain`
  (same-cycle chained) event type: a `ChainEventDispatch`'s parent(s) are
  identified by type-erased function-pointer factories specifically so the
  parent marker type never needs instantiating (see
  `sand-core/src/events/graph.rs` `OccurrenceParent`), so `for_event`
  cannot generically call `for_event::<Parent>()` from inside an
  already-erased dispatch value. It returns
  `EventContextCapabilities::NONE` for every chained event type rather than
  fabricating a subject capability. A caller who knows the concrete parent
  type must call `for_event::<Parent>()` themselves and combine it with
  `propagate_after`/`merge_after_any`/`merge_after_all` from
  `sand_core::participant::capabilities`. Full graph-integrated capability
  resolution (walking a `ChainEventDispatch`'s real parent chain
  automatically) is Phase 9 work.
  Affects: `participant-context-capabilities`.
  Evidence: `sand-core/src/participant/capabilities.rs`
  (`EventContextCapabilities::for_event`,
  `chained_event_capabilities_are_not_resolved_generically` test),
  `docs/event-context.md`.

- **LIM-CTX-002** — No entity/item/location participant capability is
  populated for any currently supported `SandEvent` family — every family
  audited in `sand-core/tests/participant_context_capability_audit.rs`
  (player join/state-tick, death/respawn-adjacent, kill/damage advancement
  triggers, item-used, placed-block, interaction, projectile-adjacent,
  ride/vehicle) resolves to an exact player subject with empty
  `entities`/`items`/`locations` lists. There is no attacker, victim,
  interacted-entity, or projectile-owner capability anywhere in the
  codebase yet — populating any of those for a real event type is #230
  Phase 9 work, not something #230 Phase 8's type system alone provides.
  Affects: `participant-context-capabilities`.
  Evidence: `sand-core/tests/participant_context_capability_audit.rs`,
  `docs/event-context.md`.

- **LIM-CTX-003** — `EntityParticipant`'s only exact constructor is
  `EntityParticipant::subject()`/`PlayerParticipant::subject()` (the
  event's own triggering/polled player). `EntityParticipant::correlated`/
  `::inferred` are the only constructors for a non-subject reference, and
  they hard-code `ParticipantReliability::Correlated`/`Inferred`
  respectively — there is no API path to construct a non-subject
  participant claiming `Exact`. An "exact non-subject entity" would require
  a stable generated binding mechanism (e.g. the tag-then-target pattern
  `sand_core::entity::EntityScope::bind` already uses for live traversal)
  applied at an authoritative event boundary, which is #230 Phase 9
  observation-backend work, not a type-system concern Phase 8 addresses.
  Affects: `participant-reliability-model`.
  Evidence: `sand-core/src/participant/reference.rs`.

- **LIM-CTX-004** — Graph propagation/merge functions in
  `sand_core::participant::capabilities` (`propagate_after`,
  `merge_after_any`, `merge_after_all`, `propagate_while`,
  `propagate_when_unless`, `propagate_within`) operate on `SubjectCapability`
  values only — they are pure functions the caller invokes explicitly, not
  something the event graph exporter calls automatically during
  `after`/`after_any`/`after_all`/`while`/`within`/advancement-bridge
  composition today. No export-time validation currently rejects a context
  request that would require unsupported propagation; that wiring (and
  automatic entity/item/location list merging, which the functions do not
  yet do — only subject-level merging is implemented and tested) is #230
  Phase 9 work.
  Affects: `participant-context-capabilities`.
  Evidence: `sand-core/src/participant/capabilities.rs`, `docs/event-context.md`.

- **LIM-CTX-005** — `observe_correlated_attacker` (#230 Phase 9) is not
  reentrant for the same `event_label` within one synchronous call tree:
  two nested calls with the same schema would use the same
  `__sand_observed_<key>` tag and `obs.<key>.present` storage path, so an
  inner call's reset/cleanup would interfere with an outer call's still-in-
  use observation. This mirrors Phase 7's identical `ItemSnapshot`
  same-schema-reentrancy caveat (documented in
  `sand-core/src/item/snapshot.rs`'s module doc) and is not independently
  guarded against — give a nested observation its own distinct
  `event_label` if this could occur.
  Affects: `participant-attacker-observation`.
  Evidence: `sand-core/src/participant/observation.rs` (module doc,
  "Multiplayer safety" section).

- **LIM-EXP-004** — Same-cycle single- and multi-parent `SandEvent` dispatch
  and explicit persistent `while_<E>()` conditions are the implemented
  composition phases of #240. Multi-parent groups support two through eight
  typed parents, at most one `after_any` and one `after_all` group per child,
  conjunctive clauses, deterministic per-subject occurrence staging, and
  at-most-once any-group coalescing. Persistent conditions are currently
  player-scoped, directly queryable states; they are evaluated live at the
  child boundary and do not invoke another event detector or transition
  lifecycle. Bounded `within::<E>(TickWindow)` cross-tick correlation
  (Phase 5 of #240) is implemented — see `sandevent-bounded-correlation`. An
  advancement-backed `SandEvent` parent (Phase 6 of #240) is accepted only as
  a child's sole `after::<Parent>()` occurrence dependency, bridged directly
  from its own advancement reward function — see
  `sandevent-advancement-graph-parent`. Every other advancement-backed
  occurrence shape (`after_any`/`after_all`, combined with a second
  occurrence clause, `.within(...)`, or a direct `#[event]` handler combined
  with graph composition on the same type) is explicitly rejected, since Sand
  does not control the reward function's execution order relative to the
  tick coordinator. A bridged advancement-backed parent's own
  `SandEvent::setup()` must also be empty — the synchronous bridge never
  executes the parent's own lifecycle (objectives/pre_observation/
  post_observation), so a non-empty setup is rejected at graph discovery
  (before any records are emitted) rather than silently discarded; the
  dependent child's own setup and conditions are unaffected. Participant
  recovery for victim/interacted-entity/projectile-owner and arbitrary
  non-player entity execution scopes are not implemented and are not
  exposed as partial APIs. Correlated attacker recovery (#230 Phase 9,
  `observe_correlated_attacker`) is implemented but not integrated into
  this composition surface's graph — it is manually embedded per event,
  same as Phase 7's item snapshots. The typed reliability/availability/
  lifetime/capability *vocabulary* is #230 Phase 8
  (`sand_core::participant`, see `LIM-CTX-001`..`LIM-CTX-005`).
  Affects: `sandevent-chained-dispatch`, `sandevent-persistent-conditions`,
  `sandevent-multi-parent-composition`, `sandevent-bounded-correlation`,
  `sandevent-advancement-graph-parent`.
  Evidence: `sand-core/src/events/graph.rs`, `sand-core/src/component.rs`,
  `book/src/manual/events.md`
  (Same-cycle and persistent composition).

- **LIM-EXP-005** — Bounded `.within(...)` age counters only advance for
  online players: the generated age update runs under `execute as @a`, which
  only iterates currently-online players, so age advances while a player is
  online and pauses (does not advance) while they are offline. The
  underlying scoreboard value is not reset by disconnect/reconnect or
  `/reload` (it persists like `Cooldown`/`Timer` state), so a returning
  player resumes aging from wherever it paused rather than restarting from
  0. Practical effect: a bounded window that would have expired in real time
  can still be open when a player reconnects, if the parent fired shortly
  before they disconnected and few enough *online* ticks have elapsed since.
  This is consistent with Sand's existing scoreboard-state guarantees
  elsewhere (no vanilla mechanism ticks state for offline players) and is
  not a bug, but callers relying on `.within(...)` as an approximation of
  wall-clock recency should account for it. Separately, the increment is
  guarded to stop at `TickWindow::MAX_TICKS` (24,000) rather than
  incrementing unboundedly, so a permanently-idle parent's age cannot
  overflow the signed 32-bit scoreboard value and wrap negative.
  Affects: `sandevent-bounded-correlation`.
  Evidence: `sand-core/src/component.rs` (bounded age-counter maintenance),
  `sand-core/tests/event_chain_within_export.rs`,
  `docs/events.md`, `book/src/manual/events.md`.

## Validation gaps

- **LIM-VAL-001** — Passing `cargo test --workspace` (including golden
  export tests) is evidence the generated JSON/mcfunction *shape* is
  correct, not that a vanilla server successfully loads the datapack.
  Vanilla-reload validation (`scripts/validate-vanilla-reload.sh`) is not part
  of the default `cargo test` run. Local runs require cached server jars and
  Java 21/25; the checked-in scheduled/workflow-dispatch action covers the
  verified profiles separately from pull-request CI.
  Affects: `cli-validate`.
  Evidence: `scripts/validate-vanilla-reload.sh`, `docs/vanilla-reload-validation.md`.

- **LIM-VAL-002** — `docs/research/datapack-parity-audit.md` claims a last
  local vanilla-reload run of 2026-07-12, but this claim is inside a
  document already shown (LIM-DOC-002) to contain stale per-registry
  counts from the same review cycle. Treat that specific claim as
  unverified until re-run and cross-checked against `registry_coverage.rs`.
  Affects: `cli-validate`.
  Evidence: `docs/research/datapack-parity-audit.md`.

- **LIM-VAL-003** — Real-server load/reload evidence proves that advancement
  JSON parses, not that a filtered criterion has the requested gameplay
  semantics. A connected protocol-client fixture provides positive, negative,
  final-item-in-stack, and revoke/reset re-fire evidence for `placed_block`
  and `item_used_on_block` on 1.21.4. Other triggers and the 26.2 profile have
  no automated semantic-runtime evidence and must not inherit that claim from
  snapshots or reload success.
  Affects: `advancement-triggers`, `cli-validate`.
  Evidence: `docs/vanilla-reload-validation.md`,
  `sand-components/src/advancement/trigger_coverage.rs`.

- **LIM-VAL-004** — Persistent `while_<E>()` has real single-player runtime
  evidence only for current sneaking on Minecraft 1.21.4. The other supported
  persistent providers, Minecraft 26.2 semantics, and two-player isolation
  remain structural export/unit evidence; reload success does not upgrade
  those claims.
  Affects: `sandevent-persistent-conditions`, `cli-validate`.
  Evidence: `sand-vanilla-audit/src/lib.rs`,
  `scripts/vanilla-semantic-client/client.cjs`.

- **LIM-VAL-005** — Multi-parent same-cycle composition has real single-player
  1.21.4 protocol-client evidence for either-parent matching, both-parent
  at-most-once coalescing, all-parent positive/negative matching, reverse
  atomic parent order, repeated-one-parent rejection, reset, and no stale
  next-tick occurrence. Per-subject scoreboard structure proves isolation, but
  no two-client runtime test currently verifies multiplayer behavior. 26.2 has
  real load/reload evidence only; that does not prove gameplay semantics.
  Affects: `sandevent-multi-parent-composition`, `cli-validate`.
  Evidence: `sand-core/tests/event_multi_parent_export.rs`,
  `sand-vanilla-audit/src/lib.rs`,
  `scripts/vanilla-semantic-client/client.cjs`,
  `docs/vanilla-reload-validation.md`.

- **LIM-VAL-006** — Bounded `.within(...)` cross-tick correlation (Phase 5 of
  #240) has deterministic, exact boundary evidence only from `sand-core`
  unit/export tests (the generated `matches ..N-1` condition, age-counter
  ordering, and objective dedup) — those tests assert exact generated command
  text, not live server timing. A real 1.21.4 protocol-client fixture
  (`SemanticWithin`) is prepared and exercises the "clearly within,"
  "refreshes," and "clearly expired" cases, but has not been run against a
  live server as part of landing this feature (no server available in the
  authoring environment); do not treat it as executed runtime evidence until
  someone runs `scripts/validate-vanilla-semantics.sh` against it. 26.2 has no
  semantic-runtime claim at all. Two-client multiplayer isolation remains
  structural (per-`@s` command generation), not a two-client runtime test.
  Affects: `sandevent-bounded-correlation`, `cli-validate`.
  Evidence: `sand-core/src/events/graph.rs`,
  `sand-core/tests/event_chain_within_export.rs`,
  `sand-vanilla-audit/src/lib.rs`,
  `scripts/vanilla-semantic-client/client.cjs`,
  `docs/vanilla-reload-validation.md`.

- **LIM-VAL-007** — Advancement-backed graph parents (Phase 6 of #240) have
  exact structural evidence from `sand-core` export tests (advancement/entry
  generation, revoke-first ordering, provider-only subscription, multi-child
  sharing, and every rejected combination's diagnostics) plus deterministic
  1.21.4 and 26.2 load/reload export evidence (`SemanticAdvancementParent`,
  `SemanticAdvancementBridgeChild` in `sand-vanilla-audit`) — the generated
  advancement/reward JSON and entry function parse and export identically
  across repeated runs. There is no dedicated client-driven positive/negative
  timing test for this specific bridge (unlike `sandevent-bounded-correlation`
  and `sandevent-multi-parent-composition`, which have a real 1.21.4
  protocol-client fixture) — do not treat load/reload/export success as
  evidence that the bridged child actually dispatches correctly during real
  advancement-firing gameplay. 26.2 has no semantic-runtime claim at all.
  Two-player multiplayer isolation remains structural (the advancement
  reward mechanism itself scopes `@s` to the triggering player; no
  `execute as @a`/`@e` wrapper is ever generated for the bridge), not a
  two-client runtime test.
  Affects: `sandevent-advancement-graph-parent`, `cli-validate`.
  Evidence: `sand-core/tests/event_chain_advancement_parent_rejected.rs`,
  `sand-core/tests/event_chain_advancement_parent_composition.rs`,
  `sand-vanilla-audit/src/lib.rs`,
  `docs/vanilla-reload-validation.md`.

- **LIM-VAL-008** — Item locations and snapshots (Phase 7 of #229) have
  exact structural evidence from `sand-core` unit tests (per-`ItemLocation`
  variant NBT rendering, index validation, `EquipmentSlot::Body` rejection)
  and export-level integration evidence
  (`item_snapshot_tick_capture_export.rs` proves capture commands are the
  first lines of a real generated tick-check function, strictly before the
  condition test, strictly before `post_observation` cleanup, and that
  repeated export is byte-identical). There is no real-server or
  protocol-client evidence that a captured snapshot's data actually matches
  the item as it existed at the moment of the vanilla event (e.g. that
  `SelectedItem` truly reflects the pre-`consume_item` stack at the instant
  `pre_observation` runs, or that a given advancement criterion's reward
  function runs before the criterion's own item mutation). Treat
  `SnapshotReliability::Exact`/`ExactPostTrigger` as documented intent
  backed by command-ordering evidence, not as gameplay-verified runtime
  fact — see `LIM-ITEM-003`. No `sand-vanilla-audit` semantic fixture exists
  yet for item snapshots; this phase only re-confirmed deterministic
  1.21.4/26.2 export of the *existing* regression suite is unaffected by
  the new module (additive-only change), which is compile/export evidence,
  not new gameplay-timing evidence for capture itself.
  Affects: `item-locations`, `item-snapshots`, `cli-validate`.
  Evidence: `sand-core/src/item/location.rs`, `sand-core/src/item/snapshot.rs`,
  `sand-core/tests/item_snapshot_tick_capture_export.rs`, `docs/items.md`.

- **LIM-VAL-009** — The participant reliability/availability/lifetime/
  capability model (Phase 8 of #230) is type-system and metadata
  architecture with unit-test and export-level integration-test evidence
  only (`sand-core/src/participant/*`,
  `sand-core/tests/participant_context_capability_audit.rs`). No new
  runtime commands are generated by this module beyond what Phase 6/7
  already emit — `EventContextCapabilities::for_event`'s exact-player-subject
  claim rides entirely on Phase 6's already-verified `@s` subject/
  `TickScope::has_player_subject` behavior, not on any new capture/command
  path. There is no attacker/victim/interacted-entity/projectile-owner
  runtime behavior to verify, because none is implemented. Do not cite this
  phase as runtime evidence for anything beyond "the existing exact-subject
  guarantee is now also expressed as a typed capability descriptor."
  Affects: `participant-reliability-model`, `participant-context-capabilities`,
  `cli-validate`.
  Evidence: `sand-core/src/participant/`,
  `sand-core/tests/participant_context_capability_audit.rs`,
  `docs/event-context.md`.

- **LIM-EXP-006** — Discovered while building `observe_correlated_attacker`
  (#230 Phase 9): calling a `register_dyn_fn_dedup`-backed API (e.g.
  `RelationQuery::if_present`/`if_player`, which wraps a multi-command
  relation body in a separately generated function) from inside
  `SandEvent::setup()` produced non-deterministic export output —
  a first and second `try_export_components_json` call for the identical
  input produced different JSON (the dynamically-registered function record
  was present in one export and absent in the other). This points to a
  timing dependency between when `SandEvent::setup()` runs relative to the
  exporter's `drain_dyn_fns()` call that is not consistent across repeated
  export invocations in the same process. Root cause not investigated
  further — `sand-core/src/participant/observation.rs` avoids the pattern
  entirely (using two direct single-command `execute on attacker run
  <command>` lines instead of the multi-command wrapper) rather than fixing
  the underlying exporter behavior, which was judged out of scope for this
  phase. Anyone calling a `RelationQuery::if_present`/`if_player`-style API
  (or any other `register_dyn_fn_dedup` consumer) from `SandEvent::setup()`
  in the future should re-verify determinism with a `repeated_export_is_identical`-style
  test before relying on it, and only currently-known-safe use sites
  (already-existing `#[component]`/`#[function]` bodies, not
  `SandEvent::setup()`) should be assumed safe.
  Affects: `sandevent-chained-dispatch`, `participant-attacker-observation`.
  Evidence: discovered and worked around in
  `sand-core/src/participant/observation.rs`; not independently reproduced
  outside this phase's own test suite.

- **LIM-VAL-010** — Correlated attacker observation
  (`observe_correlated_attacker`, #230 Phase 9) has exact structural
  evidence from `sand-core` unit tests (reset/mark/cleanup command
  ordering, version rejection, deterministic per-schema identity) and
  export-level integration evidence
  (`participant_attacker_observation_export.rs` proves the same ordering
  through the real export pipeline, and byte-identical repeated export).
  There is **no real-server or protocol-client evidence** that vanilla's
  `execute on attacker` relation actually resolves, at the moment this
  observation's commands run, to the specific entity that caused the
  damage event the observation is embedded in — as opposed to an earlier
  hit in the same tick, or a stale "last attacker" memory from a prior,
  unrelated encounter. This is precisely why the observation is classified
  `ParticipantReliability::Correlated`, not `Exact` (see
  `docs/event-context.md` "Reliability: always Correlated, never Exact").
  Victim correlation (for `PlayerDamageEntityEvent`-style events, where the
  player is the attacker and the interesting participant is who they hit),
  interacted-entity correlation, and projectile-owner recovery were not
  implemented in this phase because no comparably strong, single-valued
  vanilla relation evidence was identified for them within this phase's
  scope (see `docs/event-context.md` "What Phase 9 does not do") — this is
  a scope decision, not an oversight, and should not be read as "these are
  harder but still planned for certain" without further evidence-gathering
  first.
  Affects: `participant-attacker-observation`, `cli-validate`.
  Evidence: `sand-core/src/participant/observation.rs`,
  `sand-core/tests/participant_attacker_observation_export.rs`,
  `docs/event-context.md`.

## Documentation and status contradictions found during audit (2026-07-12)

- **LIM-DOC-001** — `book/src/SUMMARY.md` does not link
  `book/src/version-capabilities.md`,
  `book/src/getting-started.md`, `book/src/typed-state.md`, and roughly a
  dozen other top-level `book/src/*.md` files, even though they are actively
  edited (some as recently as the same day as this review) and are linked
  from `README.md` and other docs. Several of these orphan files have a
  same-named counterpart under `docs/` with materially different content
  (confirmed for `typed-state.md` and `version-capabilities.md`), so two
  independently maintained versions of some topics exist and only one
  (`docs/`) is reachable from the rendered mdBook navigation. The former
  orphan `book/src/events.md` now points to the navigation-linked
  `book/src/manual/events.md`; do not assume the same repair has been made for
  other top-level pages. Do not assume
  `book/src/*.md` content is current or canonical solely because the file
  exists — check whether it's reachable from `SUMMARY.md`, and prefer
  `docs/` for these overlapping topics until reconciled.
  Evidence: `book/src/SUMMARY.md` vs `ls book/src/*.md`; diffed
  `book/src/typed-state.md` against `docs/typed-state.md`.
  Status: unresolved, out of scope for this review to rewrite the book.

- **LIM-DOC-002** — `docs/research/datapack-parity-audit.md`'s own summary
  tables lag behind `sand-components/src/registry_coverage.rs` and
  `.../trigger_coverage.rs`, despite the file having a recent mtime (it gets
  re-touched by unrelated feature PRs without its counts being
  regenerated). Confirmed discrepancies as of 2026-07-12:
  - Doc claims registry status counts `FullyImplemented: 12,
    PartiallyImplemented: 9, Missing: 10, RawOnly: 19,
    IntentionallyUnsupported: 3`; live source has `16, 9, 10, 20, 4`.
  - Doc marks loot tables and damage types `❌ Not implemented` /
    `Missing`; live source marks `minecraft:loot_table`
    `PartiallyImplemented` and `minecraft:damage_type`
    `FullyImplemented`.
  - Doc's trigger status breakdown (`FullyImplemented: 51,
    PartiallyImplemented: 0, Missing: 0, RawOnly: 0`) doesn't match live
    source (`50 FullyImplemented, 1 IntentionallyUnsupported`), even though
    the total row count (51) happens to match.
  Treat `registry_coverage.rs`/`trigger_coverage.rs` as ground truth over
  this document's prose tables. `ai/capability-manifest.yaml` was built from
  the source tables, not from this audit doc.
  Evidence: direct comparison of `docs/research/datapack-parity-audit.md`
  against `sand-components/src/registry_coverage.rs` and
  `.../advancement/trigger_coverage.rs` on 2026-07-12.

- **LIM-DOC-003** — `Datapacks.md` (repo root) is generic vanilla
  Minecraft 1.21.11 datapack reference notes, not Sand documentation, and
  has not been touched since the initial commit. It does not self-label as
  historical (unlike `Milestones.md`, which does). An agent should not cite
  it as describing Sand's current version support, which spans 1.18–26.2
  per `sand-core/src/version.rs`.
  Evidence: `git log --follow Datapacks.md` (initial commit only);
  `sand-core/src/version.rs` version table.

- **LIM-DOC-005** — `examples/custom_items.rs` is stale and does not compile
  against current source: it calls `CustomItem::new(id, base_item)` with two
  arguments, but `CustomItem::new` takes exactly one (`base: impl
  fmt::Display`) — identity is set separately via `.custom_data("key")`. It
  also passes raw JSON strings to `.custom_name(...)`/`.lore_line(...)`,
  which take `TextComponent`, not strings, and wraps builder functions in
  `#[component]`, but `CustomItem` does not implement `DatapackComponent`
  (it has no resource location; it's an inline item-component string passed
  to `cmd::give`). `git log` shows this file has not been touched since the
  initial commit while `sand-components/src/item/mod.rs` has changed since.
  `examples/README.md` itself labels this file (and `advancements.rs`,
  `recipes.rs`, `loot_tables.rs`, `particle_effects.rs`, `player_join.rs`)
  "legacy reference files," and none of the top-level `examples/*.rs` files
  are wired into any `[[example]]`/`[[bin]]` target or compiled by CI — they
  are copy-paste reference text only. Verified correct usage (compiled
  against current source during this review) is the doc-comment example at
  the top of `sand-components/src/item/mod.rs` and mirrored in
  `ai/recipes/custom-item.md`. `player_join.rs`, by contrast, does compile
  correctly against current source — it was spot-checked and its API usage
  matches; only `custom_items.rs`'s constructor/text-component/`#[component]`
  usage was confirmed broken during this review. The other four "legacy"
  files were not individually re-verified — treat them as unverified pending
  a compile check, not as confirmed-correct or confirmed-broken.
  Affects: `custom-items`.
  Evidence: `sand-components/src/item/mod.rs:1004-1044` (`CustomItem::new`),
  `:419,431` (`custom_name`/`lore_line` take `TextComponent`),
  `examples/README.md`, `git log -- examples/custom_items.rs`.

- **LIM-DOC-004** — `sand-core/Cargo.toml`'s `systems-all` feature does not
  explicitly list `systems-lifecycle`; it is only pulled in transitively
  through `systems-player-data`. This is currently correct (no missing
  functionality), but is easy to misread as a gap — noted here so it isn't
  re-flagged as a bug without checking the transitive dependency.
  Evidence: `sand-core/Cargo.toml` (`[features]`).
