# Architecture

Sand is split into focused crates:

- `sand`: CLI
- `sand-core`: framework APIs, state, conditions, version model, component export
- `sand-commands`: typed Minecraft command builders
- `sand-components`: typed datapack JSON builders
- `sand-macros`: proc macros
- `sand-build`: Minecraft data generation and codegen
- `sand-resourcepack`: optional resource-pack and HUD helpers
- `sand-example`: integration coverage

Build flow:

1. `sand-build` resolves Minecraft data and generates Rust types.
2. `sand-core` and `sand-commands` expose typed APIs over those generated types.
3. `sand-macros` registers functions and components.
4. `sand build` writes datapack/resource-pack output.

## Event dependency graph

Custom `SandEvent` definitions are normalized into an export-time graph in
`sand-core/src/events/graph.rs`. The graph keeps single-parent `after`,
multi-parent `after_any`/`after_all`, persistent `while`, and bounded `within`
dependencies as distinct IR rather than anonymous command conditions.
Canonical concrete Rust type names supply deterministic graph and
generated-resource identity; `TypeId` is used only for in-process grouping and
collision checks.

Single-parent edges retain their immediate inherited-subject fan-out.
Multi-parent and bounded graphs add a generated cycle coordinator: it clears
only the required per-player occurrence marks, invokes root checks in
canonical order, updates each bounded parent's shared per-subject age counter
(refresh-to-`0` on occurrence, else increment — refresh always wins), then
evaluates composed nodes in deterministic occurrence-topological order. Any
node with a bounded dependency is always staged through the coordinator, even
when its occurrence shape is otherwise a single `after`, since the age counter
it reads is only current there. An event detector/setup is emitted once even
when several children, groups, or distinct `.within` windows reuse it —
distinct windows on the same bounded parent share one exact age objective
rather than one lossy objective per window. Occurrence marks are set on
inherited `@s` before dependent checks; persistent-only providers are queried
live and remain unsubscribed.
Post-observation lifecycle is deferred until dependent composed nodes finish;
mixed immediate/staged intermediates use a per-subject attempted-observation
mark so post-observation still runs after a failed child condition attempt.

Graph discovery rejects duplicate parents/groups, incompatible scopes,
canonical/generated identity collisions, conflicting `.within` windows for the
same parent, and direct or mixed cycles with edge labels (including through
`within` edges).

An advancement-backed `SandEvent` may also participate as a graph parent
(#240 Phase 6), but only as a child's sole `after::<Parent>()` occurrence
dependency — never inside `after_any`/`after_all`, never combined with a
second occurrence clause, and never referenced by `.within(...)`. Unlike
tick-backed parents, an advancement-backed parent is never inserted as a
graph node (`EventGraph::advancement_bridges` tracks it separately, keyed by
canonical type name); its detection stays owned by a synthesized advancement
+ reward-entry function pair rather than the `minecraft:tick` coordinator.
Each dependent child's condition-gated dispatch call is generated directly
inside that reward entry — synchronously, under the triggering player's `@s`,
after the existing revoke-first ordering — so no per-tick polling, pending
flag, or coordinator involvement is introduced for this relationship. This
constraint exists because Sand does not control (and will not pretend to
guarantee) the reward function's execution order relative to the tick
coordinator's own tick-tagged pass, so anything requiring the coordinator to
observe this parent's occurrence alongside another parent's mark in one
deterministic pass is rejected with a diagnostic rather than silently
approximated. The bridged parent type must have zero direct `#[event]`
handlers — combining one with graph composition on the same type is rejected,
since it would otherwise require either duplicating the live advancement
grant or splicing into the separate, pre-existing per-handler advancement
lowering path. `TickScope::AdvancementPlayer` (alongside the existing
`TickScope::Players`) is the graph's deterministic capability seam for this:
both guarantee an exact player subject, but only `Players` supports
coordinator-mediated multi-parent/staged composition.

Because the bridge dispatches the dependent directly from the parent's
reward entry rather than through any generated coordinator step, it never
runs the parent's own `SandEvent::setup()` (`EventSetup::objectives`/
`pre_observation`/`post_observation`). `resolve_occurrence_dependencies`
validates this during graph discovery — before any datapack records are
emitted — via `EventSetup::is_empty()` (the single canonical, full-field
check), rejecting the relationship with a diagnostic naming the concrete
child, the concrete parent, and (via `EventSetup::first_non_empty_category`)
which lifecycle category is non-empty, rather than silently discarding
setup the parent's author declared. Executing an advancement parent's own
lifecycle synchronously is future work, not attempted here — it would need
new ordering semantics this phase does not design. The dependent child's own
`EventSetup` is unaffected and continues to be honored normally.

Participant (entity/item) context propagation across same-cycle graph edges
is implemented for the plain single-parent case —
`EventParticipantPlan::inherit_entity`/`inherit_item` plus
`sand-core/src/compiler/export/participant_transport.rs`'s export-time
validation (#264) — but not for the advancement-bridge relationship this
section describes, nor for `after_any`/`after_all`/`.within(...)` edges; see
`docs/testing/participant-role-evidence.md`'s edge/role support matrix for
exactly which shapes are supported today.

## Export-scoped typed command registries

Eight typed command families (`blocks`, `nbt`, `particles`, `sound`,
`display`, `text`, `effect`, `inventory`) render to a `String` but retain
their typed node in a `rendered line -> node` side table, plus `execute_ir`,
which retains per-line capability requirements. The export pipeline's
pre-write boundary (`sand_commands::render::validate_collected_line`) looks
each emitted line back up so the *typed* node can be re-validated against
the export's resolved `CommandProfile` after the type has been erased into a
function body.

All nine share one store, `sand-commands/src/export_registry.rs`. The
contract:

- **Ownership.** State lives in a thread-local stack of layers, one `State`
  per family keyed by the family marker's `TypeId`. Nothing is
  process-global. Registration and lookup target the **top** layer only.
- **Lifecycle.** `ExportRegistryGuard::enter()` pushes a fresh layer;
  `Drop` pops it. `try_export_components_impl` takes the guard as its first
  act, so every one of its `?` early returns, and any unwind out of a user
  factory or `#[event]` handler, discards the layer. Cleanup is never a call
  at the end of the happy path.
- **Isolation.** Layers are thread-local and the guard is `!Send`, so
  concurrent exports on different threads cannot observe each other. An
  export cannot observe the ambient (layer 0) entries left by typed commands
  rendered outside any export.
- **Nesting.** Not supported. A second `enter()` on a thread with an open
  scope returns `NestedExportError` (`SAND-EXPORT-REGISTRY-NESTED`), which
  the pipeline surfaces as a component-validation error. The guard is taken
  *before* the process-wide dialog callback lock precisely so a reentrant
  export reports this instead of deadlocking on that non-reentrant lock.
- **Raw lines stay opaque.** A lookup miss always means "pass through
  unvalidated", never "invalid". Hand-authored raw commands, lines from an
  earlier export, and lines rendered on another thread are all misses.

A tenth family gets this lifecycle by construction: implement
`RegistryFamily` and keep the state in the layer. There is no per-family
reset to write and none to wire into the pipeline. Historically each family
owned an independent `OnceLock<Mutex<BTreeMap<..>>>`; a `Mutex` gives
data-race safety but not export isolation, so a stale entry keyed only by
line text could re-validate a later, unrelated export's byte-identical line
against the wrong node (#293).

Coverage: `sand-commands`' `export_registry::family_coverage` harness (four
lifecycle properties x nine families) and
`sand-core/tests/export_registry_scope.rs` (the same properties through the
real pipeline).
