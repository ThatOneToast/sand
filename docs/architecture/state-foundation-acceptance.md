# Unified State Foundation Acceptance Matrix

This matrix tracks issue #298 against executable evidence at the implementation branch's starting commit (`7485f6aa0c3888b0bcadd3227c2c866a146b7315`) and the current implementation. “Partial” is not treated as completion.

| Requirement | Starting evidence | Current status | Required validation |
| --- | --- | --- | --- |
| Four scoped `State` schemas and concrete bound views | Merged by #302; `sand-macros/src/entity_state.rs` and `state_schema_lifecycle.rs` | Complete for current executor and global singleton; collection binding remains absent | Compile/export/cardinality tests |
| Canonical score, fixed score, flag, timer, cooldown, enum, marker, typed data vocabulary | Score-backed types existed; marker and typed data were rejected/absent | Partial: aliases, enum derive, empty markers, and global component-owned `Data<T>` lowering work; fixed-point field scaling and entity/player keyed data remain incomplete | Numeric/storage behavior tests |
| Independent component identity, ownership, presence, attach/detach | Per-schema fields existed without presence | Implemented for score-backed components, including player detach suppression and owned dirty cleanup | Reload/reattach/detach export and live tests |
| Entity/living lifecycle | Explicitly rejected by #302 | Partial: dependencies, explicit attachment/adoption, and presence-constrained ticks work; scan sharing by cadence is incomplete | Multi-entity live adoption, migration, unload/re-observation passed on 26.2 |
| Optional lifecycle hooks | Descriptor named `StateLifecycle`; no trait | Implemented trait/registration for provision, initialize, tick, reconcile, migrate, cleanup; hooks are version-gated | Compile/export/hook-order tests |
| Per-component migrations | Archetype migrations only | Implemented contiguous declared component transitions for explicit attachment and observed player/global lifecycle; composition migrations remain absent | Gap and value-preservation tests |
| Nested `StateBundle` | Absent | Implemented concrete nested views, command deduplication, independent versions, and exact-scope rejection | Compile/export duplicate-work tests |
| Typed `StateQuery` | Typed selector `EntityQuery` only | Implemented required, optional, and forbidden component/bundle presence; optional callbacks are runtime guarded | Compile/export executor/filter tests |
| Tick and event systems | Existing scheduler and event dispatcher | Partial: deterministic function/grouped-impl tick systems, cadence, and grouped event adapters work; dependency planning, ordering diagnostics, and scan coalescing remain | Compile/export tests pass; shared-planning behavior remains unverified |
| Archetype composition | Generic archetype supports one State schema | Absent: concrete component/bundle composition helpers are still required | Summon/adopt/attach migration tests |
| Global resources | Global bound State from #302 | Global singleton lifecycle and component-owned typed storage fields work | Export tests and live 26.2 initialization/reload value-preservation checks pass |
| Shared collision-safe identities | State used logical objective hashing; schedules had their own collision allocator | Partial: state collisions reject with owners and schedules resolve deterministically; shared allocator integration and forced cross-kind collision tests remain | Fresh-process/order/collision tests |
| Structured compiler diagnostics | Field/schema proc-macro diagnostics and archetype diagnostics existed | Partial: scope and migration errors are static; system dependency/cardinality/backend diagnostics remain | Compile/export failure tests |
| API contracts and discoverability | State generated-contract enforcement existed | Extended for StateBundle, StateQuery, StateEnum, lifecycle, and systems without pending scopes/exclusions | Contract CLI discovery, downstream consumer builds, and rustdoc audits pass |
| Tutorial, docs, migration guide | #302 architecture docs | Added a complete isolated tutorial, book chapter, README update, and persisted-objective migration procedure | Tutorial workspace check/export, CLI pack build, and mdBook build pass |
| Minecraft 26.2 runtime validation | Repository harness exists | Partial: two real-server harnesses passed entity adoption, two-owner isolation, nested attachment, query filtering, system ticking, v1→v2 migration, global typed-data reload, repeated attachment, unload/re-observation, ownership-safe detach/cleanup, and preservation of unrelated data | Multiple online players were not exercised because no automated Minecraft clients are available in the harness environment |

The implementation is not closure-ready while any row remains partial, absent, or lacks its required runtime evidence.
