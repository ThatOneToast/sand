# Sand game-framework roadmap

Sand compiles typed Rust gameplay definitions into vanilla Minecraft Java
datapacks. The aim is to let authors describe gameplay, ownership and state while
the compiler owns command generation, version differences and optimization.
Rust does not run inside Minecraft, and Sand cannot supply server-plugin hooks
that vanilla does not expose.

This is the current implementation roadmap, reviewed on 2026-11-03. “Implemented”
means a bounded API exists with tests, not that the entire feature category is
finished. Sand is pre-1.0; public APIs remain changeable. Default target: **26.3**.
Keep **26.1 and 26.2** export compatibility, with explicit capability errors for
features whose native contract has only been verified on a particular version.

## Implemented foundations

- [x] Attribute-first functions, load/tick hooks, event subscriptions and typed
  systems. Authors opt into work; there is no mandatory application class.
- [x] Canonical State for scores, fixed-point values, flags, cooldowns, timers,
  enums, markers and typed storage; bundles, queries, migrations and archetypes.
- [x] Direct UUID-keyed owner storage, bounded journaled migration from legacy
  lists and explicit safe reclamation after component cleanup.
- [x] Typed entity/player/world contexts, scoped participants, inventory slots,
  item matching, native values, conditions and numeric expressions.
- [x] Typed commands/resources, generated registries and explicit escape hatches.
  Compiler diagnostics, deterministic export and API-contract enforcement.
- [x] Build-time world/server settings, CLI scaffolding, local-server workflow,
  machine-readable API discovery and portable-program frontends.
- [x] Exact 26.3 export/codegen profile and generated `compute` command support.
  Checked numeric additions use the new backend where semantics permit it;
  older targets retain compatible lowering. This is not a claim that all math
  uses `compute` or that every provider has a high-level gameplay abstraction.

Evidence: [State guide](book/src/23-unified-state.md),
[version support](book/src/reference/version-support.md),
[API enforcement](docs/architecture/public-api-surface.md),
[crate boundaries](docs/architecture/adr-001-crate-boundaries.md).

## Implemented gameplay increments

### Ownership and execution

- [x] `Lifetime` and `TaskContext`: owner/instance identity, capacity, admission,
  required and owned State, callbacks, expiry, cancellation and cleanup.
- [x] `Ability`: checked costs and cooldowns before starting owned work.
- [x] Synchronous `systems::ability` policies, shared `ResourcePool` regeneration
  and explicit `ResourceMaintenance`; accepted targets pay before effects. These
  policies do not create an owned task or promise rollback.
- [x] `StatusEffect`: owner-scoped effects with refresh and cleanup behavior.
- [x] `Job`: bounded per-instance batches and completion callbacks.
- [x] `Sequence`: ordered actions, tick waits, bounded condition waits and joins.
- [x] Parent-child composition with descendant cancellation and owner-safe
  continuation after dimension transfers.
- [x] `GameplayTest` assertions and native lifecycle/multiplayer proof harnesses.

Limits: loaded-tick execution, explicit concurrency bounds, no automatic forced
chunks. A callback budget is not a bound on arbitrary callback cost or MSPT.
Unload/reload behavior follows the documented ownership contracts; tasks are not
an automatically persistent coroutine/save system.

Evidence: [owned gameplay](book/src/23d-owned-gameplay.md),
[jobs](book/src/23e-scenes-and-jobs.md),
[composition](book/src/23g-task-composition.md),
[owned combat example](examples/owned_combat/).

### Spatial, presentation and combat

- [x] Owned projectiles with bounded substeps and block/entity hit policies.
- [x] `BlockRaycast` and `HitVolume` sphere/axis-aligned cuboid queries.
- [x] Canonical vector expressions: dot, cross, squared distance and interpolation.
- [x] `Scene` block-display parts, finite keyframes, translation, scale, yaw and
  optional owner following, all under Lifetime cleanup.
- [x] `Combat` custom State health/shields, resistance, critical conditions,
  invulnerability, healing and isolated outcome callbacks.
- [x] Native damage source attribution through scoped entity context.
- [x] `Encounter` priority-based child attack selection with cancellation/reset.

Limits: sampled collision rather than exact continuous collision; no general
physics solver, rotated/cone hit volumes or pathfinding. Scenes are flat block
parts, not skeletal animation. Combat does not intercept all native damage.
Encounters are priority controllers, not a complete AI behavior tree.

Evidence: [hit volumes/sequences](book/src/23f-hit-volumes-and-sequences.md),
[combat/encounters](book/src/23h-combat-and-encounters.md),
[animated boss](examples/animated_boss/).

### Interactions

- [x] `Menu`: expiring player sessions, server-side guards, one-shot tokens,
  disconnect/reload invalidation and automatic pagination.
- [x] `Interactable`: owned native hitboxes and actor-context use/attack callbacks.
- [x] `Interactable::opens_menu`: exact instance binding across page navigation;
  range, dimension and object availability invalidate the session.
- [x] `Purchase`: one item stack into an empty player slot, checked State currency
  and personal/shared claims, placement before payment, reentrancy protection.
- [x] Typed written books using canonical rich text and item components.

Limits: native click records may coalesce; dialog close has no reliable callback,
so timeout/explicit cleanup remains necessary. Books are snapshots. Purchases do
not merge stacks or provide general transactions. Chest-slot interception,
forced book opening and unrestricted form submission are not provided.

Evidence: [interaction guide](book/src/23i-interactions.md),
[interaction hub](examples/interaction_hub/),
[export regressions](sand-core/tests/gameplay_menu.rs),
[native proof](scripts/validate_interactions.py).

