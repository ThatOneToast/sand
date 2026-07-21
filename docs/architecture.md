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
