# Sand game-framework roadmap

Sand compiles typed Rust gameplay definitions into vanilla Minecraft Java
datapacks. The aim is to let authors describe gameplay, ownership and state while
the compiler owns command generation, version differences and optimization.
Rust does not run inside Minecraft, and Sand cannot supply server-plugin hooks
that vanilla does not expose.

This is the current implementation roadmap, reviewed on 2026-11-05. “Implemented”
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
- [x] `Progression`: ordered objectives on canonical owner State, commit-before-reward
  advancement, explicit reset, and terminal completion queries.
- [x] `Session`: Lifetime-owned progression checks, descendant cleanup, reset/restart
  and explicit resume of saved State after reload.
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
- [x] `Scene` block/item/text display parts, finite keyframes, translation, scale,
  yaw, checked transform composition and owner following under Lifetime cleanup.
- [x] `Hud`: owner-private bossbars driven by canonical numeric expressions and
  actionbars with newest-owner leases, bounded stop/reload cleanup.
- [x] `Combat` custom State health/shields, resistance, critical conditions,
  invulnerability, healing and isolated outcome callbacks.
- [x] Native damage source attribution through scoped entity context.
- [x] `Encounter` priority-based child attack selection with cancellation/reset.

Limits: sampled collision rather than exact continuous collision; no general
physics solver, rotated/cone hit volumes or pathfinding. Scenes are flat display
parts, not skeletal animation or an automatic hierarchy. Combat does not intercept all native damage.
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
- [x] Typed conversation views and guarded forward/back/confirmation flows in
  Menu, preserving exact object binding and callback navigation precedence.
- [x] Whole-stack and exact-quantity numbered player-slot transfers, native-capacity stack merging,
  exact-quantity item payments and listener-interference compensation.
- [x] Typed written books using canonical rich text and item components.

Limits: native click records may coalesce; dialog close has no reliable callback,
so timeout/explicit cleanup remains necessary. Books are snapshots. Transfers merge only when effective native stack capacity and exact components
agree; they and purchases are not general transactions. Chest-slot interception,
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
- [x] Stateful conversations and confirmation flows composed from Menu and State,
  with context-preserving transitions and cancellation. See the
  [conversation guide](docs/gameplay/conversations.md).
- [x] Dynamic Journal snapshots from canonical State with explicit player-slot
  delivery/owned replacement, safe rich text and checked native delivery.
  See [journals and checkpoints](book/src/23m-journals-and-checkpoints.md).
- [x] Partial quantities and effective native default stack-capacity queries for
  numbered player slots, preserving remainders and exact components.
- [ ] Extend inventory workflows to loaded block/container destinations. Preserve identity and listener-interference
  checks; see [current bounded workflows](docs/gameplay/inventory-workflows.md).
- [ ] Reusable block/chest-shaped interactions through managed hitboxes and world
  capabilities, clearly separated from unavailable native container GUI hooks.

Acceptance: multiplayer isolation, stale/replayed submission rejection, full-slot
and unloaded-container failures, reentrant callbacks, context invalidation and
cleanup proven in native tests. Include a maintained end-to-end authoring example.

### 2. Game state and encounter authoring

- [x] Ordered quest/objective progression and once-per-reset rewards over canonical
  State and events; [quest_rounds](examples/quest_rounds/) demonstrates both.
- [x] Forward-only conditional progression branches with declaration-order priority,
  canonical objective prerequisites and once-per-reset rewards.
- [ ] Broaden reusable quest-journal policies.
- [x] Owner/host-scoped session lifecycle and explicit reset/restart policies.
- [x] Explicit bounded online arena rosters over canonical player State and typed
  roles, exclusive transfers, disconnect/reload invalidation, and a two-arena
  cooperative reference game. See [arena rosters](docs/gameplay/arena-rosters.md).
- [ ] Dynamic matchmaking, offline reservations and broader reusable round policies.
- [ ] Target selection, threat and encounter-phase policies composed with existing
  queries, Combat and child tasks. Define movement/navigation limits separately.
- [x] Checkpoint projections between canonical integer State fields, schema
  migration, all-field validation and explicit cancel/restore/fresh-task policy.
- [ ] Broaden checkpoint projections beyond integer fields where a canonical
  type and proven restore policy exist; arbitrary world/item/task snapshots remain
  unsupported.

Acceptance: two simultaneous independent sessions, deterministic transitions,
reconnect/reload behavior, reset without orphan entities/scores and reusable examples.

### 3. Broaden spatial and presentation systems

