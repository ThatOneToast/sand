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
`within` edges). Advancement-backed graph parents and participant context
propagation are later roadmap phases and are not modeled as implemented
behavior here.