## Next increments, in priority order

### 1. Finish interaction workflows

- [ ] Typed form fields and submission validation. First prove a survival-player
  response channel on each supported target; arbitrary text must not be exposed
  as an apparent callback without a working vanilla transport. Bound choice and
  numeric flows can precede free text. Reject unsupported capabilities explicitly.
- [ ] Stateful conversations and confirmation flows composed from Menu and State,
  with context-preserving transitions and cancellation.
- [ ] Dynamic journals: generate/refresh a player's book from canonical quest
  State, with explicit delivery/slot policy and no invented persistent book state.
- [ ] General inventory transfer/payment operations: item identity/components,
  counts, destination capacity, stack merging, source removal and failure semantics.
  Reuse existing slot-copy/removal primitives; do not add a second inventory model.
- [ ] Reusable block/chest-shaped interactions through managed hitboxes and world
  capabilities, clearly separated from unavailable native container GUI hooks.

Acceptance: multiplayer isolation, stale/replayed submission rejection, full-slot
and unloaded-container failures, reentrant callbacks, context invalidation and
cleanup proven in native tests. Include a maintained end-to-end authoring example.

### 2. Game state and encounter authoring

- [ ] Quest/objective progression and rewards over canonical State and events.
- [ ] Reusable match/round/session lifecycle, teams, spectators and reset policies.
- [ ] Target selection, threat and encounter-phase policies composed with existing
  queries, Combat and child tasks. Define movement/navigation limits separately.
- [ ] Checkpoints and save/load policies with explicit schema migration and task
  restoration semantics; distinguish saved gameplay state from ephemeral runtime.

Acceptance: two simultaneous independent sessions, deterministic transitions,
reconnect/reload behavior, reset without orphan entities/scores and reusable examples.

### 3. Broaden spatial and presentation systems

- [ ] Additional collision shapes and measured higher-fidelity projectile sweeps.
- [ ] Explicit movement/knockback policies with native capability boundaries.
- [ ] Item/text scene parts, hierarchical transforms, loops and animation blending.
- [ ] Reusable HUD/bossbar/actionbar and audiovisual presentation tied to owners.
- [ ] Optional companion resource-pack asset pipeline for models, textures, fonts
  and sounds. The current CLI reserves this capability but does not implement it.

Acceptance: bounded candidate work, documented geometry/rounding semantics,
callback context preservation, unload/dimension tests and cleanup proofs. Prefer
native interpolation and existing models before adding per-tick command work.

### 4. Compiler optimization and authoring experience

- [ ] Consolidate shared admission/resource rules between synchronous activation
  policies and owned abilities while preserving their different execution and
  cleanup contracts; avoid exposing two competing state models.

- [ ] Expand typed `compute` providers and backend selection only where numeric
  failure, overflow and rounding semantics match the canonical model.
- [ ] Report actionable expensive-tick patterns, unknown costs and population
  bounds through the existing RuntimeReport; add before/after native workloads.
- [ ] Improve diagnostics for ownership, version limits, execution context and
  unsupported vanilla signals; offer canonical API alternatives.
- [ ] Explicit pack reset/uninstall tooling with owned-resource cleanup; document
  that removing a datapack cannot trigger a fabricated vanilla unload callback.
- [ ] Improve gameplay test authoring and failure evidence, including negative
  cases and performance budgets. Keep export, native and compile-fail tests distinct.
- [ ] Measure cold, warm and no-op builds before changing crate boundaries.
  Preserve the facade and dependency architecture; avoid merging crates merely
  to reduce the directory count. Revisit codegen/API-contract invalidation and
  isolated test compilation using current timings, not historical estimates.

Acceptance: deterministic output and semantic parity across targets; optimized
paths have versioned native evidence. Static command counts are not MSPT claims.
See [benchmarks and limitations](BENCHMARKS.md).

## Validation and coverage work

- [x] Full local gate: `scripts/check.sh` plus `git diff --check`.
- [x] Shared Python test discovery for local checks and Rust CI.
- [x] Example export/runtime baselines and compile/API-contract architecture guards.
- [x] Vanilla CI wiring: 26.2 for player ownership/interactions/resource policies and 26.3 for
  entity gameplay/compute;
  retain failure artifacts.
- [ ] Extend real-player protocol support and scenario proofs to 26.3. Startup
  and reload success alone do not establish multiplayer gameplay correctness.
- [x] Native 26.2 proofs for exact-object isolation with nearby simultaneous
  instances, bound-menu dimension transitions and navigation token exhaustion.
  See [interaction boundary evidence](docs/testing/interaction-boundaries.md).
- [ ] Audit each version-restricted capability before broadening it; furnace
  serialization currently remains verified on 26.2 only.
- [ ] Establish a versioned capability matrix covering commands, resources,
  events, runtime abstractions, unsupported edges and evidence quality.

Do not publish a single “percent of vanilla supported” yet. The denominator is
undefined: generated commands, typed resource schemas, native signals and
high-level systems are different dimensions. Raw escape hatches broaden what can
be emitted but do not count as typed or ergonomic coverage. Measure those axes
separately once the matrix and acceptance cases exist.

## Maintenance rules

Update this file when an increment lands: record the public abstraction, evidence
and remaining limits. Keep README concise, guides behavior-specific and historical
benchmark evidence dated. Do not relabel old 26.2 measurements as 26.3 results.
New systems should compose State, Lifetime, Target, text, items and registration;
replace obsolete APIs rather than accumulating compatibility wrappers.