- [ ] Additional collision shapes and measured higher-fidelity projectile sweeps.
- [ ] Explicit movement/knockback policies with native capability boundaries.
- [x] Item/text scene parts and checked local transform composition.
- [ ] Hierarchical scene parenting, loops and animation blending.
- [x] Reusable lifetime-owned HUD/bossbar/actionbar presentation.
- [ ] Broader audiovisual cue composition and presentation presets.
- [x] Optional companion resource-pack asset packaging with independent version
  metadata, deterministic output, release archives and grouped directory rollback.
  Client asset codecs/rendering and automatic installation remain separate work.
  See the [companion resource-pack guide](book/src/17a-companion-resource-packs.md).

Acceptance: bounded candidate work, documented geometry/rounding semantics,
callback context preservation, unload/dimension tests and cleanup proofs. Prefer
native interpolation and existing models before adding per-tick command work.

### 4. Compiler optimization and authoring experience

- [ ] Consolidate shared admission/resource rules between synchronous activation
  policies and owned abilities while preserving their different execution and
  cleanup contracts; avoid exposing two competing state models.

- [ ] Expand typed `compute` providers and backend selection only where numeric
  failure, overflow and rounding semantics match the canonical model.
- [x] Bounded actionable tick-cost advice through RuntimeReport and CLI JSON,
  preserving unknown costs and explicit review thresholds.
- [ ] Add population-aware before/after native workloads; static command counts
  and advice thresholds are not server performance guarantees.
- [x] Fresh-agent CLI-only authoring audit, word-boundary search, type/member
  navigation, State/Scene workflow examples and missing gameplay-registration
  diagnostics. [Evidence and limits](benchmarks/evidence/api-authoring-2026-11-05/README.md).
- [ ] Improve diagnostics for ownership, version limits, execution context and
  unsupported vanilla signals; offer canonical API alternatives.
- [ ] Explicit pack reset/uninstall tooling with owned-resource cleanup; document
  that removing a datapack cannot trigger a fabricated vanilla unload callback.
- [x] One [alpha scenario command](docs/testing/alpha-workflow.md) with retained
  per-scenario logs, aggregate results and failure propagation; passive clients
  remain explicitly separate from actual player-input/rendering proof.
- [ ] Improve general gameplay test authoring and failure evidence, including negative
  cases and performance budgets. Keep export, native and compile-fail tests distinct.
- [x] Measure isolated fresh-target, warm facade-edit and no-op checks on 26.3;
  retain source fingerprints and raw logs in the [build baseline](docs/testing/build-benchmarks.md).
- [ ] Profile sand-core edits, codegen/API-contract invalidation and isolated test
  compilation before changing crate boundaries. Preserve the facade and dependency
  architecture; directory count alone is not evidence for merging crates.

Acceptance: deterministic output and semantic parity across targets; optimized
paths have versioned native evidence. Static command counts are not MSPT claims.
See [benchmarks and limitations](BENCHMARKS.md).

## Sand language expansion

The language uses the portable compiler shared with Rust. The current surface
and Rust-only areas are documented in [the language guide](book/src/language.md).
This checklist tracks the requested expansion; unchecked entries are not shipped.

- [x] Recursive functions, scoped locals, scalar casts, counting loops and custom
  events with payloads.
- [x] Exhaustive statement `match`, constant alternatives, mutable parameters,
  reversed/stepped ranges, left-to-right bound evaluation and safe range endpoints.
- [x] Package formatting through `sand fmt`, `sand lang fmt`, `--check` and LSP;
  naming and mutability quick fixes that edit declarations and references.
- [x] Automated vanilla 26.2 control-flow probe with 20 assertions and reload;
  run `scripts/validate_language.py` with a built CLI and bundled server JAR.
- [x] Constant/runtime signed arithmetic parity, target-specific vanilla catalogs
  shared by validation/completion, and runtime item counts, damage, teleport,
  experience and sound options.
- [x] General boolean arguments and stored predicates; two-player native proof
  for inventory isolation, runtime numeric commands and player properties.
- [x] Named, nested struct values in locals, arguments, returns, events and State;
  value receivers and State receivers in impl blocks, with privacy and IDE support.
- [x] Final-write assignment clamping and RHS snapshots; vanilla regression probes
  for aliasing, casts, bounded float arithmetic, record calls and event copies.
- [x] Runtime timer durations with checked seconds conversion and player-State
  method isolation in the two-player runtime probe.
- [x] Fixed-size arrays with inference, repeat literals, nested State/records,
  calls/returns, snapshot iteration, checked indexing, indexed mutable receivers,
  and len/is_empty/contains/get_or/fill methods; forty native assertions.
- [x] Owned runtime strings, scalar formatting, Unicode indexing/slicing, substring
  methods, plain runtime text and styled interpolation; native recursive, owner
  lifecycle and per-player reload proofs.
- [x] Growable `Vec<T>` values, nested/recursive records, indexed borrowed
  methods, deep copies/equality, bounded mutation and snapshot iteration; native
  vector, array and text regressions plus independent player reload proofs.
- [x] Value-producing matches for owned values, flat dispatch for large matches,
  nested record/fixed-array destructuring and shorthand-safe editor renames.
- [x] Match guards for statement and value forms, with ordered fallthrough.
- [x] Arm-local match bindings and nested record and array destructuring.
- [x] Scoped block expressions with tail values, including value match arms.
- [x] Bundled generic maps and sets with owned values, whole-record and array
  equality, an offline installer, and vanilla capacity and persistence checks.
- [x] Read-only and mutable struct receivers, with write-back on early return and
  recursive calls; retained EntityRef values, reload-safe identities, explicit
  resolution, and damage attribution with native two-player kill-credit proof.
- [x] Generic functions, nominal records and impl methods, inference and explicit
  applications, per-specialization slots/contexts, canonical source identities,
  bounded expansion and native recursive/owned-value proof.
- [x] Static traits, required signatures, and generic function and method bounds.
- [x] Trait default method bodies with concrete `Self`, overrides, and generic impls.
- [x] Typed function values, owned move closures, generic call signatures,
  recursive indirect calls and player-State persistence.
- [x] Player- and entity-scoped callable signatures with context checks at assignment and invocation.
- [x] Named groups for cancelling a subset of one executor's pending waits.
- [x] Payload enums with owned values, exhaustive matching and State defaults.
- [x] Generic enums and bundled `Option<T>` and `Result<T, E>`.
- [ ] General borrowed slices.
- [x] Owned vector subranges with clamped half-open bounds.
- [x] Borrowed range iteration over immutable local arrays, vectors, and strings.
- [x] Typed enchantments generated from one verified catalog for Rust and Sand.
- [x] Bundled typed durations and storable positions through ordinary Sand
  records and impl methods. Use existing entity State for checked membership.
- [x] Retain individual vanilla entity relationships in `EntityRef` values,
  and mount and dismount retained vehicles.
- [x] Equip, clear, and test hand and armor slots with typed equipment slots and
  custom item identity.
- [x] Filter entity queries with a State score range and a typed native NBT
  compound, composed with attachment, distance, sorting, and limits.
- [x] Velocity reads and bounded non-player velocity writes.
- [x] Read-only inventory counts for vanilla and custom items, numeric queries
  for checked entity NBT paths, and bounded item removal with a computed count.
- [x] Numbered player-slot inventory transfers through the shared Rust system
  with whole stacks, exact amounts, component checks, and callbacks.
- [ ] Broader equipment and entity data operations, motion controls, and
  archetypes.
- [ ] Broader event triggers, filters, tracked changes and event chains.
- [x] Player sneaking, sprinting, swimming, and flying transitions through the
  shared observation planner.
- [x] Named native predicate guards on player and custom event handlers.
- [x] Top-level `wait` statements in entity and player functions, with
  per-invocation locals, original dimension, and owner-scoped cancellation.
- [x] Runtime wait durations in ticks and seconds, captured for each invocation.
- [x] Waits in nested conditional branches with one shared continuation and
  per-invocation locals.
- [x] Waits inside statement match arms with guards, alternatives, and captures.
- [x] Waits inside local loops retain iteration, collection and jump state.
- [x] Server-owned continuations and independent waits inside executor-selection loops.
- [x] Ordered State migrations for global, player, and entity State.
- [x] Calls that suspend and resume their callers with return values or local mutable receiver updates.
- [x] Dimension checks and scoped calls, and block cloning with mask, filter,
  overlap, and move modes.
- [x] Biome predicates validated against the target version’s shared catalog.
- [x] Checked block entity paths, numeric reads and writes, and field removal.
- [x] Typed NBT literal writes and native data copies between block entities.
- [x] Flat dimension declarations, typed references and loaded waits in the
  declared dimension.
- [x] Custom biome climate and colors, with typed references from dimensions and
  predicates.
- [x] Structure template imports from saved native NBT files and typed placement
  at absolute or relative positions.
- [x] Biome spawn tables with typed weighted entities, group sizes, and density costs.
- [ ] Natural structure generation, biome terrain features, other world
  generators and dimension types, and first-class native block data.
- [x] Fixed and checkerboard biome placement in dimensions driven by native noise settings references.
- [x] Independent loot pools with weighted entries, random chance, and
  weather conditions.
- [x] Native damage type declarations and typed vanilla damage types, both
  available with retained attacker attribution.
- [x] Named native world predicates shared by function guards and loot pools.
- [x] Smithing armor trim recipes with a named native pattern.
- [x] Client-localized component text with positional arguments in messages and
  static resources.
- [x] Interactive chat with hover text, links, command suggestions and clipboard actions.
- [x] Fixed written books using canonical item components, rich pages and page navigation.
- [x] Runtime particle count, spread and speed with atomic range checks.
- [x] Native advancement resources with named inventory, placement and kill criteria,
  all/any requirements, visible presentation and experience rewards.
- [x] Native advancement criteria for consuming a specific item and arriving
  in a specific dimension.
- [x] Player-scoped advancement completion and named criterion queries.
- [x] Ordered completion handlers using the authored advancement's native reward
  function.
- [x] Native notice dialogs with translated text and player-scoped open and clear.
- [x] Paginated native menus with named views, player-scoped one-shot callbacks,
  and State guards for choices and sessions.
- [ ] Broader advancement triggers, worldgen and VFX.
- [x] Owner-scoped HUDs with bounded lifetime, rich actionbars, private
  State-driven boss bars, and cancellation.
- [x] Player-owned sampled projectiles with impact callbacks, target filters,
  native damage attribution and owner-scoped cancellation.
- [x] Player progression and sessions, with guarded objectives, forward
  branches, committed rewards, and per-player lifecycle controls.
- [x] Player abilities with checked costs and cooldowns, diagnostics, and
  optional projectile flight under the same admission.
- [x] Custom player health using shared Combat mitigation, shields, critical
  damage, immunity, healing and committed callbacks.
- [x] Bounded indexed player jobs with independent loaded-tick batches,
  completion callbacks and owner cancellation.
- [x] Owner-scoped native display scenes with block, item and text parts,
  local poses, interpolation keyframes, follow behavior and cleanup.
- [ ] Expose the Rust encounters, rosters, and checkpoints through canonical
  shared models.
- [x] Sand-authored tests with assertions, disposable vanilla runs, and source
  maps for portable operations and emitted function identities.
- [x] Emitted command-line mappings, including generated helper functions.
- [x] CLI lookup of authored source for a generated function and line.
- [x] Source annotations on identifiable commands in native debug traces.
- [x] Read-only State inspection against a local running server.
- [x] Pending wait inspection with source locations and group filtering.
- [x] Live Zed diagnostics and inferred-context hints using a worktree build.
- [ ] Breakpoints.

Acceptance: compiler/export tests prove observable output, runtime features have
native evidence, and each new public model preserves deterministic export,
collision safety and per-player/per-invocation ownership. Minecraft's finite
integer range, command limit and player-data restrictions remain runtime
constraints; a new surface syntax alone does not remove them.

## Validation and coverage work

- [x] Full local gate: `scripts/check.sh` plus `git diff --check`.
- [x] Shared Python test discovery for local checks and Rust CI.
- [x] Example export/runtime baselines and compile/API-contract architecture guards.
- [x] Vanilla CI wiring: both 26.2 and 26.3 for player ownership, interactions,
  inventory and HUD; 26.3 for entity gameplay/compute;
  retain failure artifacts.
- [x] Verified real-player protocol lifecycle on 26.2 and 26.3: two players,
  keepalive, dimension changes, controlled damage and reconnect persistence. See
  [jar-derived protocol evidence](docs/testing/player-protocols.md).
- [x] Run player ownership, inventory and HUD proofs on both 26.2 and 26.3.
- [ ] Broaden every remaining player scenario to 26.3; passive client support
  does not establish scripted controls or all gameplay policies on that target.
- [x] Native 26.2 proofs for exact-object isolation with nearby simultaneous
  instances, bound-menu dimension transitions and navigation token exhaustion.
  See [interaction boundary evidence](docs/testing/interaction-boundaries.md).
- [ ] Audit each version-restricted capability before broadening it; furnace
  serialization currently remains verified on 26.2 only.
- [x] Establish a [versioned capability matrix](docs/architecture/capability-matrix.md)
  separating typed authoring, runtime abstractions, unsupported edges and evidence.
- [ ] Expand the matrix into exhaustive per-command/resource/signal inventories.

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
